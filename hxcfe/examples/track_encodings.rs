use hxcfe::TrackEncoding;

fn main() {
    println!("HxC Track Encodings:\n");

    // List all available track encodings
    for encoding in TrackEncoding::all() {
        println!("  {:2} - {}", *encoding as u32, encoding.encoding_name());
    }

    println!("\n=== Usage Examples ===\n");

    // Example 1: Getting encoding details
    let mfm = TrackEncoding::IsoibmMfm;
    println!("ISO IBM MFM Encoding:");
    println!("  Name: {}", mfm.encoding_name());
    println!("  ID: {}", mfm as u32);

    // Example 2: Parsing from string
    println!("\nParse from string:");
    if let Some(encoding) = TrackEncoding::from_str("AMIGA_MFM_ENCODING") {
        println!("  Found: {}", encoding);
    }

    // Example 3: Using Display trait
    println!("\nDisplay trait:");
    println!("  Amiga MFM: {}", TrackEncoding::AmigaMfm);
    println!("  Apple II GCR: {}", TrackEncoding::AppleiiGcr1);
    println!("  C64 GCR: {}", TrackEncoding::C64Gcr);

    // Example 4: Converting from u32
    println!("\nFrom u32 conversion:");
    if let Some(encoding) = TrackEncoding::from_u32(0) {
        println!("  Encoding 0 = {}", encoding);
    }

    println!("\nTotal encodings: {}", TrackEncoding::all().len());
}
