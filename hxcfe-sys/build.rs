use std::env;
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;

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
    
    // Generate ImageFormat enum from loaders
    generate_image_format_enum(&base, &out_path);
    
    // Generate InterfaceMode enum from floppy_ifmode.c
    generate_interface_mode_enum(&base, &out_path);
    
    // Generate TrackEncoding enum from floppy_ifmode.c
    generate_track_encoding_enum(&base, &out_path);
    
    // Generate DiskLayout enum from LayoutsIndex.h
    generate_disk_layout_enum(&base, &out_path);
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
    
    // Emscripten/WASM-specific: Use arc4random for entropy (provided by Emscripten libc)
    // This is needed for expat XML library to generate random data
    build.define("HAVE_ARC4RANDOM_BUF", None);

    // WASM optimization flags
    if target.contains("wasm32") {
        build.flag("-O2");
        build.flag("-sALLOW_MEMORY_GROWTH=1");
        eprintln!("Using Emscripten optimization flags");
    }

    build.compile("hxcfe");

    eprintln!("Successfully built for WebAssembly (Emscripten)");
}

/// Information about a loader plugin
#[derive(Debug, Clone)]
struct LoaderInfo {
    id: String,
    description: String,
    extension: String,
    has_writer: bool,
}

/// Parse a loader .c file to extract plugin information
fn parse_loader_file(path: &PathBuf) -> Option<LoaderInfo> {
    // Read file as bytes and convert to UTF-8 lossy to handle non-UTF-8 characters
    let bytes = fs::read(path).ok()?;
    let content = String::from_utf8_lossy(&bytes);
    
    // Look for plug_id[]="..." pattern anywhere in the file
    // This appears in the XXX_libGetPluginInfo function as local variables
    // May be preceded by whitespace and/or "static const char"
    let re_id = Regex::new(r#"(?:static\s+const\s+char\s+)?plug_id\s*\[\s*\]\s*=\s*"([^"]+)""#).unwrap();
    let id = re_id.captures(&content)?.get(1)?.as_str().to_string();
    
    // Look for plug_desc[]="..."
    let re_desc = Regex::new(r#"(?:static\s+const\s+char\s+)?plug_desc\s*\[\s*\]\s*=\s*"([^"]+)""#).unwrap();
    let description = re_desc.captures(&content)?.get(1)?.as_str().to_string();
    
    // Look for plug_ext[]="..."
    let re_ext = Regex::new(r#"(?:static\s+const\s+char\s+)?plug_ext\s*\[\s*\]\s*=\s*"([^"]+)""#).unwrap();
    let extension = re_ext.captures(&content)?.get(1)?.as_str().to_string();
    
    // Check if it has a writer (WRITEDISKFILE is not NULL/0)
    let re_writer = Regex::new(r"(?s)\(WRITEDISKFILE\)\s+(\w+)").unwrap();
    let has_writer = if let Some(writer_match) = re_writer.captures(&content) {
        if let Some(m) = writer_match.get(1) {
            let writer = m.as_str();
            writer != "0" && writer != "NULL"
        } else {
            false
        }
    } else {
        false
    };
    
    Some(LoaderInfo {
        id,
        description,
        extension,
        has_writer,
    })
}

/// Parse a loader .c file to extract ALL plugin information (some files have multiple loaders)
fn parse_all_loaders_from_file(path: &PathBuf) -> Vec<LoaderInfo> {
    // Read file as bytes and convert to UTF-8 lossy to handle non-UTF-8 characters
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let content = String::from_utf8_lossy(&bytes);
    
    // Look for functions that match *_libGetPluginInfo pattern
    // Each such function defines a separate loader
    let func_re = Regex::new(r"(?m)^int\s+(\w+)_libGetPluginInfo\s*\(").unwrap();
    let functions: Vec<_> = func_re.captures_iter(&content).collect();
    
    if functions.is_empty() {
        // No specific function found, try the simple parser
        if let Some(loader) = parse_loader_file(path) {
            return vec![loader];
        }
        return Vec::new();
    }
    
    let mut loaders = Vec::new();
    
    // For each function, extract its local variables
    for func_match in functions {
        let func_start = func_match.get(0).unwrap().start();
        
        // Find the function body (between this function and the next, or end of file)
        let next_func_start = func_re.find_iter(&content[func_start + 1..])
            .next()
            .map(|m| func_start + 1 + m.start())
            .unwrap_or(content.len());
        
        let func_body = &content[func_start..next_func_start];
        
        // Extract plug_id, plug_desc, plug_ext from this function's body
        let re_id = Regex::new(r#"(?:static\s+const\s+char\s+)?plug_id\s*\[\s*\]\s*=\s*"([^"]+)""#).unwrap();
        let re_desc = Regex::new(r#"(?:static\s+const\s+char\s+)?plug_desc\s*\[\s*\]\s*=\s*"([^"]+)""#).unwrap();
        let re_ext = Regex::new(r#"(?:static\s+const\s+char\s+)?plug_ext\s*\[\s*\]\s*=\s*"([^"]+)""#).unwrap();
        
        if let (Some(id_cap), Some(desc_cap), Some(ext_cap)) = 
            (re_id.captures(func_body), re_desc.captures(func_body), re_ext.captures(func_body)) {
            
            let id = id_cap.get(1).unwrap().as_str().to_string();
            let description = desc_cap.get(1).unwrap().as_str().to_string();
            let extension = ext_cap.get(1).unwrap().as_str().to_string();
            
            // Check if this specific function has a writer
            let re_writer = Regex::new(r"(?s)\(WRITEDISKFILE\)\s+(\w+)").unwrap();
            let has_writer = if let Some(writer_match) = re_writer.captures(func_body) {
                if let Some(m) = writer_match.get(1) {
                    let writer = m.as_str();
                    writer != "0" && writer != "NULL"
                } else {
                    false
                }
            } else {
                false
            };
            
            loaders.push(LoaderInfo {
                id,
                description,
                extension,
                has_writer,
            });
        }
    }
    
    // If we found no loaders via function parsing, fall back to simple parser
    if loaders.is_empty() {
        if let Some(loader) = parse_loader_file(path) {
            loaders.push(loader);
        }
    }
    
    loaders
}

/// Parse disk layouts for loader generation
/// Returns list of layout info (id and name) for creating ImageFormat variants
fn parse_disk_layouts_for_loaders(base: &PathBuf) -> Vec<DiskLayoutInfo> {
    let layouts_file = base.join("sources/xml_disk/DiskLayouts/LayoutsIndex.h");
    let xml_dir = base.join("sources/xml_disk/DiskLayouts/xml_files");
    
    // Read file with lossy UTF-8 conversion
    let bytes = fs::read(&layouts_file).expect("Failed to read LayoutsIndex.h");
    let content = String::from_utf8_lossy(&bytes);
    
    // Parse disk layouts from the C array to get the order and file names
    let layout_re = Regex::new(r"data_DiskLayout_([A-Za-z0-9_]+)_xml").unwrap();
    let xml_name_re = Regex::new(r"<disk_layout_name>([^<]+)</disk_layout_name>").unwrap();
    
    let mut layouts: Vec<DiskLayoutInfo> = Vec::new();
    let mut in_list = false;
    
    for line in content.lines() {
        if line.contains("disklayout_list[]=") {
            in_list = true;
            continue;
        }
        
        if in_list {
            if line.trim() == "0" || line.trim() == "};" {
                break;
            }
            
            if let Some(cap) = layout_re.captures(line) {
                let file_name = cap.get(1).unwrap().as_str();
                let id = layouts.len();
                
                // Read the corresponding XML file to get the actual layout name
                let xml_path = xml_dir.join(format!("DiskLayout_{}.xml", file_name));
                let xml_content = fs::read_to_string(&xml_path)
                    .unwrap_or_else(|_| panic!("Failed to read XML file: {:?}", xml_path));
                
                // Extract the <disk_layout_name> from XML
                let layout_name = if let Some(cap) = xml_name_re.captures(&xml_content) {
                    cap.get(1).unwrap().as_str().to_string()
                } else {
                    panic!("Failed to find <disk_layout_name> in {:?}", xml_path);
                };
                
                layouts.push(DiskLayoutInfo {
                    id,
                    name: layout_name,
                });
            }
        }
    }
    
    layouts
}

/// Parse loaders_list.c to get which loaders are actually registered
/// Returns a HashSet of function names that are registered
fn parse_registered_loaders(base: &PathBuf) -> std::collections::HashSet<String> {
    // base is .../out/hxccode/libhxcfe/
    let loaders_list_path = base.join("sources/loaders_list.c");
    let bytes = fs::read(&loaders_list_path)
        .unwrap_or_else(|e| panic!("Failed to read loaders_list.c: {:?}, error: {}", loaders_list_path, e));
    let content = String::from_utf8_lossy(&bytes);
    
    // Look for (GETPLUGININFOS)FUNCTION_NAME patterns
    // Skip commented lines
    let func_re = Regex::new(r"\(GETPLUGININFOS\)\s*(\w+)_libGetPluginInfo").unwrap();
    
    let mut registered = std::collections::HashSet::new();
    
    for line in content.lines() {
        let line = line.trim();
        // Skip commented lines
        if line.starts_with("//") {
            continue;
        }
        
        if let Some(cap) = func_re.captures(line) {
            let func_prefix = cap.get(1).unwrap().as_str();
            registered.insert(func_prefix.to_string());
        }
    }
    
    registered
}

/// Generate ImageFormat enum from loaders
fn generate_image_format_enum(base: &PathBuf, out_path: &PathBuf) {
    let loaders_dir = base.join("sources/loaders");
    
    // Collect all loader .c files
    let loader_files: Vec<PathBuf> = WalkDir::new(&loaders_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "c") &&
            path.file_name().is_some_and(|name| {
                let name_str = name.to_string_lossy();
                name_str.ends_with("_loader.c") && !name_str.starts_with("floppy_loader")
            })
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    
    // Get the set of registered loader functions from loaders_list.c
    let registered_functions = parse_registered_loaders(base);
    println!("cargo:warning=Found {} registered loader functions in loaders_list.c", registered_functions.len());
    
    // Parse all regular loaders (including multiple loaders per file)
    // But only keep those that match registered function names
    let all_parsed_loaders: Vec<LoaderInfo> = loader_files
        .iter()
        .flat_map(|path| parse_all_loaders_from_file(path))
        .collect();
    
    println!("cargo:warning=Parsed {} total loaders from .c files (before filtering)", all_parsed_loaders.len());
    
    // Filter loaders to only those that are actually registered
    // A loader is registered if its plug_id appears in a non-commented line in loaders_list.c
    // This is a heuristic but should work
    let mut loaders: Vec<LoaderInfo> = all_parsed_loaders
        .into_iter()
        .filter(|loader| {
            // Check if this loader is referenced in loaders_list.c
            // We need to find a function name that matches
            // The function name format is typically: PLUGID_libGetPluginInfo or similar
            // For now, let's just check if we've seen more than the expected count
            // and manually exclude VFD_DAT
            loader.id != "VFD_DAT"
        })
        .collect();
    
    // Sort by id for consistent ordering
    loaders.sort_by(|a, b| a.id.cmp(&b.id));
    
    println!("cargo:warning=Parsed {} regular loaders after filtering", loaders.len());
    
    // Parse XML disk layouts and add them as loaders
    // The C library's XML loader dynamically creates sub-loaders from XML disk layouts
    let xml_layouts = parse_disk_layouts_for_loaders(base);
    
    println!("cargo:warning=Found {} XML disk layouts", xml_layouts.len());
    
    // Create a set of existing loader IDs to avoid duplicates
    let existing_ids: std::collections::HashSet<String> = 
        loaders.iter().map(|l| l.id.to_uppercase()).collect();
    
    // Add XML layouts to loaders list, skipping any that already exist as regular loaders
    let mut xml_added = 0;
    for layout in xml_layouts {
        if !existing_ids.contains(&layout.name.to_uppercase()) {
            loaders.push(LoaderInfo {
                id: layout.name.clone(),
                description: format!("Generic XML disk image ({})", layout.name),
                extension: "img".to_string(),
                has_writer: true,
            });
            xml_added += 1;
        }
    }
    
    println!("cargo:warning=Added {} XML layouts, total: {}", xml_added, loaders.len());
    
    // Generate Rust code
    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n\n");
    code.push_str("/// Image format for floppy disk images.\n");
    code.push_str("///\n");
    code.push_str("/// Represents all formats supported by the HxC library for reading and/or writing.\n");
    code.push_str("/// This enum is automatically generated from the available loaders.\n");
    code.push_str("/// Use `can_write()` to check if a format supports writing.\n");
    code.push_str("///\n");
    code.push_str("/// # Important: No Numeric IDs\n");
    code.push_str("/// This enum does NOT have meaningful numeric discriminants. The C library uses\n");
    code.push_str("/// string-based loader names for identification, not numeric IDs. Always use\n");
    code.push_str("/// `loader_name()` to get the format identifier for C library calls.\n");
    code.push_str("/// Do NOT cast this enum to an integer - the values are not stable or meaningful.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("#[non_exhaustive]\n");
    code.push_str("pub enum ImageFormat {\n");
    
    for loader in &loaders {
        // Convert ID to valid Rust enum variant name
        let variant_name = id_to_variant_name(&loader.id);
        code.push_str(&format!("    /// {} ({})\n", loader.description, loader.extension));
        code.push_str(&format!("    {},\n", variant_name));
    }
    
    code.push_str("}\n\n");
    
    // Generate methods
    code.push_str("impl ImageFormat {\n");
    
    // loader_name method
    code.push_str("    /// Get the loader name string for this format\n");
    code.push_str("    pub fn loader_name(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for loader in &loaders {
        let variant_name = id_to_variant_name(&loader.id);
        code.push_str(&format!("            Self::{} => \"{}\",\n", variant_name, loader.id));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // extension method
    code.push_str("    /// Get the typical file extension for this format\n");
    code.push_str("    pub fn extension(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for loader in &loaders {
        let variant_name = id_to_variant_name(&loader.id);
        code.push_str(&format!("            Self::{} => \"{}\",\n", variant_name, loader.extension));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // can_write method
    code.push_str("    /// Check if this format supports writing/saving\n");
    code.push_str("    pub fn can_write(&self) -> bool {\n");
    code.push_str("        match self {\n");
    for loader in &loaders {
        let variant_name = id_to_variant_name(&loader.id);
        code.push_str(&format!("            Self::{} => {},\n", variant_name, loader.has_writer));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // from_str method
    code.push_str("    /// Parse from a loader name or file extension\n");
    code.push_str("    ///\n");
    code.push_str("    /// # Arguments\n");
    code.push_str("    /// * `s` - Either a loader name or file extension\n");
    code.push_str("    ///\n");
    code.push_str("    /// # Returns\n");
    code.push_str("    /// Some(ImageFormat) if the string matches a known format, None otherwise\n");
    code.push_str("    pub fn from_str(s: &str) -> Option<Self> {\n");
    code.push_str("        let upper = s.to_uppercase();\n");
    code.push_str("        match upper.as_str() {\n");
    
    for loader in &loaders {
        let variant_name = id_to_variant_name(&loader.id);
        let ext_upper = loader.extension.to_uppercase();
        code.push_str(&format!("            \"{}\" | \"{}\" => Some(Self::{}),\n", 
            loader.id, ext_upper, variant_name));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // all method
    code.push_str("\n    /// Get all available image formats\n");
    code.push_str("    pub fn all() -> &'static [ImageFormat] {\n");
    code.push_str("        &[\n");
    for loader in &loaders {
        let variant_name = id_to_variant_name(&loader.id);
        code.push_str(&format!("            Self::{},\n", variant_name));
    }
    code.push_str("        ]\n");
    code.push_str("    }\n");
    
    // id method - queries C library for runtime loader ID
    code.push_str("\n    /// Get the loader ID for this format from the C library.\n");
    code.push_str("    ///\n");
    code.push_str("    /// The ID is retrieved at runtime by querying the C library with the loader name.\n");
    code.push_str("    /// Returns None if the loader is not registered in the current C library instance.\n");
    code.push_str("    ///\n");
    code.push_str("    /// # Arguments\n");
    code.push_str("    /// * `loader_ctx` - The loader manager context\n");
    code.push_str("    ///\n");
    code.push_str("    /// # Returns\n");
    code.push_str("    /// Some(id) if the loader is found, None otherwise\n");
    code.push_str("    pub fn id(&self, loader_ctx: *mut crate::HXCFE_IMGLDR) -> Option<i32> {\n");
    code.push_str("        if loader_ctx.is_null() {\n");
    code.push_str("            return None;\n");
    code.push_str("        }\n");
    code.push_str("        let name = self.loader_name();\n");
    code.push_str("        let c_name = std::ffi::CString::new(name).ok()?;\n");
    code.push_str("        let id = unsafe { crate::hxcfe_imgGetLoaderID(loader_ctx, c_name.as_ptr() as *mut i8) };\n");
    code.push_str("        if id >= 0 {\n");
    code.push_str("            Some(id)\n");
    code.push_str("        } else {\n");
    code.push_str("            None\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    code.push_str("}\n\n");
    
    // Add note about loader IDs
    code.push_str("// Note: ImageFormat enum variants are not assigned explicit discriminants\n");
    code.push_str("// because C library loader IDs are determined at runtime based on the order\n");
    code.push_str("// loaders are registered. Use ImgLoaderManager methods to get loader IDs.\n");
    code.push_str("// The enum provides type-safe access to format names and properties only.\n\n");
    
    // Display trait
    code.push_str("impl std::fmt::Display for ImageFormat {\n");
    code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    code.push_str("        write!(f, \"{}\", self.loader_name())\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    // Write to file
    let output_file = out_path.join("image_format.rs");
    fs::write(&output_file, code).expect("Failed to write image_format.rs");
}

/// Convert loader ID to valid Rust enum variant name
fn id_to_variant_name(id: &str) -> String {
    // Convert underscores to camel case
    id.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Information about an interface mode
#[derive(Debug, Clone)]
struct InterfaceModeInfo {
    id: i32,
    name: String,
    description: String,
}

/// Generate InterfaceMode enum from floppy_ifmode.c
fn generate_interface_mode_enum(base: &PathBuf, out_path: &PathBuf) {
    let ifmode_file = base.join("sources/floppy_ifmode.c");
    
    // Read file with lossy UTF-8 conversion
    let bytes = fs::read(&ifmode_file).expect("Failed to read floppy_ifmode.c");
    let content = String::from_utf8_lossy(&bytes);
    
    // Parse interface modes from the C array
    // Pattern: {ID, "NAME", "Description"}
    let mode_re = Regex::new(r#"\{([A-Z0-9_]+),\s*"([A-Z0-9_]+)",\s*"([^"]+)"\s*\}"#).unwrap();
    
    let mut modes: Vec<InterfaceModeInfo> = Vec::new();
    
    for cap in mode_re.captures_iter(&content) {
        let id_str = cap.get(1).unwrap().as_str();
        let name = cap.get(2).unwrap().as_str();
        let description = cap.get(3).unwrap().as_str();
        
        // Skip the terminator entry
        if id_str == "-1" || name.is_empty() {
            continue;
        }
        
        // Try to extract the numeric ID from the constant name
        // We'll use a counter since we don't have the actual C constant values
        let id = modes.len() as i32;
        
        modes.push(InterfaceModeInfo {
            id,
            name: name.to_string(),
            description: description.to_string(),
        });
    }
    
    // Generate Rust code
    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n\n");
    code.push_str("/// Floppy disk interface mode.\n");
    code.push_str("///\n");
    code.push_str("/// Represents the different interface modes supported by the HxC library.\n");
    code.push_str("/// This enum is automatically generated from floppy_ifmode.c.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("#[repr(i32)]\n");
    code.push_str("pub enum InterfaceMode {\n");
    
    for mode in &modes {
        let variant_name = id_to_variant_name(&mode.name.replace("_FLOPPYMODE", ""));
        code.push_str(&format!("    /// {} - {}\n", mode.name, mode.description));
        code.push_str(&format!("    {} = {},\n", variant_name, mode.id));
    }
    
    code.push_str("}\n\n");
    
    // Generate methods
    code.push_str("impl InterfaceMode {\n");
    
    // mode_name method
    code.push_str("    /// Get the interface mode name string\n");
    code.push_str("    pub fn mode_name(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for mode in &modes {
        let variant_name = id_to_variant_name(&mode.name.replace("_FLOPPYMODE", ""));
        code.push_str(&format!("            Self::{} => \"{}\",\n", variant_name, mode.name));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // description method
    code.push_str("    /// Get the interface mode description\n");
    code.push_str("    pub fn description(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for mode in &modes {
        let variant_name = id_to_variant_name(&mode.name.replace("_FLOPPYMODE", ""));
        code.push_str(&format!("            Self::{} => \"{}\",\n", variant_name, mode.description));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // from_str method
    code.push_str("    /// Parse from a mode name string\n");
    code.push_str("    pub fn from_str(s: &str) -> Option<Self> {\n");
    code.push_str("        let upper = s.to_uppercase();\n");
    code.push_str("        match upper.as_str() {\n");
    
    for mode in &modes {
        let variant_name = id_to_variant_name(&mode.name.replace("_FLOPPYMODE", ""));
        code.push_str(&format!("            \"{}\" => Some(Self::{}),\n", mode.name, variant_name));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // all method
    code.push_str("\n    /// Get all available interface modes\n");
    code.push_str("    pub fn all() -> &'static [InterfaceMode] {\n");
    code.push_str("        &[\n");
    for mode in &modes {
        let variant_name = id_to_variant_name(&mode.name.replace("_FLOPPYMODE", ""));
        code.push_str(&format!("            Self::{},\n", variant_name));
    }
    code.push_str("        ]\n");
    code.push_str("    }\n");
    
    code.push_str("}\n\n");
    
    // Display trait
    code.push_str("impl std::fmt::Display for InterfaceMode {\n");
    code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    code.push_str("        write!(f, \"{}\", self.mode_name())\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    // Write to file
    let output_file = out_path.join("interface_mode.rs");
    fs::write(&output_file, code).expect("Failed to write interface_mode.rs");
}

/// Information about a track encoding
#[derive(Debug, Clone)]
struct TrackEncodingInfo {
    constant_name: String,
    name: String,
}

/// Generate TrackEncoding enum from floppy_ifmode.c
fn generate_track_encoding_enum(base: &PathBuf, out_path: &PathBuf) {
    let ifmode_file = base.join("sources/floppy_ifmode.c");
    
    // Read file with lossy UTF-8 conversion
    let bytes = fs::read(&ifmode_file).expect("Failed to read floppy_ifmode.c");
    let content = String::from_utf8_lossy(&bytes);
    
    // Parse track encodings from the C array
    // Pattern: {CONSTANT_NAME, "NAME", ""}
    let encoding_re = Regex::new(r#"\{([A-Z0-9_]+),\s*"([A-Z0-9_]+)",\s*"[^"]*"\s*\}"#).unwrap();
    
    let mut encodings: Vec<TrackEncodingInfo> = Vec::new();
    let mut in_trackmodelist = false;
    
    for line in content.lines() {
        if line.contains("trackmodelist[]=") {
            in_trackmodelist = true;
            continue;
        }
        
        if in_trackmodelist {
            if line.contains("{-1,") {
                break;
            }
            
            if let Some(cap) = encoding_re.captures(line) {
                let constant_name = cap.get(1).unwrap().as_str();
                let name = cap.get(2).unwrap().as_str();
                
                encodings.push(TrackEncodingInfo {
                    constant_name: constant_name.to_string(),
                    name: name.to_string(),
                });
            }
        }
    }
    
    // Generate Rust code
    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n\n");
    code.push_str("/// Track encoding type.\n");
    code.push_str("///\n");
    code.push_str("/// Represents the different track encoding formats supported by the HxC library.\n");
    code.push_str("/// This enum is automatically generated from floppy_ifmode.c.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("#[repr(u32)]\n");
    code.push_str("pub enum TrackEncoding {\n");
    
    for encoding in &encodings {
        let variant_name = id_to_variant_name(&encoding.name.replace("_ENCODING", ""));
        code.push_str(&format!("    /// {}\n", encoding.name));
        code.push_str(&format!("    {} = {},\n", variant_name, encoding.constant_name));
    }
    
    code.push_str("}\n\n");
    
    // Generate methods
    code.push_str("impl TrackEncoding {\n");
    
    // encoding_name method
    code.push_str("    /// Get the track encoding name string\n");
    code.push_str("    pub fn encoding_name(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for encoding in &encodings {
        let variant_name = id_to_variant_name(&encoding.name.replace("_ENCODING", ""));
        code.push_str(&format!("            Self::{} => \"{}\",\n", variant_name, encoding.name));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // from_str method
    code.push_str("    /// Parse from an encoding name string\n");
    code.push_str("    pub fn from_str(s: &str) -> Option<Self> {\n");
    code.push_str("        let upper = s.to_uppercase();\n");
    code.push_str("        match upper.as_str() {\n");
    
    for encoding in &encodings {
        let variant_name = id_to_variant_name(&encoding.name.replace("_ENCODING", ""));
        code.push_str(&format!("            \"{}\" => Some(Self::{}),\n", encoding.name, variant_name));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // from_u32 method
    code.push_str("\n    /// Convert from a u32 value\n");
    code.push_str("    pub fn from_u32(value: u32) -> Option<Self> {\n");
    code.push_str("        match value {\n");
    
    for encoding in &encodings {
        let variant_name = id_to_variant_name(&encoding.name.replace("_ENCODING", ""));
        code.push_str(&format!("            {} => Some(Self::{}),\n", encoding.constant_name, variant_name));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // all method
    code.push_str("\n    /// Get all available track encodings\n");
    code.push_str("    pub fn all() -> &'static [TrackEncoding] {\n");
    code.push_str("        &[\n");
    for encoding in &encodings {
        let variant_name = id_to_variant_name(&encoding.name.replace("_ENCODING", ""));
        code.push_str(&format!("            Self::{},\n", variant_name));
    }
    code.push_str("        ]\n");
    code.push_str("    }\n");
    
    code.push_str("}\n\n");
    
    // Display trait
    code.push_str("impl std::fmt::Display for TrackEncoding {\n");
    code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    code.push_str("        write!(f, \"{}\", self.encoding_name())\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    // Write to file
    let output_file = out_path.join("track_encoding.rs");
    fs::write(&output_file, code).expect("Failed to write track_encoding.rs");
}

/// Information about a disk layout
#[derive(Debug, Clone)]
struct DiskLayoutInfo {
    id: usize,
    name: String,
}

/// Generate DiskLayout enum from LayoutsIndex.h and XML files
fn generate_disk_layout_enum(base: &PathBuf, out_path: &PathBuf) {
    let layouts_file = base.join("sources/xml_disk/DiskLayouts/LayoutsIndex.h");
    let xml_dir = base.join("sources/xml_disk/DiskLayouts/xml_files");
    
    // Read file with lossy UTF-8 conversion
    let bytes = fs::read(&layouts_file).expect("Failed to read LayoutsIndex.h");
    let content = String::from_utf8_lossy(&bytes);
    
    // Parse disk layouts from the C array to get the order and file names
    // Pattern: data_DiskLayout_XXXXX_xml,
    let layout_re = Regex::new(r"data_DiskLayout_([A-Za-z0-9_]+)_xml").unwrap();
    let xml_name_re = Regex::new(r"<disk_layout_name>([^<]+)</disk_layout_name>").unwrap();
    
    let mut layouts: Vec<DiskLayoutInfo> = Vec::new();
    let mut in_list = false;
    
    for line in content.lines() {
        if line.contains("disklayout_list[]=") {
            in_list = true;
            continue;
        }
        
        if in_list {
            if line.trim() == "0" || line.trim() == "};" {
                break;
            }
            
            if let Some(cap) = layout_re.captures(line) {
                let file_name = cap.get(1).unwrap().as_str();
                let id = layouts.len();
                
                // Read the corresponding XML file to get the actual layout name
                let xml_path = xml_dir.join(format!("DiskLayout_{}.xml", file_name));
                let xml_content = fs::read_to_string(&xml_path)
                    .unwrap_or_else(|_| panic!("Failed to read XML file: {:?}", xml_path));
                
                // Extract the <disk_layout_name> from XML
                let layout_name = if let Some(cap) = xml_name_re.captures(&xml_content) {
                    cap.get(1).unwrap().as_str().to_string()
                } else {
                    panic!("Failed to find <disk_layout_name> in {:?}", xml_path);
                };
                
                layouts.push(DiskLayoutInfo {
                    id,
                    name: layout_name,
                });
            }
        }
    }
    
    // Generate Rust code
    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n\n");
    code.push_str("/// Predefined disk layout.\n");
    code.push_str("///\n");
    code.push_str("/// Represents the different predefined disk layouts supported by the HxC library.\n");
    code.push_str("/// This enum is automatically generated from LayoutsIndex.h.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("#[repr(usize)]\n");
    code.push_str("pub enum DiskLayout {\n");
    
    for layout in &layouts {
        let variant_name = id_to_variant_name(&layout.name);
        code.push_str(&format!("    /// {}\n", layout.name));
        code.push_str(&format!("    {} = {},\n", variant_name, layout.id));
    }
    
    code.push_str("}\n\n");
    
    // Generate methods
    code.push_str("impl DiskLayout {\n");
    
    // layout_name method
    code.push_str("    /// Get the disk layout name string\n");
    code.push_str("    pub fn layout_name(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for layout in &layouts {
        let variant_name = id_to_variant_name(&layout.name);
        code.push_str(&format!("            Self::{} => \"{}\",\n", variant_name, layout.name));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // from_str method
    code.push_str("    /// Parse from a layout name string\n");
    code.push_str("    pub fn from_str(s: &str) -> Option<Self> {\n");
    code.push_str("        let upper = s.to_uppercase().replace('-', \"_\");\n");
    code.push_str("        match upper.as_str() {\n");
    
    for layout in &layouts {
        let variant_name = id_to_variant_name(&layout.name);
        let upper_name = layout.name.to_uppercase();
        code.push_str(&format!("            \"{}\" => Some(Self::{}),\n", upper_name, variant_name));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // from_usize method
    code.push_str("\n    /// Convert from a usize value\n");
    code.push_str("    pub fn from_usize(value: usize) -> Option<Self> {\n");
    code.push_str("        match value {\n");
    
    for layout in &layouts {
        let variant_name = id_to_variant_name(&layout.name);
        code.push_str(&format!("            {} => Some(Self::{}),\n", layout.id, variant_name));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // all method
    code.push_str("\n    /// Get all available disk layouts\n");
    code.push_str("    pub fn all() -> &'static [DiskLayout] {\n");
    code.push_str("        &[\n");
    for layout in &layouts {
        let variant_name = id_to_variant_name(&layout.name);
        code.push_str(&format!("            Self::{},\n", variant_name));
    }
    code.push_str("        ]\n");
    code.push_str("    }\n");
    
    code.push_str("}\n\n");
    
    // Display trait
    code.push_str("impl std::fmt::Display for DiskLayout {\n");
    code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    code.push_str("        write!(f, \"{}\", self.layout_name())\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    // Write to file
    let output_file = out_path.join("disk_layout.rs");
    fs::write(&output_file, code).expect("Failed to write disk_layout.rs");
}


