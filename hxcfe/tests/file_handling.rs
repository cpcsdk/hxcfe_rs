use hxcfe::{Hxcfe};
const DSK_FNAME: &'static str = "tests/ECOLE_BUISSONNIERE_(OVERLANDERS).DSK";

#[test]
fn load_dsk() {
    let hxcfe = Hxcfe::get();
    let img = hxcfe
        .load(DSK_FNAME)
        .expect(&format!("Unable to read {}", DSK_FNAME));

    let interface = img.interface_mode();
    println!("Interface mode {} {}", interface.name(), interface.desc());
    println!("Size: {}", img.size());
    println!("Nb sectors: {}", img.nb_sectors());
    println!("Nb sides: {}", img.nb_sides());

    let fsmngr = hxcfe.file_system_manager().unwrap();
    fsmngr.select_fs(0);
    fsmngr.mount(&img);
}

#[test]
#[should_panic]
fn load_missing_dsk() {
    let hxcfe = Hxcfe::get();
    let _res = dbg!(hxcfe.load("missing.dsk"));
}
