/// Quick verification that new APIs work
use hxcfe::Hxcfe;

fn main() {
    println!("=== Testing New HxCFE Features ===\n");
    
    let hxcfe = Hxcfe::get();
    
    // Test 1: Image duplication
    println!("1. Testing image duplicate...");
    match hxcfe.load("tests/EXPERTS.HFE") {
        Ok(img) => {
            println!("   ✓ Loaded EXPERTS.HFE");
            match img.duplicate() {
                Ok(copy) => {
                    println!("   ✓ Duplicated image successfully");
                    println!("     Original: {} tracks x {} sides = {} bytes",
                             img.nb_tracks(), img.nb_sides(), img.size());
                    println!("     Copy:     {} tracks x {} sides = {} bytes",
                             copy.nb_tracks(), copy.nb_sides(), copy.size());
                }
                Err(e) => println!("   ✗ Failed to duplicate: {:?}", e),
            }
        }
        Err(e) => println!("   ✗ Failed to load: {}", e),
    }
    
    // Test 2: Interface utilities
    println!("\n2. Testing interface utilities...");
    let encoding_name = hxcfe.get_track_encoding_name(0);
    println!("   ✓ Track encoding 0: {}", encoding_name);
    
    // Test 3: Filesystem space query
    println!("\n3. Testing filesystem operations...");
    match hxcfe.load("tests/EXPERTS.HFE") {
        Ok(img) => {
            if let Some(fs) = hxcfe.file_system_manager() {
                fs.select_fs(15); // FS_720KB_MSDOS_FAT12
                if fs.mount(&img) >= 0 {
                    let free = fs.free_space();
                    let total = fs.total_space();
                    println!("   ✓ Mounted image");
                    println!("     Free: {} bytes, Total: {} bytes", free, total);
                    fs.umount();
                } else {
                    println!("   - Could not mount (may not be FAT12)");
                }
            }
        }
        Err(e) => println!("   ✗ Failed to load: {}", e),
    }
    
    println!("\n=== All Tests Completed ===");
}
