use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Files to exclude from compilation (command-line tools, tests, examples)
const EXCLUDED_FILES: &[&str] = &[
    "test", "example", "Demo", "HxCFloppyEmulator_cmdline", "Generic",
    "adfvolinfo.c", "nt4_dev.c", "fuzz", "xmlwf", "gennmtab",
    "FATIOlib\\Main.c", "FATIOlib/Main.c",
    "xdms.c", "minizip.c", "miniunz.c", "untgz.c", "bmptoh.c",
    "programs",
];

/// Checks if a path should be excluded from compilation
fn should_exclude(path_str: &str, additional_exclusions: &[&str]) -> bool {
    EXCLUDED_FILES.iter().any(|&e| path_str.contains(e)) ||
    additional_exclusions.iter().any(|&e| path_str.contains(e))
}

/// Collects C files from a directory, applying exclusion rules
fn collect_c_files(dir: &PathBuf, exclusions: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
        .filter(|e| !should_exclude(&e.path().to_string_lossy(), exclusions))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Configures common includes for the build
fn add_common_includes(build: &mut cc::Build, sources_dir: &PathBuf, base: &PathBuf, libhxcadaptor_sources: &PathBuf) {
    build
        .include(sources_dir)
        .include(libhxcadaptor_sources)
        .include(base.parent().unwrap().join("libusbhxcfe/sources"))
        .include(base.parent().unwrap().join("build"))
        .include(sources_dir.join("thirdpartylibs/zlib"))
        .include(sources_dir.join("thirdpartylibs/zlib/contrib/minizip"))
        .include(sources_dir.join("thirdpartylibs/xdms"))
        .include(sources_dir.join("thirdpartylibs/xdms/xdms-1.3.2/src"))
        .include(sources_dir.join("thirdpartylibs/expat/lib"))
        .include(sources_dir.join("thirdpartylibs/FATIOlib"))
        .include(sources_dir.join("thirdpartylibs/adflib/Lib"))
        .include(sources_dir.join("thirdpartylibs/lz4/lib"));
}

/// Configures common defines for the build
fn add_common_defines(build: &mut cc::Build) {
    build
        .define("XML_STATIC", None)
        .define("XML_GE", "1")
        .define("XML_DTD", "1")
        .warnings(false);
}

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
    let sources_dir = base.join("sources");
    let libhxcadaptor_sources = base.parent().unwrap().join("libhxcadaptor/sources");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    //  generate cargo information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:include={}", include_dir.display());

    // Build with cc crate for all platforms (unified build system)
    if target.contains("wasm") {
        eprintln!("Building for WebAssembly target");
        build_wasm(&base, &sources_dir, &libhxcadaptor_sources, &out_path, &target);
    } else if target.contains("windows") {
        build_windows(&base, &sources_dir, &libhxcadaptor_sources, &target);
    } else {
        build_unix(&base, &sources_dir, &libhxcadaptor_sources);
    }

    // Generate bindings
    generate_bindings(&base, &include_dir, &libhxcadaptor_sources, &out_path);
}

fn build_windows(base: &PathBuf, sources_dir: &PathBuf, libhxcadaptor_sources: &PathBuf, target: &str) {
    let toolchain = if target.contains("msvc") { "MSVC" } else { "MinGW/GCC" };
    eprintln!("Building with {} using cc crate", toolchain);

    let usb_enabled = env::var("CARGO_FEATURE_USB").is_ok();
    
    // Collect C files
    let mut c_files = collect_c_files(libhxcadaptor_sources, &[]);
    eprintln!("Found {} C files in libhxcadaptor", c_files.len());

    c_files.extend(collect_c_files(sources_dir, &[]));
    let total_count = c_files.len();
    
    // Add USB files if enabled
    if usb_enabled {
        let libusbhxcfe_sources = base.parent().unwrap().join("libusbhxcfe/sources");
        let usb_files = collect_c_files(&libusbhxcfe_sources, &["/linux/", "/macosx/", "\\linux\\", "\\macosx\\"]);
        eprintln!("Added {} USB C files", usb_files.len());
        c_files.extend(usb_files);
    }
    
    eprintln!("Compiling {} total C files", total_count);

    // Build
    let mut build = cc::Build::new();
    for file in c_files {
        build.file(&file);
    }

    // Add Windows compatibility headers
    build.include("src/win_compat");
    
    add_common_includes(&mut build, sources_dir, base, libhxcadaptor_sources);

    if usb_enabled {
        build.include(base.parent().unwrap().join("libusbhxcfe/sources/win32"));
        eprintln!("USB feature enabled - added win32 include path");
    }

    add_common_defines(&mut build);
    build.define("WIN32", None);

    // MinGW: add -static-libgcc
    if target.contains("gnu") {
        build.flag("-static-libgcc");
        eprintln!("MinGW: Added -static-libgcc flag");
    }

    build.compile("hxcfe");

    // Link Windows system libraries
    println!("cargo:rustc-link-lib=dylib=advapi32");
    println!("cargo:rustc-link-lib=dylib=ws2_32");

    // Link FTDI USB library if USB feature is enabled (MSVC only)
    if usb_enabled && target.contains("msvc") {
        let ftdi_lib_dir = base.parent().unwrap().join("libusbhxcfe/sources/win32");
        println!("cargo:rustc-link-search=native={}", ftdi_lib_dir.display());
        println!("cargo:rustc-link-lib=static=ftd2xx");
        eprintln!("USB feature enabled - linking ftd2xx library (MSVC)");
    } else if usb_enabled {
        eprintln!("WARNING: USB feature enabled but ftd2xx.lib is MSVC format only");
        eprintln!("         USB functionality will not be available with MinGW build");
    }

    eprintln!("Successfully built with {}", toolchain);
}

fn build_unix(base: &PathBuf, sources_dir: &PathBuf, libhxcadaptor_sources: &PathBuf) {
    eprintln!("Building for Unix platforms");

    let usb_enabled = env::var("CARGO_FEATURE_USB").is_ok();
    
    // Collect C files (exclude Windows-specific files)
    let mut c_files = collect_c_files(libhxcadaptor_sources, &[]);
    eprintln!("Found {} C files in libhxcadaptor", c_files.len());

    c_files.extend(collect_c_files(sources_dir, &["iowin32.c", "/Win32/", "\\Win32\\"]));
    let total_count = c_files.len();
    
    // Add USB files if enabled
    if usb_enabled {
        let libusbhxcfe_sources = base.parent().unwrap().join("libusbhxcfe/sources");
        let usb_files = collect_c_files(&libusbhxcfe_sources, &["/win32/", "/macosx/", "\\win32\\", "\\macosx\\"]);
        eprintln!("Added {} USB C files", usb_files.len());
        c_files.extend(usb_files);
    }
    
    eprintln!("Compiling {} total C files", total_count);

    // Build
    let mut build = cc::Build::new();
    for file in c_files {
        build.file(&file);
    }

    add_common_includes(&mut build, sources_dir, base, libhxcadaptor_sources);

    if usb_enabled {
        build.include(base.parent().unwrap().join("libusbhxcfe/sources/linux"));
        eprintln!("USB feature enabled - added linux include path");
    }

    add_common_defines(&mut build);
    build.define("XML_DEV_URANDOM", None);  // Use /dev/urandom for entropy on Linux

    build.compile("hxcfe");

    // Link USB library if USB feature is enabled
    if usb_enabled {
        println!("cargo:rustc-link-lib=dylib=usb-1.0");
        eprintln!("USB feature enabled - linking libusb-1.0");
    }

    eprintln!("Successfully built for Unix platforms");
}

fn generate_bindings(base: &PathBuf, include_dir: &PathBuf, libhxcadaptor_sources: &PathBuf, out_path: &PathBuf) {
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

fn build_wasm(base: &PathBuf, sources_dir: &PathBuf, libhxcadaptor_sources: &PathBuf, _out_path: &PathBuf, target: &str) {
    let is_emscripten = target.contains("emscripten");
    
    // The C library code requires standard C library headers (stdlib.h, string.h, etc.)
    // which are only available with Emscripten's libc implementation.
    // The original project's Makefiles only support Emscripten for WASM builds.
    if !is_emscripten {
        eprintln!("\n==========================================================");
        eprintln!("WARNING: Building for '{}' without Emscripten", target);
        eprintln!("==========================================================");
        eprintln!("The C library requires standard C headers (stdlib.h, string.h, etc.)");
        eprintln!("which are not available in {}.", target);
        eprintln!("");
        eprintln!("The original HxCFloppyEmulator project only supports Emscripten for WASM.");
        eprintln!("Recommended target: wasm32-unknown-emscripten");
        eprintln!("");
        eprintln!("To install: rustup target add wasm32-unknown-emscripten");
        eprintln!("To build:   cargo build --target wasm32-unknown-emscripten");
        eprintln!("==========================================================\n");
        
        panic!("Unsupported WASM target: {}. Use wasm32-unknown-emscripten instead.", target);
    }
    
    // Collect C files
    let mut c_files = collect_c_files(libhxcadaptor_sources, &[]);
    eprintln!("Found {} C files in libhxcadaptor", c_files.len());

    // Add libhxcfe C files - skip USB support entirely for WASM
    let wasm_exclusions = &["usb", "USB", "ftdi", "FTDI"];
    c_files.extend(collect_c_files(sources_dir, wasm_exclusions));
    
    eprintln!("Compiling {} total C files (libhxcfe + libhxcadaptor)", c_files.len());

    // Build
    let mut build = cc::Build::new();
    for file in c_files {
        build.file(&file);
    }

    add_common_includes(&mut build, sources_dir, base, libhxcadaptor_sources);
    add_common_defines(&mut build);

    // WASM optimization flags
    if target.contains("wasm32") {
        build.flag("-O2");
        build.flag("-sALLOW_MEMORY_GROWTH=1");
        eprintln!("Using Emscripten optimization flags");
    }

    build.compile("hxcfe");

    eprintln!("Successfully built for WebAssembly (Emscripten)");
}

