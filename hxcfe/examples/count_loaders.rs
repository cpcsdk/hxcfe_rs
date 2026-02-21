use hxcfe::Hxcfe;

fn main() {
    let hxc = Hxcfe::get();
    let manager = hxc.loaders_manager().expect("Failed to create manager");

    let total = manager.nb_loaders();
    println!("Total loaders: {}", total);

    // Count by prefix to identify XML loaders
    let mut xml_count = 0;
    for i in 0..total {
        if let Some(loader) = manager.loader_for_id(i) {
            if i < 10 {
                println!("{}: {}", i, loader.name());
            }
            if loader.name().starts_with("GENERIC_XML") {
                xml_count += 1;
            }
        }
    }

    println!("\nXML loaders: {}", xml_count);
    println!("Non-XML loaders: {}", total - xml_count);
}
