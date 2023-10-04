mod floppy_interface;
mod fs_manager;
mod img;
mod img_loaders;
mod layouts;
mod sector_access;

pub use fs_manager::FileSystemManager;
use once_cell::sync::Lazy;

use std::{
    ffi::{CStr},
    ops::Deref,
    path::Path,
    sync::Arc,
};

use floppy_interface::FloppyInterface;
use hxcfe_sys::{
    hxcfe_getVersion, AED6200P_MFM_ENCODING, AMIGA_MFM_ENCODING,
    APPLEII_GCR1_ENCODING, APPLEII_GCR2_ENCODING, APPLEII_HDDD_A2_GCR1_ENCODING,
    APPLEII_HDDD_A2_GCR2_ENCODING, APPLEMAC_GCR_ENCODING, ARBURGDAT_ENCODING, ARBURGSYS_ENCODING,
    C64_GCR_ENCODING, DEC_RX02_M2FM_ENCODING, EMU_FM_ENCODING, HEATHKIT_HS_FM_ENCODING, HXCFE, ISOIBM_FM_ENCODING, ISOIBM_MFM_ENCODING, MEMBRAIN_MFM_ENCODING,
    MICRALN_HS_FM_ENCODING, NORTHSTAR_HS_MFM_ENCODING, QD_MO5_ENCODING, TYCOM_FM_ENCODING,
    UNKNOWN_ENCODING, VICTOR9K_GCR_ENCODING,
};
pub use img::Img;
pub use img_loaders::ImgLoaderManager;
pub use layouts::LayoutManager;

#[repr(i32)]
#[derive(enumn::N, PartialEq, Debug)]
pub enum HxcfeError {
    HXCFE_VALIDFILE = 1,
    HXCFE_NOERROR = 0,
    HXCFE_ACCESSERROR = -1,
    HXCFE_BADFILE = -2,
    HXCFE_FILECORRUPTED = -3,
    HXCFE_BADPARAMETER = -4,
    HXCFE_INTERNALERROR = -5,
    HXCFE_UNSUPPORTEDFILE = -6,
}

#[repr(u32)]
#[derive(Copy, Clone, enumn::N)]
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
    pub fn get() -> &'static Hxcfe {
        &HXCFE_INSTANCE
    }

    pub fn version(&self) -> &str {
        let version = unsafe { hxcfe_getVersion(self.handler) };
        let version = unsafe { CStr::from_ptr(version) };
        version.to_str().unwrap()
    }

    pub fn loaders_manager<'hfe>(&'hfe self) -> Option<ImgLoaderManager> {
        ImgLoaderManager::new(self)
    }

    pub fn layout_manager<'hfe>(&'hfe self) -> Option<LayoutManager> {
        LayoutManager::new(self)
    }

    pub fn file_system_manager<'hfe>(&'hfe self) -> Option<FileSystemManager<'hfe>> {
        FileSystemManager::new(self)
    }

    pub fn floppy_interface<'hfe>(&'hfe self, idx: i32) -> Option<FloppyInterface<'hfe>> {
        FloppyInterface::new(self, idx)
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

        loader
            .load(&p)
            .or_else(|e| Err(format!("Load error {:?}", e)))
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
            .or_else(|e| Err(format!("Save error {:?}", e)))
    }
}

#[cfg(test)]
mod test {
    use parking_lot::Mutex;

    use once_cell::sync::Lazy;

    use crate::Hxcfe;

    static TESTS: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn version() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        assert_eq!(hxcfe.version(), "2.14.12.1");
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
            println!("\t{:?}", manager.layout_name(i));
            println!("\t{:?}", manager.layout_description(i));
        }
    }

    #[test]
    fn list_interfaces() {
        let _locker = TESTS.lock();
        let hxcfe = Hxcfe::get();
        let mut idx = 0;
        while let Some(interface) = hxcfe.floppy_interface(idx) {
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
