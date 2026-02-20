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
        fs_err::remove_dir_all(&base).unwrap();
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
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:include={}", include_dir.display());

    // Check if we should compile with cc crate (MSVC/MinGW/WASM) or with GNU make
    // For Windows (both MSVC and MinGW) and WASM, use cc crate
    if target.contains("wasm") {
        eprintln!("Building libhxcfe and libhxcadaptor for WebAssembly target");
        build_wasm(&base, &sources_dir, &libhxcadaptor_sources, &out_path, &target);
    } else if target.contains("windows") {
        let toolchain = if target.contains("msvc") { "MSVC" } else { "MinGW/GCC" };
        eprintln!("Building libhxcfe and libhxcadaptor with {} using cc crate", toolchain);

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
                || path_str.contains("untgz.c")  // Command-line program (has main())
                || path_str.contains("bmptoh.c")  // Convert tool with main(), needs sysexits.h (POSIX)
                || path_str.contains("programs")
            // CLI utilities
            {
                continue;
            }
            c_files.push(path.to_path_buf());
        }

        let hxcfe_count = c_files.len();
        eprintln!(
            "Found {} total C files to compile (libhxcfe + libhxcadaptor)",
            hxcfe_count
        );

        // Add USB C files if feature is enabled (check via cargo env var)
        let usb_enabled = env::var("CARGO_FEATURE_USB").is_ok();
        if usb_enabled {
            let libusbhxcfe_sources = base.parent().unwrap().join("libusbhxcfe/sources");
            for entry in WalkDir::new(&libusbhxcfe_sources)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                // Skip test files, examples, and platform-specific subdirectories
                if path_str.contains("test")
                    || path_str.contains("example")
                    || path_str.contains("/linux/")
                    || path_str.contains("/macosx/")
                    || path_str.contains("\\linux\\")
                    || path_str.contains("\\macosx\\")
                {
                    continue;
                }
                c_files.push(path.to_path_buf());
            }
            eprintln!("Added {} USB C files", c_files.len() - hxcfe_count);
        }

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
            .include(base.parent().unwrap().join("libusbhxcfe/sources")) // For usb_hxcfloppyemulator.h
            .include(base.parent().unwrap().join("build"))
            .include(sources_dir.join("thirdpartylibs/zlib"))
            .include(sources_dir.join("thirdpartylibs/zlib/contrib/minizip")) // For IMZ loader
            .include(sources_dir.join("thirdpartylibs/xdms")) // For DMS loader
            .include(sources_dir.join("thirdpartylibs/xdms/xdms-1.3.2/src")) // For DMS loader headers
            .include(sources_dir.join("thirdpartylibs/expat/lib")) // Updated path for expat 2.x
            .include(sources_dir.join("thirdpartylibs/FATIOlib"))
            .include(sources_dir.join("thirdpartylibs/adflib/Lib"))
            .include(sources_dir.join("thirdpartylibs/adflib/Lib/Win32"))
            .include(sources_dir.join("thirdpartylibs/lz4/lib"));

        // Add USB-specific includes for Windows
        if usb_enabled {
            build.include(base.parent().unwrap().join("libusbhxcfe/sources/win32")); // For ftd2xx.h
            eprintln!("USB feature enabled - added win32 include path for ftd2xx.h");
        }

        build
            .define("WIN32", None) // MSVC needs WIN32 defined
            // Z_SOLO removed: gzip support now enabled via Windows unistd.h shim (src/win_compat/unistd.h)
            // This enables ADZ, IMZ, and DMS loaders
            .define("XML_STATIC", None) // Use static linking for expat XML library
            .define("XML_GE", "1") // Enable general entities in expat (required by expat 2.5+)
            .define("XML_DTD", "1") // Enable DTD processing in expat
            .warnings(false);

        // For MinGW, add -static-libgcc to match original Makefile behavior
        // This statically links libgcc so executables don't require libgcc_s_seh-1.dll
        if target.contains("gnu") {
            build.flag("-static-libgcc");
            eprintln!("MinGW: Added -static-libgcc flag to statically link GCC runtime");
        }

        build.compile("hxcfe");

        // Note: libhxcadaptor is now compiled together with libhxcfe
        // No separate linking needed

        // Link Windows system libraries that might be needed
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");

        // Link FTDI USB library if USB feature is enabled
        // Note: The original Makefile builds a separate libusbhxcfe.dll for MinGW using:
        //   LDFLAGS += -static-libgcc ../sources/win32/libusbhxcfe.def
        // For Rust static linking, we currently support MSVC only with ftd2xx.lib
        // TODO: MinGW USB support would require either:
        //   1. Dynamic loading of ftd2xx.dll at runtime, or
        //   2. Finding a MinGW-compatible import library for ftd2xx
        if usb_enabled && target.contains("msvc") {
            let ftdi_lib_dir = base.parent().unwrap().join("libusbhxcfe/sources/win32");
            println!("cargo:rustc-link-search=native={}", ftdi_lib_dir.display());
            println!("cargo:rustc-link-lib=static=ftd2xx");
            eprintln!("USB feature enabled - linking ftd2xx library (MSVC)");
        } else if usb_enabled {
            eprintln!("WARNING: USB feature enabled but ftd2xx.lib is MSVC format only");
            eprintln!("         USB functionality will not be available with MinGW build");
            eprintln!("         (Original Makefile builds separate DLL; Rust uses static linking)");
        }

        let toolchain_name = if target.contains("msvc") { "MSVC" } else { "MinGW/GCC" };
        eprintln!("Successfully built libhxcfe and libhxcadaptor with {}", toolchain_name);
    } else {
        // Build libhxcadaptor first (dependency of libhxcfe)
        let libhxcadaptor_build_dir = base.parent().unwrap().join("libhxcadaptor/build");
        eprintln!("Building with GNU make for non-Windows platforms");
        eprintln!("Building libhxcadaptor...");
        let o = gnu_make()
            .arg("libhxcadaptor.a")
            .current_dir(&libhxcadaptor_build_dir)
            .output()
            .expect("failed to build libhxcadaptor");
        eprintln!("{}", String::from_utf8_lossy(&o.stdout));
        eprintln!("{}", String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success(), "libhxcadaptor build failed");
        
        // Build libhxcfe
        eprintln!("Building libhxcfe...");
        let o = gnu_make()
            .arg("libhxcfe.a")
            .current_dir(&build_dir)
            .output()
            .expect("failed to build libhxcfe");
        eprintln!("{}", String::from_utf8_lossy(&o.stdout));
        eprintln!("{}", String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success(), "libhxcfe build failed");
        
        // Add link search paths (both libraries are copied to build_dir by Makefile)
        println!("cargo:rustc-link-search=native={}", build_dir.display());
        println!("cargo:rustc-link-lib=static=hxcfe");
        println!("cargo:rustc-link-lib=static=hxcadaptor");
    }

    // Generate bindings
    let usb_enabled = env::var("CARGO_FEATURE_USB").is_ok();
    let mut builder = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("-I{}", libhxcadaptor_sources.display()))
        .header("wrapper.h")
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        .generate_cstr(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // Add USB support if feature is enabled
    if usb_enabled {
        let libusbhxcfe_sources = base.parent().unwrap().join("libusbhxcfe/sources");
        builder = builder
            .clang_arg(format!("-I{}", libusbhxcfe_sources.display()))
            .clang_arg("-DENABLE_USB");

        // Add win32 include for FTDI headers on Windows
        if cfg!(target_os = "windows") {
            let win32_sources = base.parent().unwrap().join("libusbhxcfe/sources/win32");
            builder = builder.clang_arg(format!("-I{}", win32_sources.display()));
        }

        eprintln!("USB feature enabled - added USB headers to bindgen");
    }

    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn build_wasm(
    base: &PathBuf,
    sources_dir: &PathBuf,
    libhxcadaptor_sources: &PathBuf,
    out_path: &PathBuf,
    target: &str,
) {
    // Collect all .c files from sources directory, excluding test files and examples
    let mut c_files = Vec::new();

    // Add libhxcadaptor C files
    for entry in WalkDir::new(libhxcadaptor_sources)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
    {
        c_files.push(entry.path().to_path_buf());
    }
    eprintln!("Found {} C files in libhxcadaptor", c_files.len());

    // Add libhxcfe C files (same exclusions as Windows build)
    for entry in WalkDir::new(sources_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();
        // Skip test files, examples, demos, command-line tools, USB sources (not supported in WASM)
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
            || path_str.contains("xdms.c")
            || path_str.contains("minizip.c")
            || path_str.contains("miniunz.c")
            || path_str.contains("untgz.c")
            || path_str.contains("bmptoh.c")
            || path_str.contains("programs")
            || path_str.contains("usb")  // Skip all USB-related files for WASM
            || path_str.contains("USB")
            || path_str.contains("ftdi")
            || path_str.contains("FTDI")
        {
            continue;
        }
        c_files.push(path.to_path_buf());
    }

    let hxcfe_count = c_files.len();
    eprintln!("Found {} total C files to compile for WASM (libhxcfe + libhxcadaptor)", hxcfe_count);

    // Build with cc crate for WASM target
    let mut build = cc::Build::new();
    for file in c_files {
        build.file(&file);
    }

    build
        .include(sources_dir)
        .include(libhxcadaptor_sources)
        .include(base.parent().unwrap().join("build"))
        .include(sources_dir.join("thirdpartylibs/zlib"))
        .include(sources_dir.join("thirdpartylibs/zlib/contrib/minizip"))
        .include(sources_dir.join("thirdpartylibs/xdms"))
        .include(sources_dir.join("thirdpartylibs/xdms/xdms-1.3.2/src"))
        .include(sources_dir.join("thirdpartylibs/expat/lib"))
        .include(sources_dir.join("thirdpartylibs/FATIOlib"))
        .include(sources_dir.join("thirdpartylibs/adflib/Lib"))
        .include(sources_dir.join("thirdpartylibs/lz4/lib"));

    // WASM-specific defines (matching Emscripten Makefile)
    build
        .define("XML_STATIC", None)
        .define("XML_GE", "1")
        .define("XML_DTD", "1")
        .warnings(false);

    // WASM optimization flags (Emscripten recommends -O2 or -O3 for production)
    if target.contains("wasm32") {
        build.flag("-O2");
        // Emscripten-specific: enable memory growth, disable pthreads (not needed for core lib)
        build.flag("-sALLOW_MEMORY_GROWTH=1");
        eprintln!("WASM: Using Emscripten optimization flags");
    }

    build.compile("hxcfe");

    eprintln!("Successfully built libhxcfe and libhxcadaptor for WebAssembly");
}

