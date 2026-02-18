use make_cmd::gnu_make;
use std::env;
use std::path::PathBuf;

fn main() {
    // setup paths of interest
    let original_base: PathBuf = "vendor/HxCFloppyEmulator/".into();
    assert!(original_base.exists());
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    // clone source code in output as it is the sole place where we can build
    let base = out_path.join("hxccode");
    if base.exists() {
        std::fs::remove_dir_all(&base).unwrap();
    }
    copy_dir::copy_dir(&original_base, &base).unwrap();

    let base = base.join("libhxcadaptor/");

    let include_dir = dunce::canonicalize(base.join("sources")).unwrap();
    let build_dir = dunce::canonicalize(base.join("build")).unwrap();
    let sources_dir = base.join("sources");
    let libhxcfe_sources = base.parent().unwrap().join("libhxcfe/sources");
    let libusbhxcfe_sources = base.parent().unwrap().join("libusbhxcfe/sources");

    //  generate cargo information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:include={}", include_dir.display());

    // Check if we should compile with MSVC (using cc crate) or with make
    if target.contains("windows-msvc") {
        eprintln!("Building libhxcadaptor with MSVC using cc crate");
        
        // Compile C files with cc crate for MSVC compatibility
        cc::Build::new()
            .file(sources_dir.join("libhxcadaptor.c"))
            .file(sources_dir.join("fs.c"))
            .file(sources_dir.join("network.c"))
            .include(&include_dir)
            .include(&libhxcfe_sources)
            .include(&libusbhxcfe_sources)
            .include(base.parent().unwrap().join("build"))
            .define("WIN32", None)  // MSVC needs WIN32 defined
            .warnings(false)
            .compile("hxcadaptor");
        
        eprintln!("Successfully built libhxcadaptor with MSVC");
        // cc crate handles the linking automatically, no need for rustc-link-search
    } else {
        println!("cargo:rustc-link-search=native={}", build_dir.display());
        println!("cargo:rustc-link-lib=hxcadaptor");

        eprintln!("Building libhxcadaptor with make");
        let o = gnu_make()
            .current_dir(&build_dir)
            .output()
            .expect("failed to build libhxcadaptor");
        eprintln!("{}", String::from_utf8_lossy(&o.stdout));
        eprintln!("{}", String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success());

        if cfg!(target_os = "windows") {
            eprintln!("Build the windows library");
            std::fs::copy(build_dir.join("libhxcadaptor.a"), build_dir.join("hxcadaptor.lib")).unwrap();
        }
    }

    eprintln!("Generate bindings");
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("--target={}", target))
        .header("wrapper.h")
        .generate_cstr(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
