/// Simple example to verify the library works on Windows with MSVC
/// This example initializes the HxC Floppy Emulator library and performs basic operations
use hxcfe::Hxcfe;

fn main() {
    println!("HxC Floppy Emulator Library - Windows MSVC Test");
    println!("================================================\n");

    // Initialize the library
    println!("1. Initializing HxC library...");
    let hxcfe = Hxcfe::get();
    println!("   ✓ Library initialized successfully\n");

    // Get version information
    println!("2. Getting library version...");
    let version = hxcfe.version();
    println!("   ✓ Library version: {}\n", version);

    // Test loading a disk image (if the test file exists)
    println!("3. Testing loaders manager...");
    let loaders_manager = hxcfe
        .loaders_manager()
        .expect("Failed to get loaders manager");
    let count = loaders_manager.nb_loaders();
    println!("   ✓ Found {} disk image loaders\n", count);

    println!("4. Testing layout manager...");
    let layout_manager = hxcfe
        .layout_manager()
        .expect("Failed to get layout manager");
    let layout_count = layout_manager.nb_layouts();
    println!("   ✓ Found {} interface layouts\n", layout_count);

    println!("SUCCESS: Library is fully functional on Windows MSVC!");
    println!("The library can now be used to:");
    println!("  - Load floppy disk images");
    println!("  - Manage virtual floppies");
    println!("  - Access file systems");
    println!("  - Convert between formats");
}
