use hxcfe::{FileSystemManager, Hxcfe};
const DSK_FNAME: &'static str = "tests/ECOLE_BUISSONNIERE_(OVERLANDERS).DSK";

/// Get filesystem name from the C library constants (best guess)
fn fs_name(id: i32) -> &'static str {
    match id {
        0 => "FS_720KB_ATARI_FAT12",
        6 => "FS_CPC_DD_FAT12",
        9 => "FS_MSX2_DD_FAT12",
        15 => "FS_720KB_MSDOS_FAT12",
        16 => "FS_5P25_300RPM_1200KB_MSDOS_FAT12",
        17 => "FS_1_44MB_MSDOS_FAT12",
        _ => "Unknown FS",
    }
}

fn display_dir(fsmngr: &FileSystemManager, folder: &str, level: u32) -> i32 {
    if let Ok(dir_handle) = dbg!(fsmngr.open_dir(folder)) {
        loop {
            let mut dir = false;
            if let Ok(dirent) = dir_handle.read() {
                for _ in 0..level {
                    print!("    ")
                }
                if dirent.is_dir() {
                    print!(">");
                    dir = true;
                } else {
                    print!(" ");
                }
                println!("{} <{}>", dirent.entry_name(), dirent.size());

                if dir {
                    let mut fullpath = folder.to_owned();
                    if !fullpath.ends_with("/") {
                        fullpath.push('/');
                    }
                    fullpath.push_str(dirent.entry_name());
                    if dirent.entry_name() != "." && dirent.entry_name() != ".." {
                        if display_dir(fsmngr, &fullpath, level + 1) < 0 {
                            dir_handle.close();
                            return 0;
                        }
                    }
                }
            } else {
                return 0;
            }
        }
    }
    return 0;
}

fn main() {
    let hxcfe = Hxcfe::get();
    let img = hxcfe
        .load(DSK_FNAME)
        .expect(&format!("Unable to read {}", DSK_FNAME));

    let interface = img.interface_mode();
    println!("Interface mode {} {}", interface.name(), interface.description());
    println!("Size: {}", img.size());
    println!("Nb sectors: {}", img.nb_sectors());
    println!("Nb sides: {}", img.nb_sides());

    let fsmngr = hxcfe.file_system_manager().unwrap();
    
    println!("\nAttempting to mount filesystem...");
    println!("Note: Many CPC game disks use custom formats without standard filesystems.");
    
    // Try common filesystem types for different platforms
    let fs_types = [
        6,  // CPC FAT12
        15, // MS-DOS FAT12 720KB
        0,  // Atari FAT12
        9,  // MSX2 FAT12
        16, // MS-DOS 1.2MB
        17, // MS-DOS 1.44MB
    ];
    
    let mut mounted = false;
    
    for fs_id in fs_types {
        fsmngr.select_fs(fs_id);
        let result = fsmngr.mount(&img);
        println!("  Trying {:30} (ID {:2}): result = {}", fs_name(fs_id), fs_id, result);
        if result >= 0 {
            println!("\n✓ Successfully mounted with {} (ID {})", fs_name(fs_id), fs_id);
            mounted = true;
            break;
        }
    }
    
    if mounted {
        println!("\nDirectory listing:");
        display_dir(&fsmngr, "/", 0);
        fsmngr.umount();
    } else {
        println!("\n✗ ERROR: Could not mount image with any known filesystem type");
        println!("\nPossible reasons:");
        println!("  - This disk uses a custom/non-standard filesystem");
        println!("  - This is a game disk with copy protection");
        println!("  - The disk needs sector-level access instead of filesystem mounting");
        println!("\nTry using sector_access() API for raw sector reading instead.");
    }
}
