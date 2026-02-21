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
    
    
    // Parse all loaders
    let mut loaders: Vec<LoaderInfo> = loader_files
        .iter()
        .filter_map(|path| parse_loader_file(path))
        .collect();
    
    // Sort by id for consistent ordering
    loaders.sort_by(|a, b| a.id.cmp(&b.id));
    
    // Generate Rust code
    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n\n");
    code.push_str("/// Image format for floppy disk images.\n");
    code.push_str("///\n");
    code.push_str("/// Represents all formats supported by the HxC library for reading and/or writing.\n");
    code.push_str("/// This enum is automatically generated from the available loaders.\n");
    code.push_str("/// Use `can_write()` to check if a format supports writing.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
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
    
    code.push_str("}\n\n");
    
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

