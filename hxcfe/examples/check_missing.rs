use hxcfe::Hxcfe;
use hxcfe_sys::ImageFormat;

fn main() {
    let hxc = Hxcfe::get();
    let manager = hxc.loaders_manager().expect("Failed to create manager");
    
    let total = manager.nb_loaders();
    
    // Get all C library loaders
    let mut c_loaders = Vec::new();
    for i in 0..total {
        if let Some(loader) = manager.loader_for_id(i) {
            c_loaders.push(loader.name().to_string());
        }
    }
    
    // Get all Rust enum loaders
    let rust_loaders: Vec<String> = ImageFormat::all()
        .iter()
        .map(|f| f.loader_name().to_string())
        .collect();
    
    // Find what's in C but not in Rust
    let mut missing = Vec::new();
    for c_loader in &c_loaders {
        if !rust_loaders.iter().any(|r| r.eq_ignore_ascii_case(c_loader)) {
            missing.push(c_loader.clone());
        }
    }
    
    println!("Missing loaders (in C but not in Rust enum):");
    for (i, m) in missing.iter().enumerate() {
        println!("  {}: {}", i + 1, m);
    }
    println!("\nTotal missing: {}", missing.len());
}
