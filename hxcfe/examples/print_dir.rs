use hxcfe::{FileSystemManager, Hxcfe};
const DSK_FNAME: &'static str = "tests/ECOLE_BUISSONNIERE_(OVERLANDERS).DSK";


fn display_dir(fsmngr: &FileSystemManager, folder: &str, level: u32) -> i32  {
    if let Ok(dir_handle) = dbg!(fsmngr.open_dir(folder)) {
		loop {
			let mut dir = false;
			if let Ok(dirent) = dir_handle.read() {

				for _ in 0..level {
					print!("    ")
				}
				if (dirent.is_dir()) {
					print!(">");
					dir = true;
				} else {
					print!(" ");
				}
				println!("{} <{}>",dirent.entry_name(),dirent.size());

				if dir {
					let mut fullpath = folder.to_owned();
					if !fullpath.ends_with("/") {
						fullpath.push('/');
					}
					fullpath.push_str(dirent.entry_name());
					if dirent.entry_name() != "." && dirent.entry_name() != ".." {
						if(display_dir(fsmngr, &fullpath,level+1)<0)
						{
							dir_handle.close();
							return 0;
						}
					}
				}
			} else {
				return 0;
			}
		}

    }
	return 0;

}

fn main() {
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
    display_dir(&fsmngr, "/", 0);
}