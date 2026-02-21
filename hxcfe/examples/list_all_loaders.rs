use hxcfe::Hxcfe;

fn main() {
    let hxc = Hxcfe::get();
    let manager = hxc.loaders_manager().expect("Failed to create manager");

    let total = manager.nb_loaders();
    println!("Total loaders: {}\n", total);

    for i in 0..total {
        if let Some(loader) = manager.loader_for_id(i) {
            println!("{}: {}", i, loader.name());
        }
    }
}
