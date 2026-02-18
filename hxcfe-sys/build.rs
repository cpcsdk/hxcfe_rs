use make_cmd::gnu_make;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
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
    let target = env::var("TARGET").unwrap();

    let include_dir = dunce::canonicalize(base.join("sources")).unwrap();
    let build_dir = dunce::canonicalize(base.join("build")).unwrap();
    let sources_dir = base.join("sources");
    let libhxcadaptor_sources = base.parent().unwrap().join("libhxcadaptor/sources");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    //  generate cargo information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:include={}", include_dir.display());

    // Check if we should compile with MSVC or with make  
    if target.contains("windows-msvc") {
        eprintln!("Building libhxcfe with MSVC using cc crate");
        
        // Collect all .c files from sources directory, excluding test files and examples
        let mut c_files = Vec::new();
        for entry in WalkDir::new(&sources_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "c"))
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            // Skip test files, examples, demos, command-line tools, Generic templates,
            // Windows GUI files (adfvolinfo.c, nt4_dev.c), fuzzing tests, xmlwf utility,
            // FATIOlib Main.c (test program), xdms (Unix-specific), zlib gz* (gzip, needs unistd.h),
            // adz_loader (uses gzFile), programs (CLI utilities), contrib (examples)
            if path_str.contains("test") 
                || path_str.contains("example")
                || path_str.contains("Demo")
                || path_str.contains("HxCFloppyEmulator_cmdline")
                || path_str.contains("Generic")
                || path_str.contains("adfvolinfo.c")
                || path_str.contains("nt4_dev.c")
                || path_str.contains("fuzz")
                || path_str.contains("xmlwf")
                || path_str.contains("gennmtab")
                || path_str.contains("FATIOlib\\Main.c")
                || path_str.contains("FATIOlib/Main.c")
                || path_str.contains("xdms")
                || (path_str.contains("zlib") && path_str.contains("\\gz"))
                || (path_str.contains("zlib") && path_str.contains("/gz"))
                || path_str.contains("adz_loader")  // Uses gzFile from gzip API
                || path_str.contains("imz_loader")  // Uses minizip which needs contrib files
                || path_str.contains("programs")  // CLI utilities
                || path_str.contains("contrib")  // Example/utility code (including minizip)
            {
                continue;
            }
            c_files.push(path.to_path_buf());
        }

        eprintln!("Found {} C files to compile", c_files.len());
        
        // Add stub implementations for excluded loaders
        c_files.push(PathBuf::from("src/loader_stubs.c"));
        
        // Build with cc crate
        let mut build = cc::Build::new();
        for file in c_files {
            build.file(&file);
        }
        
        build
            .include(&sources_dir)
            .include(&libhxcadaptor_sources)
            .include(base.parent().unwrap().join("build"))
            .include(sources_dir.join("thirdpartylibs/zlib"))
            .include(sources_dir.join("thirdpartylibs/expat/expat-2.5.0/lib"))
            .include(sources_dir.join("thirdpartylibs/FATIOlib"))
            .include(sources_dir.join("thirdpartylibs/adflib/Lib"))
            .include(sources_dir.join("thirdpartylibs/adflib/Lib/Win32"))
            .include(sources_dir.join("thirdpartylibs/lz4/lib"))
            .define("WIN32", None)  // MSVC needs WIN32 defined
            .define("Z_SOLO", None)  // Exclude gzip support from zlib (requires unistd.h)
            .define("XML_STATIC", None)  // Use static linking for expat XML library
            .warnings(false)
            .compile("hxcfe");
        
        // Link hxcadaptor (compiled by hxcadaptor-sys crate)
        println!("cargo:rustc-link-lib=static=hxcadaptor");
        
        // Link Windows system libraries that might be needed
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        
        eprintln!("Successfully built libhxcfe with MSVC");
    } else {
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
