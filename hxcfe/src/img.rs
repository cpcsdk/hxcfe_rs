use std::ffi::CStr;

use std::path::Path;

use hxcfe_sys::hxcfe_floppyDuplicate;
use hxcfe_sys::hxcfe_floppyGetInterfaceMode;
use hxcfe_sys::hxcfe_floppySectorBySectorCopy;
use hxcfe_sys::hxcfe_getFloppyInterfaceModeDesc;
use hxcfe_sys::hxcfe_getFloppyInterfaceModeName;
use hxcfe_sys::hxcfe_getFloppySize;
use hxcfe_sys::hxcfe_getNumberOfSide;
use hxcfe_sys::hxcfe_getNumberOfTrack;
use hxcfe_sys::hxcfe_imgDeInitLoader;
use hxcfe_sys::hxcfe_imgInitLoader;
use hxcfe_sys::hxcfe_imgUnload;
use hxcfe_sys::{HXCFE_FLOPPY};

use crate::sector_access::SectorAccess;
use crate::{Hxcfe, HxcfeError};

#[derive(Debug)]
pub struct FloppySizeInfo {
    pub nb_sectors: i32,
    pub nb_bad_sectors: i32,
    pub size: i32,
}

#[derive(Debug)]
pub struct Img {
    pub floppydisk: *mut HXCFE_FLOPPY,
    pub(crate) hxcfe: *const Hxcfe,
}

pub struct Interface<'img> {
    pub img: &'img Img,
    pub ifmode: i32,
}

impl Drop for Img {
    fn drop(&mut self) {
        // Unload the floppy disk image to free resources
        // We need to create a temporary loader manager for this
        unsafe {
            let loader_ctx = hxcfe_imgInitLoader(self.hxcfe.as_ref().unwrap().handler);
            if !loader_ctx.is_null() {
                hxcfe_imgUnload(loader_ctx, self.floppydisk);
                hxcfe_imgDeInitLoader(loader_ctx);
            }
        }
    }
}

impl Img {
    pub fn save<P: AsRef<Path>>(&self, p: P, format: &str) -> Result<(), String> {
        unsafe { self.hxcfe.as_ref().unwrap().save(p, format, self) }
    }

    pub fn interface_mode(&self) -> Interface {
        let ifmode = unsafe {
            hxcfe_floppyGetInterfaceMode(self.hxcfe.as_ref().unwrap().handler, self.floppydisk)
        };
        Interface { img: self, ifmode }
    }

    pub fn sector_access(&self) -> Option<SectorAccess> {
        SectorAccess::new(self)
    }

    /// Get the raw floppy disk pointer for low-level operations.
    /// 
    /// This is primarily used internally for USB operations and other
    /// low-level library functions.
    pub fn floppy(&self) -> *mut HXCFE_FLOPPY {
        self.floppydisk
    }

    // XXX how is it different than nb_tracks_per_head ?
    pub fn nb_tracks(&self) -> i32 {
        let res = unsafe {
            hxcfe_getNumberOfTrack(self.hxcfe.as_ref().unwrap().handler, self.floppydisk)
        };
        res
    }

    pub fn nb_tracks_per_head(&self) -> i32 {
        unsafe { self.floppydisk.as_ref().unwrap().floppyNumberOfTrack }
    }

    pub fn nb_sides(&self) -> i32 {
        let res =
            unsafe { hxcfe_getNumberOfSide(self.hxcfe.as_ref().unwrap().handler, self.floppydisk) };
        res
    }

    pub fn size(&self) -> i32 {
        self.size_info().size
    }

    pub fn nb_sectors(&self) -> i32 {
        self.size_info().nb_sectors
    }

    pub fn nb_bad_sectors(&self) -> i32 {
        self.size_info().nb_bad_sectors
    }

    pub fn size_info(&self) -> FloppySizeInfo {
        let mut nbofsector = 0;
        let mut nbbadsector = 0;
        let size = unsafe {
            hxcfe_getFloppySize(
                self.hxcfe.as_ref().unwrap().handler,
                self.floppydisk,
                &mut nbofsector,
                &mut nbbadsector
            )
        };
        FloppySizeInfo {
            nb_sectors: nbofsector,
            nb_bad_sectors: nbbadsector,
            size,
        }
    }

    /// Create a duplicate copy of this floppy disk image.
    /// 
    /// # Returns
    /// `Ok(Img)` containing the duplicated image on success, `Err(HxcfeError)` on failure.
    /// 
    /// # Example
    /// ```no_run
    /// # use hxcfe::Hxcfe;
    /// let hxcfe = Hxcfe::get();
    /// let img = hxcfe.load("disk.hfe").unwrap();
    /// let copy = img.duplicate().unwrap();
    /// ```
    pub fn duplicate(&self) -> Result<Img, HxcfeError> {
        let new_floppy = unsafe {
            hxcfe_floppyDuplicate(
                self.hxcfe.as_ref().unwrap().handler,
                self.floppydisk
            )
        };
        
        if new_floppy.is_null() {
            Err(HxcfeError::HXCFE_INTERNALERROR)
        } else {
            Ok(Img {
                floppydisk: new_floppy,
                hxcfe: self.hxcfe,
            })
        }
    }

    /// Copy sectors from another floppy disk image to this one.
    /// 
    /// Performs a sector-by-sector copy operation.
    /// 
    /// # Arguments
    /// * `src` - Source image to copy from
    /// 
    /// # Returns
    /// `Ok(())` on success, `Err(HxcfeError)` on failure.
    pub fn copy_sectors_from(&mut self, src: &Img) -> Result<(), HxcfeError> {
        let ret = unsafe {
            hxcfe_floppySectorBySectorCopy(
                self.hxcfe.as_ref().unwrap().handler,
                self.floppydisk,
                src.floppydisk,
                0
            )
        };
        
        let ret = HxcfeError::n(ret).unwrap_or(HxcfeError::HXCFE_INTERNALERROR);
        if ret == HxcfeError::HXCFE_NOERROR {
            Ok(())
        } else {
            Err(ret)
        }
    }
}

impl<'img> Interface<'img> {
    pub fn name(&self) -> &str {
        let res = unsafe {
            hxcfe_getFloppyInterfaceModeName(self.img.hxcfe.as_ref().unwrap().handler, self.ifmode)
        };
        if res.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(res) }.to_str().unwrap_or("")
    }

    pub fn description(&self) -> &str {
        let res = unsafe {
            hxcfe_getFloppyInterfaceModeDesc(self.img.hxcfe.as_ref().unwrap().handler, self.ifmode)
        };
        if res.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(res) }.to_str().unwrap_or("")
    }
}
