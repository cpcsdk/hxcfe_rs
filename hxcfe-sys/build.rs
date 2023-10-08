use make_cmd::gnu_make;
use std::env;
use std::path::PathBuf;

fn main() {
    // checkup

    // setup paths of interest
    let original_base: PathBuf = "vendor/HxCFloppyEmulator/".into();
    assert!(original_base.exists());
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());


    // clone source code in output as it is the sole place where we can build
    let base = out_path.join("hxccode");
    if base.exists() {
        std::fs::remove_dir_all(&base).unwrap();
    }
    copy_dir::copy_dir(&original_base, &base).unwrap();
    let base = base.join("libhxcfe");

    let include_dir = dunce::canonicalize(base.join("sources")).unwrap();
    let build_dir = dunce::canonicalize(base.join("build")).unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    //  generate cargo information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:include={}", include_dir.display());
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-lib=static=hxcfe");
    println!("cargo:rustc-link-lib=static=hxcadaptor");

    eprintln!("Really build the library");
    let o = gnu_make()
        .arg("libhxcfe.a")
        .current_dir(&build_dir)
        .output()
        .expect("failed to build libhxcfe");
    eprintln!("{}", String::from_utf8_lossy(&o.stdout));
    eprintln!("{}", String::from_utf8_lossy(&o.stderr));
    assert!(o.status.success());

    if cfg!(target_os = "windows") {
        eprintln!("Create windows file");
        std::fs::copy(build_dir.join("libhxcfe.a"), build_dir.join("hxcfe.lib")).unwrap();
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_dir.display()))
        .header("wrapper.h")
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        .generate_cstr(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
