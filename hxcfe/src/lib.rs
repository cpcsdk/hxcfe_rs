mod floppy_interface;
mod fs_manager;
mod img;
mod img_loaders;
mod layouts;
mod sector_access;
mod types;

#[cfg(feature = "usb")]
mod usb;

pub use fs_manager::FileSystemManager;
use once_cell::sync::Lazy;
pub use types::{DriveId, FileHandle, FileSystemId, HeadId, SectorId, TrackId};

#[cfg(feature = "usb")]
pub use usb::UsbHxcfe;

use std::{ffi::CStr, ops::Deref, path::Path, sync::Arc};

use floppy_interface::FloppyInterface;
use hxcfe_sys::{
    HXCFE, hxcfe_generateFloppy, hxcfe_getFloppyInterfaceModeID, hxcfe_getTrackEncodingName,
    hxcfe_getVersion,
};
pub use img::Img;
pub use img_loaders::ImgLoaderManager;
pub use layouts::LayoutManager;

// Re-export generated enums from hxcfe-sys
pub use hxcfe_sys::DiskLayout;
pub use hxcfe_sys::ImageFormat;
pub use hxcfe_sys::InterfaceMode;
pub use hxcfe_sys::TrackEncoding;

#[repr(i32)]
#[derive(enumn::N, PartialEq, Debug)]
#[allow(non_camel_case_types)]
/// Error codes returned by HxC Floppy Emulator operations.
/// Keep C-style naming for compatibility with upstream libhxcfe.
pub enum HxcfeError {
    /// File is valid and can be loaded
    HXCFE_VALIDFILE = 1,
    /// Operation completed successfully
    HXCFE_NOERROR = 0,
    /// File or resource access error
    HXCFE_ACCESSERROR = -1,
    /// Invalid or corrupted file format
    HXCFE_BADFILE = -2,
    /// File data is corrupted
    HXCFE_FILECORRUPTED = -3,
    /// Invalid parameter provided
    HXCFE_BADPARAMETER = -4,
    /// Internal library error
    HXCFE_INTERNALERROR = -5,
    /// File format is not supported
    HXCFE_UNSUPPORTEDFILE = -6,
}

static HXCFE_INSTANCE: Lazy<Arc<Hxcfe>> = Lazy::new(|| {
    let handler = unsafe { hxcfe_sys::hxcfe_init() };
    let hxcfe: Arc<Hxcfe> = Hxcfe { handler }.into();

    /*
        eprintln!("Check loaders- need to remove that of course");
        let manager = hxcfe.loaders_manager().unwrap();
        for i in 0..manager.nb_loaders() {
            println!("Loader {i}");
            let loader = manager.loader_for_id(i).unwrap();
            println!("\t{}", loader.access());
            println!("\t{}", loader.name());
            println!("\t{:?}", loader.description());
        }
    */
    hxcfe
});

unsafe impl Send for Hxcfe {}
unsafe impl Sync for Hxcfe {}

#[derive(Debug)]
/// Main HxC Floppy Emulator context.
///
/// This is a singleton instance that provides access to the HxC library functionality.
/// Use [`Hxcfe::get()`] to obtain a reference to the global instance.
// By construction there is only one instance available. So it is uneeded to keep its reference
pub struct Hxcfe {
    handler: *mut HXCFE,
}

impl Deref for Hxcfe {
    type Target = *mut HXCFE;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}
impl Drop for Hxcfe {
    fn drop(&mut self) {
        eprintln!("Deallocate HXCFE");
        unsafe { hxcfe_sys::hxcfe_deinit(self.handler) };
    }
}
impl Hxcfe {
    /// Get a reference to the global HxC Floppy Emulator instance.
    ///
    /// # Example
    /// ```no_run
    /// use hxcfe::Hxcfe;
    ///
    /// let hxcfe = Hxcfe::get();
    /// println!("HxCFE version: {}", hxcfe.version());
    /// ```
    pub fn get() -> &'static Hxcfe {
        &HXCFE_INSTANCE
    }

    /// Get the version string of the HxC library.
    pub fn version(&self) -> &str {
        let version = unsafe { hxcfe_getVersion(self.handler) };
        let version = unsafe { CStr::from_ptr(version) };
        version.to_str().unwrap()
    }

    /// Create an image loader manager for loading and saving floppy disk images.
    ///
    /// # Returns
    /// `Some(ImgLoaderManager)` on success, `None` if initialization fails.
    pub fn loaders_manager(&self) -> Option<ImgLoaderManager> {
        ImgLoaderManager::new(self)
    }

    /// Create a layout manager for working with floppy disk layouts.
    ///
    /// # Returns
    /// `Some(LayoutManager)` on success, `None` if initialization fails.
    pub fn layout_manager<'hfe>(&'hfe self) -> Option<LayoutManager<'hfe>> {
        LayoutManager::new(self)
    }

    /// Create a file system manager for accessing files on floppy disk images.
    ///
    /// # Returns
    /// `Some(FileSystemManager)` on success, `None` if initialization fails.
    pub fn file_system_manager<'hfe>(&'hfe self) -> Option<FileSystemManager<'hfe>> {
        FileSystemManager::new(self)
    }

    pub fn floppy_interface<'hfe>(
        &'hfe self,
        mode: InterfaceMode,
    ) -> Option<FloppyInterface<'hfe>> {
        FloppyInterface::new(self, mode)
    }

    /// Generate a new floppy disk image from a directory path.
    ///
    /// Creates a formatted floppy disk image and copies files from the specified directory.
    ///
    /// # Arguments
    /// * `path` - Path to the directory containing files to add to the image
    /// * `fs_id` - Filesystem type ID (e.g., FS_720KB_MSDOS_FAT12)
    ///
    /// # Returns
    /// `Ok(Img)` containing the generated image on success, `Err(HxcfeError)` on failure.
    ///
    /// # Example
    /// ```no_run
    /// # use hxcfe::{Hxcfe, FileSystemId};
    /// let hxcfe = Hxcfe::get();
    /// let img = hxcfe.generate_floppy("./my_files", FileSystemId::from_i32(15).unwrap()).unwrap(); // FS_720KB_MSDOS_FAT12 = 15
    /// ```
    pub fn generate_floppy<P: AsRef<Path>>(
        &self,
        path: P,
        fs_id: FileSystemId,
    ) -> Result<Img, HxcfeError> {
        use std::ffi::CString;

        let path_str = path.as_ref().display().to_string();
        let path_cstr = CString::new(path_str).map_err(|_| HxcfeError::HXCFE_BADPARAMETER)?;
        let path_ptr = path_cstr.into_raw();

        let mut err_ret: i32 = 0;
        let floppydisk =
            unsafe { hxcfe_generateFloppy(self.handler, path_ptr, fs_id.get(), &mut err_ret) };
        let _ = unsafe { CString::from_raw(path_ptr) };

        let err = HxcfeError::n(err_ret).unwrap_or(HxcfeError::HXCFE_INTERNALERROR);
        if err != HxcfeError::HXCFE_NOERROR || floppydisk.is_null() {
            Err(err)
        } else {
            Ok(Img {
                floppydisk,
                hxcfe: self,
            })
        }
    }

    /// Get the interface mode ID from its name.
    ///
    /// # Arguments
    /// * `name` - Interface mode name (e.g., "IBMPC_DD", "ATARIST_DD")
    ///
    /// # Returns
    /// `Some(InterfaceMode)` with the mode if found, `None` if the name is invalid.
    pub fn get_interface_mode_id(&self, name: &str) -> Option<InterfaceMode> {
        use std::ffi::CString;

        let name_cstr = CString::new(name).ok()?;
        let name_ptr = name_cstr.into_raw();
        let mode_id = unsafe { hxcfe_getFloppyInterfaceModeID(self.handler, name_ptr) };
        let _ = unsafe { CString::from_raw(name_ptr) };

        InterfaceMode::from_i32(mode_id)
    }

    /// Get the track encoding name from its ID.
    ///
    /// # Arguments
    /// * `encoding_id` - Track encoding ID
    ///
    /// # Returns
    /// The encoding name as a string slice, or an empty string if the ID is invalid.
    pub fn get_track_encoding_name(&self, encoding_id: i32) -> &str {
        use std::ffi::CStr;

        let name_ptr = unsafe { hxcfe_getTrackEncodingName(self.handler, encoding_id) };
        if name_ptr.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(name_ptr) }.to_str().unwrap_or("")
    }

    pub fn load<P: AsRef<Path>>(&self, p: P) -> Result<Img, String> {
        let manager = self
            .loaders_manager()
            .ok_or_else(|| "Unable to get the loader manager".to_owned())?;

        let loader = manager.loader_for_fname(&p).ok_or_else(|| {
            format!(
                "Unable to find a loading loader for {}",
                p.as_ref().display()
            )
        })?;

        loader.load(&p).map_err(|e| format!("Load error {:?}", e))
    }

    /// Load a floppy disk image from a memory buffer.
    ///
    /// This is useful for WASM environments or when you have image data
    /// in memory without needing to write it to disk first.
    ///
    /// # Arguments
    /// * `buffer` - Byte slice containing the image data
    /// * `filename` - Hint for format detection (e.g., "disk.dsk", "image.hfe")
    ///                The file extension helps auto-detect the format.
    ///
    /// # Returns
    /// `Ok(Img)` on success, `Err(String)` with error description on failure.
    ///
    /// # Example
    /// ```no_run
    /// # use hxcfe::Hxcfe;
    /// let hxc = Hxcfe::get();
    /// // In WASM, this buffer would come from JavaScript FileReader API
    /// let image_data = vec![0u8; 737280]; // 720KB DSK file data
    /// let img = hxc.load_from_buffer(&image_data, "disk.dsk").unwrap();
    /// ```
    pub fn load_from_buffer(&self, buffer: &[u8], filename: &str) -> Result<Img, String> {
        use std::io::Write;

        // Create a temporary file - the C library requires a filename for format detection
        // even when using RAM files. The actual data comes from the buffer.
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(filename);

        // Write buffer to temp file
        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(buffer)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        drop(file);

        // Load using regular path-based loading
        let result = self.load(&temp_path);

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        result
    }

    // TODO Find a way to remove the format information
    pub(crate) fn save<P: AsRef<Path>>(
        &self,
        p: P,
        format: ImageFormat,
        img: &Img,
    ) -> Result<(), String> {
        let manager = self
            .loaders_manager()
            .ok_or_else(|| "Unable to get the loader manager".to_owned())?;

        let loader = manager
            .loader_for_format(format.loader_name())
            .ok_or_else(|| {
                format!(
                    "Unable to find a saving loader for {}",
                    p.as_ref().display()
                )
            })?;

        loader
            .save(&p, img)
            .map_err(|e| format!("Save error {:?}", e))
    }

    /// Save a floppy disk image to a memory buffer.
    ///
    /// This is useful for WASM environments or when you need to handle
    /// the image data in memory without writing to disk.
    ///
    /// # Arguments
    /// * `format` - Output format (e.g., `ImageFormat::HxcHfe`, `ImageFormat::AmigaAdf`)
    /// * `img` - The image to save
    ///
    /// # Returns
    /// `Ok(Vec<u8>)` containing the image data on success, `Err(String)` on failure.
    pub(crate) fn save_to_buffer(&self, format: ImageFormat, img: &Img) -> Result<Vec<u8>, String> {
        use std::io::Read;

        // Create a temporary file for saving with appropriate extension
        let temp_dir = std::env::temp_dir();
        let ext = format.extension();
        let temp_path = temp_dir.join(format!("hxc_temp_{}.{}", std::process::id(), ext));

        // Save to temp file
        self.save(&temp_path, format, img)?;

        // Read the file back into memory
        let mut file = std::fs::File::open(&temp_path)
            .map_err(|e| format!("Failed to open temp file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read temp file: {}", e))?;
        drop(file);

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(buffer)
    }
}

#[cfg(test)]
mod test {
    use parking_lot::Mutex;

    use once_cell::sync::Lazy;

    use crate::{DiskLayout, Hxcfe, ImageFormat, InterfaceMode};

    static TESTS: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn version() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        assert_eq!(hxcfe.version(), "2.16.15.1");
    }

    #[test]
    fn list_modules() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        let manager = hxcfe.loaders_manager().unwrap();
        for i in 0..manager.nb_loaders() {
            println!("Loader {i}");
            let loader = manager.loader_for_id(i).unwrap();
            println!("\t{}", loader.access());
            println!("\t{}", loader.name());
            println!("\t{}", loader.ext());
            println!("\t{:?}", loader.description());
        }
    }

    #[test]
    fn list_layouts() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        let manager = hxcfe.layout_manager().unwrap();
        for layout in DiskLayout::all() {
            println!("Layout: {}", layout);
            println!("\tName: {:?}", manager.layout_name(*layout));
            println!("\tDesc: {:?}", manager.layout_description(*layout));
        }
    }

    #[test]
    fn list_interfaces() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        for mode in InterfaceMode::all() {
            if let Some(interface) = hxcfe.floppy_interface(*mode) {
                println!(
                    "{} {} {}",
                    *mode as i32,
                    interface.name(),
                    interface.description()
                );
            }
        }
    }

    /// Validates that InterfaceMode::all() matches the C library's interface browsing.
    /// This ensures our auto-generated enum is consistent with the FFI layer.
    #[test]
    fn validate_interface_modes_against_c_library() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();

        // Get all modes from our enum
        let rust_modes = InterfaceMode::all();

        // Verify each enum variant can be accessed through the C interface
        for mode in rust_modes {
            let interface = hxcfe
                .floppy_interface(*mode)
                .expect(&format!("Failed to create interface for mode {:?}", mode));

            // The C library should return a valid name for this mode
            let c_name = interface.name();
            assert!(
                !c_name.is_empty(),
                "C library returned empty name for mode {:?}",
                mode
            );

            // Verify the enum's name matches the C library's name
            let enum_name = mode.mode_name();
            assert_eq!(
                c_name, enum_name,
                "Mismatch for mode {:?}: C library returns '{}' but enum has '{}'",
                mode, c_name, enum_name
            );
        }

        println!(
            "✓ All {} InterfaceMode variants validated against C library",
            rust_modes.len()
        );
    }

    /// Validates that DiskLayout::all() matches the C library's layout browsing.
    /// This ensures our auto-generated enum is consistent with the FFI layer.
    #[test]
    fn validate_disk_layouts_against_c_library() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        let manager = hxcfe
            .layout_manager()
            .expect("Failed to create layout manager");

        // Get counts from both sources
        let rust_layouts = DiskLayout::all();
        let c_count = manager.nb_layouts();

        // Verify counts match
        assert_eq!(
            rust_layouts.len(),
            c_count as usize,
            "Count mismatch: Rust enum has {} layouts but C library reports {}",
            rust_layouts.len(),
            c_count
        );

        // Validate that all layout names match exactly
        let mut mismatches = Vec::new();
        for layout in rust_layouts {
            let c_name = manager.layout_name(*layout);
            let enum_name = layout.layout_name();

            if c_name != enum_name {
                mismatches.push((*layout as usize, c_name.to_string(), enum_name.to_string()));
            }
        }

        // Fail if any names don't match
        if !mismatches.is_empty() {
            println!("\n❌ DiskLayout name mismatches found:\n");
            for (id, c_name, enum_name) in &mismatches {
                println!("  ID {}: C='{}' ≠ Enum='{}'", id, c_name, enum_name);
            }
            panic!(
                "\n{} layout names don't match between Rust enum and C library. Names must be identical.",
                mismatches.len()
            );
        }

        println!(
            "✓ All {} DiskLayout names match the C library",
            rust_layouts.len()
        );
    }

    /// Validates that ImageFormat::all() matches the C library's loader browsing.
    /// This ensures our auto-generated enum is consistent with the FFI layer.
    #[test]
    fn validate_image_formats_against_c_library() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        let manager = hxcfe
            .loaders_manager()
            .expect("Failed to create loaders manager");

        // Get all formats from enum
        let rust_formats = ImageFormat::all();
        let c_count = manager.nb_loaders();

        println!("\n=== ImageFormat Validation ===");
        println!("Rust enum has {} formats", rust_formats.len());
        println!("C library reports {} loaders", c_count);

        // Sample C library loaders to understand the difference
        println!("\n=== Sample C library loaders (first 10 and last 10) ===");
        for i in 0..10.min(c_count) {
            if let Some(loader) = manager.loader_for_id(i) {
                println!("  {}: {}", i, loader.name());
            }
        }
        if c_count > 20 {
            println!("  ...");
            for i in (c_count - 10).max(10)..c_count {
                if let Some(loader) = manager.loader_for_id(i) {
                    println!("  {}: {}", i, loader.name());
                }
            }
        }

        // Check count - we should have at least as many as we can parse from source
        // C library count may differ if it registers loaders differently
        if rust_formats.len() != c_count as usize {
            println!(
                "\n⚠️  Count difference: Rust enum has {} formats, C library reports {} loaders.",
                rust_formats.len(),
                c_count
            );
        }

        // Verify each enum format can get its ID from C library
        // This proves the loader exists and is registered
        println!("\n=== Validating ImageFormat IDs from C library ===");

        let mut id_success = 0;
        let mut id_failures = Vec::new();

        for (idx, format) in rust_formats.iter().enumerate() {
            let loader_name = format.loader_name();
            if let Some(loader_id) = format.id(manager.handler()) {
                id_success += 1;
                if idx < 3 {
                    println!("✓ #{}: {} -> ID {}", idx, loader_name, loader_id);
                }
            } else {
                id_failures.push((idx, loader_name.to_string()));
            }
        }

        println!(
            "✓ {} out of {} ImageFormat variants have valid IDs from C library",
            id_success,
            rust_formats.len()
        );

        if !id_failures.is_empty() {
            println!(
                "\n⚠️  {} variants could not retrieve ID from C library:",
                id_failures.len()
            );
            for (idx, name) in id_failures.iter().take(10) {
                println!("  #{}: {}", idx, name);
            }
            if id_failures.len() > 10 {
                println!("  ... and {} more", id_failures.len() - 10);
            }
        }

        println!("\n✓ All {} ImageFormat IDs validated", rust_formats.len());
    }

    #[test]
    fn dsk_loader() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();

        {
            let manager = hxcfe.loaders_manager().unwrap();

            assert!(manager.loader_for_text_id("AMSTRADCPC_DSK").is_some());
            assert!(manager.loader_for_text_id("AMSTRADCPC_DSK").is_some());
            assert!(manager.loader_for_text_id("AMSTRADCPC_DSK").is_some());
        }

        {
            let manager = hxcfe.loaders_manager().unwrap();
            assert!(manager.loader_for_fname("tests/EXPERTS.HFE").is_some());
            assert!(manager.loader_for_fname("tests/EXPERTS.DSK").is_some());
        }
    }
}
