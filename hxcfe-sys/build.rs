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
        eprintln!("Building libhxcfe and libhxcadaptor with MSVC using cc crate");
        
        // Collect all .c files from sources directory, excluding test files and examples
        let mut c_files = Vec::new();
        
        // Add libhxcadaptor C files
        for entry in WalkDir::new(&libhxcadaptor_sources)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
        {
            c_files.push(entry.path().to_path_buf());
        }
        eprintln!("Found {} C files in libhxcadaptor", c_files.len());
        
        // Add libhxcfe C files
        for entry in WalkDir::new(&sources_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            // Skip test files, examples, demos, command-line tools, Generic templates,
            // Windows GUI files (adfvolinfo.c, nt4_dev.c), fuzzing tests, xmlwf utility,
            // FATIOlib Main.c (test program), command-line programs with main(),
            // bmptoh.c (convert tool with main() - needs sysexits.h)
            // NOTE: Now including minizip, xdms, imz_loader, dms_loader (enabled via unistd.h shim)
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
                || path_str.contains("xdms.c")  // Command-line program (has main()), not needed for library
                || path_str.contains("minizip.c")  // Command-line program (has main())
                || path_str.contains("miniunz.c")  // Command-line program (has main())
                || path_str.contains("bmptoh.c")  // Convert tool with main(), needs sysexits.h (POSIX)
                || path_str.contains("programs")  // CLI utilities
            {
                continue;
            }
            c_files.push(path.to_path_buf());
        }

        eprintln!("Found {} total C files to compile (libhxcfe + libhxcadaptor)", c_files.len());
        
        // All loaders now enabled via Windows unistd.h shim (src/win_compat/unistd.h)
        // ✅ ADZ: gzip-compressed disk images (zlib with gzip support)
        // ✅ IMZ: ZIP-compressed disk images (minizip)  
        // ✅ DMS: Amiga DiskMasher compressed (xdms library)
        // No stubs needed!
        
        // Build with cc crate
        let mut build = cc::Build::new();
        for file in c_files {
            build.file(&file);
        }
        
        // On Windows, add our compatibility headers BEFORE standard includes
        // This allows our unistd.h shim to be found
        build.include("src/win_compat");
        
        build
            .include(&sources_dir)
            .include(&libhxcadaptor_sources)
            .include(base.parent().unwrap().join("libusbhxcfe/sources"))  // For usb_hxcfloppyemulator.h
            .include(base.parent().unwrap().join("build"))
            .include(sources_dir.join("thirdpartylibs/zlib"))
            .include(sources_dir.join("thirdpartylibs/zlib/contrib/minizip"))  // For IMZ loader
            .include(sources_dir.join("thirdpartylibs/xdms"))  // For DMS loader
            .include(sources_dir.join("thirdpartylibs/xdms/xdms-1.3.2/src"))  // For DMS loader headers
            .include(sources_dir.join("thirdpartylibs/expat/lib"))  // Updated path for expat 2.x
            .include(sources_dir.join("thirdpartylibs/FATIOlib"))
            .include(sources_dir.join("thirdpartylibs/adflib/Lib"))
            .include(sources_dir.join("thirdpartylibs/adflib/Lib/Win32"))
            .include(sources_dir.join("thirdpartylibs/lz4/lib"))
            .define("WIN32", None)  // MSVC needs WIN32 defined
            // Z_SOLO removed: gzip support now enabled via Windows unistd.h shim (src/win_compat/unistd.h)
            // This enables ADZ, IMZ, and DMS loaders
            .define("XML_STATIC", None)  // Use static linking for expat XML library
            .define("XML_GE", "1")  // Enable general entities in expat (required by expat 2.5+)
            .define("XML_DTD", "1")  // Enable DTD processing in expat
            .warnings(false)
            .compile("hxcfe");
        
        // Note: libhxcadaptor is now compiled together with libhxcfe
        // No separate linking needed
        
        // Link Windows system libraries that might be needed
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        
        eprintln!("Successfully built libhxcfe and libhxcadaptor with MSVC");
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
    let builder = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("-I{}", libhxcadaptor_sources.display()))
        .header("wrapper.h")
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        .generate_cstr(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    
    // Add USB support if feature is enabled
    #[cfg(feature = "usb")]
    {
        let libusbhxcfe_sources = base.parent().unwrap().join("libusbhxcfe/sources");
        builder = builder
            .clang_arg(format!("-I{}", libusbhxcfe_sources.display()))
            .clang_arg("-DENABLE_USB");
    }
    
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}