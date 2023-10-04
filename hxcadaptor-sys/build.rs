use make_cmd::gnu_make;
use std::env;
use std::path::PathBuf;

fn main() {
    // setup paths of interest
    let base: PathBuf = "../vendor/HxCFloppyEmulator/libhxcadaptor/".into();
    assert!(base.exists());
    let include_dir = dunce::canonicalize(base.join("sources")).unwrap();
    let build_dir = dunce::canonicalize(base.join("build")).unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    //  generate cargo information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:include={}", include_dir.display());
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-lib=hxcadaptor");

    // Really build the library
    let o = gnu_make()
        .current_dir(build_dir)
        .output()
        .expect("failed to build libhxcadaptor");
    eprintln!("{}", std::str::from_utf8(&o.stdout).unwrap());
    eprintln!("{}", std::str::from_utf8(&o.stderr).unwrap());
    assert!(o.status.success());

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_dir.display()))
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
