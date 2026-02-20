use hxcfe::{Hxcfe, TrackEncoding, HeadId, TrackId, SectorId, FileSystemId};

#[test]
fn test_cpc_experts_hfe() {
    let hxcfe = Hxcfe::get();
    
    // EXPERTS.HFE is an Amstrad CPC disk with CP/M catalog
    const HFE_FNAME: &str = "tests/EXPERTS.HFE";
    
    let img = hxcfe
        .load(HFE_FNAME)
        .expect(&format!("Unable to read {}", HFE_FNAME));

    let interface = img.interface_mode();
    println!("Interface mode: {} - {}", interface.name(), interface.description());
    println!("Size: {} bytes", img.size());
    println!("Nb sectors: {}", img.nb_sectors());
    println!("Nb sides: {}", img.nb_sides());
    println!("Nb tracks: {}", img.nb_tracks());

    // EXPERTS.HFE is an Amstrad CPC disk with CP/M catalog on first 2 sectors
    println!("\n--- Reading CPC Catalog (CP/M format) ---");
    println!("Catalog stored in first 2 sectors (0xC1, 0xC2)");
    
    let mut files_found = Vec::new();
    let sector_access = img.sector_access().expect("Failed to get sector access");
    
    // Read first 2 directory sectors (CPC DATA format: sector IDs 0xC1, 0xC2)
    for sector_id in [0xC1, 0xC2] {
        if let Some(sconfig) = sector_access.search_sector(HeadId::new(0), TrackId::new(0), SectorId::new(sector_id), TrackEncoding::IsoIbmMfm) {
            println!("\nReading Track 0, Side 0, Sector ID {:#X}:", sector_id);
            let data = sconfig.read();
            println!("  Data length: {} bytes", data.len());
            
            // Show first 128 bytes of first sector as hexdump
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
                        let ch = if *b >= 32 && *b < 127 { *b as char } else { '.' };
                        print!("{}", ch);
                    }
                    println!();
                }
                println!();
            }
            
            // Parse CPM directory entries (16 entries per 512-byte sector)
            let entries_per_sector = data.len() / 32;
            println!("  Parsing {} directory entries:", entries_per_sector);
            
            for i in 0..entries_per_sector {
                let entry_offset = i * 32;
                if entry_offset + 32 > data.len() {
                    break;
                }
                let entry = &data[entry_offset..entry_offset + 32];
                
                let user = entry[0];
                if user == 0xE5 {
                    continue; // Deleted file
                }
                
                // CP/M format: bytes 1-8 = filename, bytes 9-11 = extension
                let filename_bytes = &entry[1..9];
                let ext_bytes = &entry[9..12];
                
                let filename: String = filename_bytes.iter()
                    .map(|&b| (b & 0x7F) as char)
                    .collect::<String>()
                    .trim_end()
                    .to_string();
                
                let ext: String = ext_bytes.iter()
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
                    
                    if !files_found.contains(&full_name) {
                        files_found.push(full_name);
                    }
                }
            }
        }
    }
    
    println!("\n--- Summary ---");
    println!("Found {} unique files:", files_found.len());
    for file in &files_found {
        println!("  - {}", file);
    }
    
    // Assert we found at least one file
    assert!(!files_found.is_empty(), "Should have found at least one file in EXPERTS.HFE catalog");
}

// TODO: Implement test_create_amiga_disk once we understand the correct API
// for creating blank disk images
#[test]
#[ignore]
fn test_create_amiga_disk() {
    let hxcfe = Hxcfe::get();
    
    // Generate a blank Amiga disk using the correct API
    // The generate_floppy method expects a directory path (or empty string for blank)
    println!("Creating blank Amiga DD disk...");
    
    // Pass empty string to create a blank disk
    match hxcfe.generate_floppy("", FileSystemId::Amiga880KbDos) {
        Ok(img) => {
            let interface = img.interface_mode();
            println!("✓ Created: {} - {}", interface.name(), interface.description());
            println!("  Size: {} bytes", img.size());
            println!("  Nb sectors: {}", img.nb_sectors());
            println!("  Nb sides: {}", img.nb_sides());
            
            // Try to mount it with AmigaDOS filesystem
            let fsmngr = hxcfe.file_system_manager().unwrap();
            fsmngr.select_fs(FileSystemId::Amiga880KbDos);
            let result = fsmngr.mount(&img);
            
            println!("\nMounting as AmigaDOS: result = {}", result);
            
            if result == 0 {
                println!("✓ Successfully mounted blank Amiga disk");
                
                // Get filesystem info
                let free_space = fsmngr.free_space();
                let total_space = fsmngr.total_space();
                println!("  Free space: {} bytes", free_space);
                println!("  Total space: {} bytes", total_space);
                
                fsmngr.umount();
            } else {
                println!("⚠ Could not mount - disk may need formatting");
                println!("  (AmigaDOS filesystem requires proper format)");
            }
        }
        Err(e) => {
            println!("✗ Failed to create disk: {:?}", e);
            panic!("Disk generation failed");
        }
    }
}
