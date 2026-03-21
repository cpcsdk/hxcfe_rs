/// Tests that verify the Drop implementations on SectorAccess, SectorConfig, and
/// SectorConfigArray do not leak, double-free, or corrupt memory.
///
/// These tests work without an external sanitiser: a double-free or heap corruption
/// will typically abort the process immediately, which Rust's test harness reports as
/// a test failure.  For leak detection, run with AddressSanitizer:
///
///   RUSTFLAGS="-Z sanitizer=address" cargo +nightly test -p hxcfe \
///       --target x86_64-unknown-linux-gnu -- sector_access_drops
use hxcfe::{HeadId, Hxcfe, SectorId, TrackEncoding, TrackId};

const HFE: &str = "tests/EXPERTS.HFE";
const DSK: &str = "tests/EXPERTS.DSK";

fn hxcfe() -> &'static hxcfe::Hxcfe {
    Hxcfe::get()
}

// ── SectorAccess ──────────────────────────────────────────────────────────────

/// Creating and immediately dropping SectorAccess many times must not leak or crash.
#[test]
fn sector_access_created_and_dropped_repeatedly() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");

    for _ in 0..50 {
        let _sa = img.sector_access().expect("sector_access");
        // dropped here — hxcfe_deinitSectorAccess must be called each iteration
    }
}

/// SectorAccess dropped after the Img is still alive: no use-after-free.
#[test]
fn sector_access_dropped_before_img() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    {
        let _sa = img.sector_access().expect("sector_access");
    } // SectorAccess drops here
    // img is still alive; verify it is still usable
    assert!(img.nb_tracks() > 0);
}

// ── SectorConfig (owned = true) ───────────────────────────────────────────────

/// search_sector returns an owned SectorConfig; dropping it must call
/// hxcfe_freeSectorConfig exactly once.  Repeat to surface any corruption.
#[test]
fn search_sector_drop_is_clean() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    let sa = img.sector_access().expect("sector_access");

    for _ in 0..20 {
        let cfg = sa.search_sector(
            HeadId::new(0),
            TrackId::new(0),
            SectorId::new(0xC1),
            TrackEncoding::IsoibmMfm,
        );
        // Drop cfg here; if owned=true path double-frees this will abort.
        drop(cfg);
    }
}

/// get_next_sector yields owned configs; iterate all sectors on a track and
/// drop each one, then do the same on a second pass to confirm no state is left.
#[test]
fn get_next_sector_drop_is_clean() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    let sa = img.sector_access().expect("sector_access");

    for _pass in 0..3 {
        sa.reset_search_track_position();
        let mut count = 0usize;
        while let Some(cfg) = sa.get_next_sector(
            HeadId::new(0),
            TrackId::new(0),
            TrackEncoding::IsoibmMfm,
        ) {
            // Read data while cfg is alive, then let it drop.
            let _data = cfg.read();
            count += 1;
            // cfg drops (owned=true → hxcfe_freeSectorConfig) here
        }
        assert!(count > 0, "expected at least one sector on track 0");
    }
}

/// read data from an owned SectorConfig, verify it is consistent across passes.
#[test]
fn owned_sector_config_data_consistent_after_multiple_drops() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    let sa = img.sector_access().expect("sector_access");

    let read_once = || -> Vec<u8> {
        sa.search_sector(
            HeadId::new(0),
            TrackId::new(0),
            SectorId::new(0xC1),
            TrackEncoding::IsoibmMfm,
        )
        .map(|cfg| cfg.read().to_vec())
        .expect("sector C1 must exist")
    };

    let first = read_once();
    let second = read_once();
    assert_eq!(first, second, "sector data must be stable across reads");
}

// ── SectorConfigArray + borrowed SectorConfig (owned = false) ─────────────────

/// Drop the whole array without extracting any element.
/// Exercises the array-level free path.
#[test]
fn sector_config_array_dropped_without_borrowing() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    let sa = img.sector_access().expect("sector_access");

    for _ in 0..20 {
        let _arr = sa.all_track_sectors(
            HeadId::new(0),
            TrackId::new(0),
            TrackEncoding::IsoibmMfm,
        );
        // SectorConfigArray drops here: all elements freed + pointer array freed
    }
}

/// Extract every element from the array, drop the borrowed configs first,
/// then drop the array.  The borrowed configs have owned=false so they must
/// NOT call hxcfe_freeSectorConfig; the array Drop does that.
#[test]
fn borrowed_sector_configs_dropped_before_array_no_double_free() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    let sa = img.sector_access().expect("sector_access");

    let arr = sa
        .all_track_sectors(HeadId::new(0), TrackId::new(0), TrackEncoding::IsoibmMfm)
        .expect("track 0 must have sectors");

    let n = arr.nb_sectors();
    assert!(n > 0);

    // Borrow each config, read it, then drop it (owned=false → no free).
    for i in 0..n {
        let cfg = arr.sector_config(i);
        let _data = cfg.read();
        // cfg dropped here — must be a no-op (owned=false)
    }

    // arr dropped here — must free all n configs exactly once
}

/// Interleave: extract config, drop config, extract the same config again.
/// If the array freed on each borrow-drop this would use-after-free.
#[test]
fn re_borrow_after_borrowed_config_drop_is_safe() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");
    let sa = img.sector_access().expect("sector_access");

    let arr = sa
        .all_track_sectors(HeadId::new(0), TrackId::new(0), TrackEncoding::IsoibmMfm)
        .expect("track 0 must have sectors");

    if arr.nb_sectors() == 0 {
        return;
    }

    let data_a = {
        let cfg = arr.sector_config(0);
        cfg.read().to_vec() // cfg dropped here (no-op)
    };
    let data_b = {
        let cfg = arr.sector_config(0); // re-borrow same slot
        cfg.read().to_vec()
    };

    assert_eq!(data_a, data_b, "re-borrowed config must yield same data");
    // arr dropped here — frees the underlying SECTCFG once
}

// ── Cross-image: DSK file ─────────────────────────────────────────────────────

/// Same ownership checks on a DSK image to cover a different loader path.
#[test]
fn sector_access_drops_on_dsk() {
    let ctx = hxcfe();
    let img = ctx.load(DSK).expect("load DSK");
    let sa = img.sector_access().expect("sector_access");

    let arr = sa.all_track_sectors(
        HeadId::new(0),
        TrackId::new(0),
        TrackEncoding::IsoibmMfm,
    );

    if let Some(arr) = arr {
        let n = arr.nb_sectors();
        for i in 0..n {
            let _data = arr.sector_config(i).read().to_vec();
        }
        // arr drops here: all elements freed once, pointer array freed
    }
    // sa drops here: hxcfe_deinitSectorAccess called
}

/// Repeated full scan over all tracks on the HFE image — exercises all three
/// Drop paths together under load.  Any corruption surfaces quickly this way.
#[test]
fn full_scan_all_tracks_repeated() {
    let ctx = hxcfe();
    let img = ctx.load(HFE).expect("load");

    for _pass in 0..5 {
        let sa = img.sector_access().expect("sector_access");
        for track_n in 0..img.nb_tracks() {
            for side_n in 0..img.nb_sides() {
                if let Some(arr) = sa.all_track_sectors(
                    HeadId::new(side_n as i32),
                    TrackId::new(track_n as i32),
                    TrackEncoding::IsoibmMfm,
                ) {
                    for i in 0..arr.nb_sectors() {
                        let _data = arr.sector_config(i).read().to_vec();
                    }
                    // arr drops
                }
            }
        }
        // sa drops (hxcfe_deinitSectorAccess)
    }
}
