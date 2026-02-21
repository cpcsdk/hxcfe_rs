use hxcfe::DiskLayout;

fn main() {
    println!("=== Available Disk Layouts ===\n");
    
    // Show total count
    let all_layouts = DiskLayout::all();
    println!("Total layouts: {}\n", all_layouts.len());
    
    // Show a few common layouts
    println!("Common disk layouts:");
    let common_layouts = [
        DiskLayout::DosDd720kb,
        DiskLayout::DosHd1m44,
        DiskLayout::DosEd2m88,
        DiskLayout::AmstradcpcDd,
        DiskLayout::AtaristDd720kb,
    ];
    
    for layout in common_layouts {
        println!("  - {} (ID: {})", layout, layout as usize);
    }
    
    // Test from_str parsing
    println!("\n=== Testing from_str ===");
    let test_names = [
        "DOS_HD_1_44MB",
        "AmstradCPC_DD",
        "AtariST_DD_720KB",
        "invalid_name",
    ];
    
    for name in test_names {
        match DiskLayout::from_str(name) {
            Some(layout) => println!("  '{}' -> {} (ID: {})", name, layout, layout as usize),
            None => println!("  '{}' -> NOT FOUND", name),
        }
    }
    
    // Test from_usize conversion
    println!("\n=== Testing from_usize ===");
    let test_ids = [0, 11, 22, 23, 85, 100];
    
    for id in test_ids {
        match DiskLayout::from_usize(id) {
            Some(layout) => println!("  ID {} -> {}", id, layout),
            None => println!("  ID {} -> INVALID", id),
        }
    }
    
    // Show some specialized layouts
    println!("\n=== Specialized Layouts ===");
    let specialized = [
        DiskLayout::Akais950Hd1600kb,
        DiskLayout::EnsoniqDd800kb,
        DiskLayout::RolandDdW30,
        DiskLayout::Korgt3Hd1m6,
        DiskLayout::X680002hd1232kb,
    ];
    
    for layout in &specialized {
        println!("  - {}", layout.layout_name());
    }
}
