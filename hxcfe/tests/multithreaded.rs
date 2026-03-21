/// Multithreaded stress tests.
///
/// Every test spawns N threads; each thread owns its own `Img` instance
/// (loaded from a different file or duplicated from a common source) and
/// performs independent operations.  All C calls are serialised through the
/// global `parking_lot::Mutex` in `Hxcfe`, so these tests act as a
/// correctness check for that locking.
///
/// Run with the thread-sanitiser for full data-race detection:
///
///   RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test -p hxcfe              \
///       --target x86_64-unknown-linux-gnu --test multithreaded
use std::{sync::Barrier, thread};

use hxcfe::{HeadId, Hxcfe, SectorId, TrackEncoding, TrackId};

const HFE: &str = "tests/EXPERTS.HFE";
const DSK: &str = "tests/EXPERTS.DSK";

// ── helpers ───────────────────────────────────────────────────────────────────

fn ctx() -> &'static Hxcfe {
    Hxcfe::get()
}

/// Scan every sector on track 0 / side 0 using get_next_sector.
fn scan_track_0(img: &hxcfe::Img) -> usize {
    let sa = img.sector_access().expect("sector_access");
    sa.reset_search_track_position();
    let mut count = 0usize;
    while let Some(cfg) = sa.get_next_sector(HeadId::new(0), TrackId::new(0), TrackEncoding::IsoibmMfm) {
        let _ = cfg.read(); // touch the data under the lock
        count += 1;
    }
    count
}

/// Read a specific sector (0xC1) and return its length.
fn read_sector_c1(img: &hxcfe::Img) -> usize {
    let sa = img.sector_access().expect("sector_access");
    sa.search_sector(HeadId::new(0), TrackId::new(0), SectorId::new(0xC1), TrackEncoding::IsoibmMfm)
        .map(|cfg| cfg.read().len())
        .unwrap_or(0)
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// N threads each load the same HFE file and scan track 0 concurrently.
/// All sector counts must agree with the single-threaded baseline.
#[test]
fn concurrent_hfe_reads() {
    const THREADS: usize = 8;

    // Establish baseline on the main thread.
    let baseline = scan_track_0(&ctx().load(HFE).expect("baseline load"));
    assert!(baseline > 0, "baseline must find at least one sector");

    let barrier = std::sync::Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let b = barrier.clone();
            thread::spawn(move || {
                let img = ctx().load(HFE).expect("thread load");
                // All threads start scanning at the same moment.
                b.wait();
                let count = scan_track_0(&img);
                assert_eq!(count, baseline, "sector count must be deterministic");
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

/// N/2 threads load HFE images while N/2 threads load DSK images; all run
/// concurrently and verify the sector 0xC1 is readable with consistent size.
#[test]
fn concurrent_mixed_format_reads() {
    const THREADS_PER_FORMAT: usize = 4;

    let barrier = std::sync::Arc::new(Barrier::new(THREADS_PER_FORMAT * 2));

    let mut handles = Vec::new();

    for format in [HFE, DSK] {
        for _ in 0..THREADS_PER_FORMAT {
            let b = barrier.clone();
            handles.push(thread::spawn(move || {
                let img = ctx().load(format).expect("thread load");
                b.wait();
                let len = read_sector_c1(&img);
                assert!(len > 0, "sector 0xC1 must have non-zero size in {format}");
            }));
        }
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}

/// Each thread creates and destroys SectorAccess many times.
/// Tests that concurrent init/deinit calls are serialised correctly.
#[test]
fn concurrent_sector_access_create_drop() {
    const THREADS: usize = 6;
    const ITERATIONS: usize = 30;

    let barrier = std::sync::Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let b = barrier.clone();
            thread::spawn(move || {
                let img = ctx().load(HFE).expect("load");
                b.wait();
                for _ in 0..ITERATIONS {
                    let sa = img.sector_access().expect("sector_access");
                    drop(sa);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

/// Each thread scans ALL tracks on its image concurrently.
/// A full scan exercises get_next_sector, SectorConfigArray, and reset across
/// many tracks while other threads do the same.
#[test]
fn concurrent_full_scan() {
    const THREADS: usize = 4;

    let barrier = std::sync::Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let b = barrier.clone();
            thread::spawn(move || {
                let img = ctx().load(HFE).expect("load");
                let nb_tracks = img.nb_tracks() as u32;
                let nb_sides = img.nb_sides() as u32;
                b.wait();

                let sa = img.sector_access().expect("sector_access");
                let mut total = 0usize;

                for track in 0..nb_tracks {
                    for side in 0..nb_sides {
                        let sca = sa.all_track_sectors(
                            HeadId::new(side as i32),
                            TrackId::new(track as i32),
                            TrackEncoding::IsoibmMfm,
                        );
                        if let Some(array) = sca {
                            for i in 0..array.nb_sectors() {
                                let cfg = array.sector_config(i);
                                let _ = cfg.read();
                                total += 1;
                            }
                        }
                    }
                }

                assert!(total > 0, "full scan must find sectors");
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

/// Each thread alternates between loading HFE and DSK images inside a loop,
/// dropping each image before loading the next.  Tests that concurrent
/// hxcfe_imgInitLoader / hxcfe_imgUnload / hxcfe_imgDeInitLoader sequences
/// (from Img::Drop) do not race with new loads on other threads.
#[test]
fn concurrent_load_drop_cycle() {
    const THREADS: usize = 6;
    const ITERATIONS: usize = 10;

    let barrier = std::sync::Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|i| {
            let b = barrier.clone();
            thread::spawn(move || {
                // Stagger the starting files so threads are not always in lockstep.
                let files = if i % 2 == 0 { [HFE, DSK] } else { [DSK, HFE] };
                b.wait();
                for iter in 0..ITERATIONS {
                    let file = files[iter % 2];
                    let img = ctx().load(file).expect("load");
                    // Brief work before drop.
                    assert!(img.nb_tracks() > 0);
                    drop(img); // exercises Img::Drop lock
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

/// While one thread repeatedly creates/drops ImgLoaderManager, other threads
/// do normal sector reads.  Tests concurrent use of the loader subsystem.
#[test]
fn concurrent_loader_manager_with_readers() {
    const READER_THREADS: usize = 4;
    const LOADER_THREADS: usize = 2;
    const ITERATIONS: usize = 20;

    let barrier = std::sync::Arc::new(Barrier::new(READER_THREADS + LOADER_THREADS));
    let mut handles = Vec::new();

    // Reader threads
    for _ in 0..READER_THREADS {
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            let img = ctx().load(HFE).expect("load");
            b.wait();
            for _ in 0..ITERATIONS {
                let _ = read_sector_c1(&img);
            }
        }));
    }

    // Loader-manager threads
    for _ in 0..LOADER_THREADS {
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..ITERATIONS {
                let mgr = ctx().loaders_manager().expect("loaders_manager");
                let n = mgr.nb_loaders();
                assert!(n > 0);
                drop(mgr);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}
