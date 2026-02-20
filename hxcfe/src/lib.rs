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
pub use types::{
    DriveId, FileHandle, FileSystemId, HeadId, InterfaceIndex, InterfaceModeId, LayoutIndex,
    SectorId, TrackId,
};

#[cfg(feature = "usb")]
pub use usb::UsbHxcfe;

use std::{ffi::CStr, ops::Deref, path::Path, sync::Arc};

use floppy_interface::FloppyInterface;
use hxcfe_sys::{
    AED6200P_MFM_ENCODING, AMIGA_MFM_ENCODING, APPLEII_GCR1_ENCODING, APPLEII_GCR2_ENCODING,
    APPLEII_HDDD_A2_GCR1_ENCODING, APPLEII_HDDD_A2_GCR2_ENCODING, APPLEMAC_GCR_ENCODING,
    ARBURGDAT_ENCODING, ARBURGSYS_ENCODING, C64_GCR_ENCODING, DEC_RX02_M2FM_ENCODING,
    EMU_FM_ENCODING, HEATHKIT_HS_FM_ENCODING, HXCFE, ISOIBM_FM_ENCODING, ISOIBM_MFM_ENCODING,
    MEMBRAIN_MFM_ENCODING, MICRALN_HS_FM_ENCODING, NORTHSTAR_HS_MFM_ENCODING, QD_MO5_ENCODING,
    TYCOM_FM_ENCODING, UNKNOWN_ENCODING, VICTOR9K_GCR_ENCODING, hxcfe_generateFloppy,
    hxcfe_getFloppyInterfaceModeID, hxcfe_getTrackEncodingName, hxcfe_getVersion,
};
pub use img::Img;
pub use img_loaders::ImgLoaderManager;
pub use layouts::LayoutManager;

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

#[repr(u32)]
#[derive(Copy, Clone, enumn::N)]
#[allow(non_camel_case_types)]
pub enum TrackEncoding {
    IsoIbmMfm = ISOIBM_MFM_ENCODING,
    Amiga_Mfm = AMIGA_MFM_ENCODING,
    IsoIbmFm = ISOIBM_FM_ENCODING,
    EmuFm = EMU_FM_ENCODING,
    TycomFm = TYCOM_FM_ENCODING,
    MembrainMfm = MEMBRAIN_MFM_ENCODING,
    AppleiiGrc1 = APPLEII_GCR1_ENCODING,
    AppleiiGrc2 = APPLEII_GCR2_ENCODING,
    AppleiiHdddA2Grc1 = APPLEII_HDDD_A2_GCR1_ENCODING,
    AppleiiHdddA2Grc2 = APPLEII_HDDD_A2_GCR2_ENCODING,
    ArburgDat = ARBURGDAT_ENCODING,
    ArburgSys = ARBURGSYS_ENCODING,
    Aed6200p = AED6200P_MFM_ENCODING,
    NorthstarHsMfm = NORTHSTAR_HS_MFM_ENCODING,
    HeatkitHsFm = HEATHKIT_HS_FM_ENCODING,
    DecRx02M2fm = DEC_RX02_M2FM_ENCODING,
    ApplemacGrc = APPLEMAC_GCR_ENCODING,
    QdMo5 = QD_MO5_ENCODING,
    C64Gcr = C64_GCR_ENCODING,
    Victor9kGcr = VICTOR9K_GCR_ENCODING,
    MicralnHsFm = MICRALN_HS_FM_ENCODING,
    Unknown = UNKNOWN_ENCODING,
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
        idx: InterfaceIndex,
    ) -> Option<FloppyInterface<'hfe>> {
        FloppyInterface::new(self, idx)
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
    /// `Some(InterfaceModeId)` with the mode ID if found, `None` if the name is invalid.
    pub fn get_interface_mode_id(&self, name: &str) -> Option<InterfaceModeId> {
        use std::ffi::CString;

        let name_cstr = CString::new(name).ok()?;
        let name_ptr = name_cstr.into_raw();
        let mode_id = unsafe { hxcfe_getFloppyInterfaceModeID(self.handler, name_ptr) };
        let _ = unsafe { CString::from_raw(name_ptr) };

        if mode_id >= 0 {
            Some(InterfaceModeId::new(mode_id))
        } else {
            None
        }
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

    // TODO Find a way to remove the format information
    pub(crate) fn save<P: AsRef<Path>>(&self, p: P, format: &str, img: &Img) -> Result<(), String> {
        let manager = self
            .loaders_manager()
            .ok_or_else(|| "Unable to get the loader manager".to_owned())?;

        let loader = manager.loader_for_format(format).ok_or_else(|| {
            format!(
                "Unable to find a saving loader for {}",
                p.as_ref().display()
            )
        })?;

        loader
            .save(&p, img)
            .map_err(|e| format!("Save error {:?}", e))
    }
}

#[cfg(test)]
mod test {
    use parking_lot::Mutex;

    use once_cell::sync::Lazy;

    use crate::{Hxcfe, InterfaceIndex, LayoutIndex};

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
        for i in 0..manager.nb_layouts() {
            println!("Loader {i}");
            println!("\t{:?}", manager.layout_name(LayoutIndex::new(i)));
            println!("\t{:?}", manager.layout_description(LayoutIndex::new(i)));
        }
    }

    #[test]
    fn list_interfaces() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        let mut idx = 0;
        while let Some(interface) = hxcfe.floppy_interface(InterfaceIndex::new(idx)) {
            idx += 1;
            println!("{idx} {} {}", interface.name(), interface.description());
        }
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
