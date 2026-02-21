use hxcfe::{FileSystemId, HeadId, Hxcfe, ImageFormat, SectorId, TrackEncoding, TrackId};
const DSK_FNAME: &'static str = "tests/ECOLE_BUISSONNIERE_(OVERLANDERS).DSK";

#[test]
fn load_from_buffer_and_save_to_buffer() {
    use std::io::Read;

    let hxcfe = Hxcfe::get();

    // Load original file into memory
    let mut file = std::fs::File::open(DSK_FNAME).expect(&format!("Unable to open {}", DSK_FNAME));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Unable to read file");

    println!("Loaded {} bytes from {}", buffer.len(), DSK_FNAME);

    // Load from buffer
    let img = hxcfe
        .load_from_buffer(&buffer, "test.dsk")
        .expect("Unable to load from buffer");

    println!("Image loaded from buffer successfully");
    println!("  Tracks: {}", img.nb_tracks());
    println!("  Sides: {}", img.nb_sides());
    println!("  Size: {} bytes", img.size());

    // Save to buffer in HFE format
    let hfe_buffer = img
        .save_to_buffer(ImageFormat::HxcHfe)
        .expect("Unable to save to buffer");

    println!("Saved to HFE buffer: {} bytes", hfe_buffer.len());

    // Verify we can load the HFE buffer
    let img2 = hxcfe
        .load_from_buffer(&hfe_buffer, "test.hfe")
        .expect("Unable to load HFE from buffer");

    println!("HFE image loaded from buffer successfully");
    println!("  Tracks: {}", img2.nb_tracks());
    println!("  Sides: {}", img2.nb_sides());

    // Both images should have the same basic properties
    assert_eq!(img.nb_tracks(), img2.nb_tracks());
    assert_eq!(img.nb_sides(), img2.nb_sides());
}

#[test]
fn load_dsk() {
    let hxcfe = Hxcfe::get();
    let img = hxcfe
        .load(DSK_FNAME)
        .expect(&format!("Unable to read {}", DSK_FNAME));

    let interface = img
        .interface_mode()
        .expect("Could not determine interface mode");
    println!(
        "Interface mode {} {}",
        interface.name(),
        interface.description()
    );
    println!("Size: {}", img.size());
    println!("Nb sectors: {}", img.nb_sectors());
    println!("Nb sides: {}", img.nb_sides());

    let fsmngr = hxcfe.file_system_manager().unwrap();

    // Try different filesystem types to mount the disk
    let fs_types = [
        ("CPC_DD_FAT12", 6),
        ("720KB_MSDOS_FAT12", 15),
        ("720KB_ATARI_FAT12", 0),
        ("MSX2_DD_FAT12", 9),
        ("1200KB_MSDOS_FAT12", 16),
        ("1440KB_MSDOS_FAT12", 17),
    ];

    let mut mounted = false;
    for (name, fs_id) in &fs_types {
        println!("\nTrying {} (ID {})", name, fs_id);
        fsmngr.select_fs(FileSystemId::from_i32(*fs_id).expect("Invalid filesystem ID"));
        let result = fsmngr.mount(&img);
        println!("  Mount result: {}", result);

        if result == 0 {
            println!("  ✓ Successfully mounted with {}", name);
            mounted = true;
            break;
        }
    }

    if mounted {
        // Try to open root directory and list files
        println!("\nAttempting to read root directory:");
        match fsmngr.open_dir("/") {
            Ok(dir) => {
                println!("  ✓ Successfully opened root directory");

                let mut found_files = Vec::new();
                let expected_files = ["ECOLE.BIN", "ECOLE.OV1", "ECOLE.OV2"];

                loop {
                    match dir.read() {
                        Ok(entry) => {
                            let name = entry.entry_name();
                            let size = entry.size();
                            let is_dir = entry.is_dir();

                            println!(
                                "  {} {:30} {:>10} bytes",
                                if is_dir { "📁" } else { "📄" },
                                name,
                                size
                            );

                            // Check if this is one of the expected files
                            if expected_files.iter().any(|&f| f == name) {
                                found_files.push(name.to_string());
                            }
                        }
                        Err(_) => break, // End of directory
                    }
                }

                dir.close();

                println!("\nExpected files check:");
                for expected in &expected_files {
                    if found_files.iter().any(|f| f == expected) {
                        println!("  ✓ Found {}", expected);
                    } else {
                        println!("  ✗ Missing {}", expected);
                    }
                }

                // Assert that we found at least one of the expected files
                assert!(
                    !found_files.is_empty(),
                    "Should have found at least one file from: {:?}",
                    expected_files
                );
            }
            Err(error_code) => {
                println!("  ✗ Failed to open directory: error code {}", error_code);
            }
        }

        fsmngr.umount();
    } else {
        // Filesystem mount failed, try sector-level access
        println!("\n⚠ Could not mount with any filesystem type.");
        println!("Attempting raw sector access to read CPC directory...\n");
        println!("CPC catalog: stored in first 2 sectors (0xC1, 0xC2)");
        println!("Format: 64 CPM directory entries, filename at bytes 1-12\n");

        let expected_files = ["ECOLE.BIN", "ECOLE.OV1", "ECOLE.OV2"];
        let mut found_files = Vec::new();

        // Get sector access API
        let sector_access = img.sector_access().expect("Failed to get sector access");

        // Read first 2 directory sectors (CPC DATA format: sector IDs 0xC1, 0xC2)
        // Each sector is 512 bytes, containing 16 directory entries of 32 bytes each
        for sector_id in [0xC1, 0xC2] {
            if let Some(sconfig) = sector_access.search_sector(
                HeadId::new(0),
                TrackId::new(0),
                SectorId::new(sector_id),
                TrackEncoding::IsoibmMfm,
            ) {
                println!("Reading Track 0, Side 0, Sector ID {:#X}:", sector_id);
                println!("  Sector size: {} bytes", sconfig.sector_size());

                let data = sconfig.read();
                println!("  Data length: {} bytes", data.len());

                // Show first 128 bytes of first sector as hexdump for debugging
                if sector_id == 0xC1 && data.len() >= 128 {
                    println!("\n  First 128 bytes (hexdump):");
                    for line in 0..8 {
                        let offset = line * 16;
                        print!("    {:04X}:  ", offset);
                        for b in &data[offset..offset + 16] {
                            print!("{:02X} ", b);
                        }
                        print!(" | ");
                        for b in &data[offset..offset + 16] {
                            let ch = if *b >= 32 && *b < 127 {
                                *b as char
                            } else {
                                '.'
                            };
                            print!("{}", ch);
                        }
                        println!();
                    }
                    println!();
                }

                // Parse CPM directory entries (each 32 bytes)
                // 512 bytes / 32 = 16 entries per sector
                let entries_per_sector = data.len() / 32;
                println!("  Parsing {} directory entries:\n", entries_per_sector);

                for i in 0..entries_per_sector {
                    let entry_offset = i * 32;
                    if entry_offset + 32 > data.len() {
                        break;
                    }
                    let entry = &data[entry_offset..entry_offset + 32];

                    // Byte 0 = user number (0xE5 = deleted, 0x00-0x0F = active)
                    let user = entry[0];

                    // Debug: show first 4 entries of first sector
                    if sector_id == 0xC1 && i < 4 {
                        println!(
                            "    Entry {} [user={:#02X}]: {}",
                            i,
                            user,
                            entry[0..16]
                                .iter()
                                .map(|b| format!("{:02X}", b))
                                .collect::<Vec<_>>()
                                .join(" ")
                        );
                    }

                    // Skip deleted entries (0xE5) or invalid user numbers
                    if user == 0xE5 {
                        continue; // Deleted file
                    }

                    // CP/M format: bytes 1-8 = filename, bytes 9-11 = extension
                    let filename_bytes = &entry[1..9];
                    let ext_bytes = &entry[9..12];

                    // Convert to string, strip high bits (bit 7 used for flags) and trim spaces
                    let filename: String = filename_bytes
                        .iter()
                        .map(|&b| (b & 0x7F) as char)
                        .collect::<String>()
                        .trim_end()
                        .to_string();

                    let ext: String = ext_bytes
                        .iter()
                        .map(|&b| (b & 0x7F) as char)
                        .collect::<String>()
                        .trim_end()
                        .to_string();

                    if !filename.is_empty() && filename.chars().all(|c| c.is_ascii_graphic()) {
                        let full_name = if ext.is_empty() {
                            filename.clone()
                        } else {
                            format!("{}.{}", filename, ext)
                        };

                        println!("    Entry {}: User {}, Name: '{}'", i, user, full_name);

                        // Check if this is one of the expected files
                        if expected_files.iter().any(|&f| f == full_name) {
                            if !found_files.contains(&full_name) {
                                found_files.push(full_name);
                            }
                        }
                    }
                }

                println!();
            }
        }

        println!("\nExpected files check (via sector access):");
        for expected in &expected_files {
            if found_files.iter().any(|f| f == expected) {
                println!("  ✓ Found {}", expected);
            } else {
                println!("  ✗ Missing {}", expected);
            }
        }

        // Assert that we found at least one of the expected files
        assert!(
            !found_files.is_empty(),
            "Should have found at least one file from: {:?}\nFound files: {:?}",
            expected_files,
            found_files
        );
    }
}

#[test]
fn load_missing_dsk() {
    let hxcfe = Hxcfe::get();
    let res = dbg!(hxcfe.load("missing.dsk"));
    assert!(res.is_err())
}
