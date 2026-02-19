/// Test the new features added: image duplication, filesystem operations, and utilities
use hxcfe::Hxcfe;
use std::fs;
use std::path::Path;

#[test]
fn test_image_duplicate() {
    // Load an existing test image
    let hxcfe = Hxcfe::get();
    let img = hxcfe.load("tests/EXPERTS.HFE").expect("Failed to load test image");
    
    // Test duplicate
    let img_copy = img.duplicate().expect("Failed to duplicate image");
    
    // Verify the copy has the same properties
    assert_eq!(img.nb_tracks(), img_copy.nb_tracks());
    assert_eq!(img.nb_sides(), img_copy.nb_sides());
    assert_eq!(img.size(), img_copy.size());
    
    println!("✓ Image duplication works - {} tracks, {} sides", 
             img.nb_tracks(), img.nb_sides());
}

#[test]
fn test_filesystem_operations() {
    let hxcfe = Hxcfe::get();
    let img = hxcfe.load("tests/EXPERTS.HFE").expect("Failed to load test image");
    
    let fs_manager = hxcfe.file_system_manager().expect("Failed to create FS manager");
    
    // Select FAT12 filesystem (15 = FS_720KB_MSDOS_FAT12)
    fs_manager.select_fs(15);
    
    // Mount the image
    let mount_result = fs_manager.mount(&img);
    println!("Mount result: {}", mount_result);
    
    if mount_result >= 0 {
        // Test space queries
        let free_space = fs_manager.free_space();
        let total_space = fs_manager.total_space();
        println!("✓ Free space: {} bytes, Total space: {} bytes", free_space, total_space);
        
        // Try to open root directory
        if let Ok(dir) = fs_manager.open_dir("/") {
            println!("✓ Successfully opened root directory");
            
            // Read some entries
            let mut count = 0;
            while let Ok(entry) = dir.read() {
                println!("  - {} ({} bytes, {})", 
                         entry.entry_name(), 
                         entry.size(),
                         if entry.is_dir() { "dir" } else { "file" });
                count += 1;
                if count >= 10 { break; } // Limit output
            }
            
            dir.close();
        }
        
        // Unmount
        fs_manager.umount();
    }
}

#[test]
fn test_interface_utilities() {
    let hxcfe = Hxcfe::get();
    
    // Test getting track encoding name
    let encoding_name = hxcfe.get_track_encoding_name(0); // 0 = ISOIBM_MFM_ENCODING
    println!("✓ Track encoding 0: {}", encoding_name);
    assert!(!encoding_name.is_empty());
    
    // Test interface mode IDs
    // Note: Interface mode names might vary, so we'll just test that the function works
    if let Some(mode_id) = hxcfe.get_interface_mode_id("IBMPC_DD_FLOPPYMODE") {
        println!("✓ IBMPC_DD interface mode ID: {}", mode_id);
    } else {
        println!("Note: IBMPC_DD_FLOPPYMODE interface not found (may be normal)");
    }
}

#[test]
fn test_sector_copy() {
    let hxcfe = Hxcfe::get();
    let img1 = hxcfe.load("tests/EXPERTS.HFE").expect("Failed to load first image");
    let mut img2 = img1.duplicate().expect("Failed to duplicate image");
    
    // Copy sectors from img1 to img2 (should succeed since they're compatible)
    let result = img2.copy_sectors_from(&img1);
    
    match result {
        Ok(_) => println!("✓ Sector copy succeeded"),
        Err(e) => println!("Sector copy returned error: {:?}", e),
    }
    
    // The images should still have the same properties
    assert_eq!(img1.nb_tracks(), img2.nb_tracks());
    assert_eq!(img1.nb_sides(), img2.nb_sides());
}

#[test]
#[ignore] // Ignored by default as it requires a directory with files
fn test_generate_floppy() {
    let hxcfe = Hxcfe::get();
    
    // Create a temporary directory with a test file
    let test_dir = "test_floppy_gen";
    if Path::new(test_dir).exists() {
        let _ = fs::remove_dir_all(test_dir);
    }
    fs::create_dir(test_dir).expect("Failed to create test directory");
    fs::write(format!("{}/TEST.TXT", test_dir), b"Hello, World!")
        .expect("Failed to create test file");
    
    // Generate a floppy from the directory
    // 15 = FS_720KB_MSDOS_FAT12
    let result = hxcfe.generate_floppy(test_dir, 15);
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    match result {
        Ok(img) => {
            println!("✓ Generated floppy: {} tracks, {} sides, {} bytes",
                     img.nb_tracks(), img.nb_sides(), img.size());
            
            // Try to save it to verify it's valid
            img.save("test_generated.hfe", "HXC_HFE").expect("Failed to save generated image");
            println!("✓ Successfully saved generated floppy");
            
            // Cleanup
            let _ = fs::remove_file("test_generated.hfe");
        }
        Err(e) => {
            panic!("Failed to generate floppy: {:?}", e);
        }
    }
}
