use hxcfe::{Hxcfe, InterfaceMode};

fn main() {
    println!("HxC Floppy Interface Modes:\n");

    // List all available interface modes
    for mode in InterfaceMode::all() {
        println!(
            "  {:2} - {:<30} {}",
            *mode as i32,
            mode.mode_name(),
            mode.description()
        );
    }

    println!("\n=== Usage Examples ===\n");

    // Example 1: Getting mode details
    let pc_dd = InterfaceMode::IbmpcDd;
    println!("PC DD Mode:");
    println!("  Name: {}", pc_dd.mode_name());
    println!("  Description: {}", pc_dd.description());
    println!("  ID: {}", pc_dd as i32);

    // Example 2: Parsing from string
    println!("\nParse from string:");
    if let Some(mode) = InterfaceMode::from_str("AMIGA_DD_FLOPPYMODE") {
        println!("  Found: {} - {}", mode, mode.description());
    }

    // Example 3: Using Display trait
    println!("\nDisplay trait:");
    println!("  Amstrad CPC: {}", InterfaceMode::CpcDd);
    println!("  Atari ST DD: {}", InterfaceMode::AtaristDd);

    // Example 4: Using with Hxcfe API
    println!("\nUsing with Hxcfe API:");
    let hxcfe = Hxcfe::get();
    let mode = InterfaceMode::IbmpcHd;
    if let Some(interface) = hxcfe.floppy_interface(mode) {
        println!("  Mode: {}", interface.name());
        println!("  Description: {}", interface.description());
    }
}
