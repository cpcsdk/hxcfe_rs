use std::ffi::CStr;

use std::path::Path;

use hxcfe_sys::HXCFE_FLOPPY;
use hxcfe_sys::ImageFormat;
use hxcfe_sys::hxcfe_floppyDuplicate;
use hxcfe_sys::hxcfe_floppyGetInterfaceMode;
use hxcfe_sys::hxcfe_floppySetInterfaceMode;
use hxcfe_sys::hxcfe_floppySectorBySectorCopy;
use hxcfe_sys::hxcfe_getFloppyInterfaceModeDesc;
use hxcfe_sys::hxcfe_getFloppyInterfaceModeName;
use hxcfe_sys::hxcfe_getFloppySize;
use hxcfe_sys::hxcfe_getNumberOfSide;
use hxcfe_sys::hxcfe_getNumberOfTrack;
use hxcfe_sys::hxcfe_imgDeInitLoader;
use hxcfe_sys::hxcfe_imgInitLoader;
use hxcfe_sys::hxcfe_imgUnload;

use crate::sector_access::SectorAccess;
use crate::{Hxcfe, HxcfeError, InterfaceMode};

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
    pub ifmode: InterfaceMode,
}

impl Drop for Img {
    fn drop(&mut self) {
        // Hold the lock across all three calls so no other thread can access
        // the HXCFE context between hxcfe_imgInitLoader and hxcfe_imgUnload /
        // hxcfe_imgDeInitLoader.
        let hxcfe = unsafe { self.hxcfe.as_ref().unwrap() };
        let h = hxcfe.lock_handler();
        let loader_ctx = unsafe { hxcfe_imgInitLoader(*h) };
        if !loader_ctx.is_null() {
            unsafe { hxcfe_imgUnload(loader_ctx, self.floppydisk) };
            unsafe { hxcfe_imgDeInitLoader(loader_ctx) };
        }
        // h drops here, releasing the lock
    }
}

impl Img {
    pub fn save<P: AsRef<Path>>(&self, p: P, format: ImageFormat) -> Result<(), String> {
        unsafe { self.hxcfe.as_ref().unwrap().save(p, format, self) }
    }

    /// Save the floppy disk image to a memory buffer.
    ///
    /// This is useful for WASM environments or when you need to handle
    /// the image data in memory without writing to disk.
    ///
    /// # Arguments
    /// * `format` - Output format (e.g., `ImageFormat::HxcHfe`, `ImageFormat::AmigaAdf`)
    ///
    /// # Returns
    /// `Ok(Vec<u8>)` containing the image data on success, `Err(String)` on failure.
    ///
    /// # Example
    /// ```no_run
    /// # use hxcfe::{Hxcfe, ImageFormat};
    /// let hxc = Hxcfe::get();
    /// let img = hxc.load("disk.dsk").unwrap();
    /// let hfe_data = img.save_to_buffer(ImageFormat::HxcHfe).unwrap();
    /// // hfe_data now contains the HFE format data in memory
    /// ```
    pub fn save_to_buffer(&self, format: ImageFormat) -> Result<Vec<u8>, String> {
        unsafe { self.hxcfe.as_ref().unwrap().save_to_buffer(format, self) }
    }

    pub fn interface_mode(&self) -> Option<Interface<'_>> {
        let ifmode = unsafe {
            hxcfe_floppyGetInterfaceMode(*self.hxcfe.as_ref().unwrap().lock_handler(), self.floppydisk)
        };
        let ifmode = InterfaceMode::from_i32(ifmode)?;
        Some(Interface { img: self, ifmode })
    }

    /// Set the interface mode for this floppy disk image.
    ///
    /// This affects how the image is interpreted and saved.
    ///
    /// # Arguments
    /// * `mode` - The interface mode to set
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(HxcfeError)` on failure.
    pub fn set_interface_mode(&mut self, mode: InterfaceMode) -> Result<(), HxcfeError> {
        let hxcfe = unsafe { self.hxcfe.as_ref().unwrap() };
        let h = hxcfe.lock_handler();
        let mode_id = mode.id(*h);
        let ret = unsafe {
            hxcfe_floppySetInterfaceMode(*h, self.floppydisk, mode_id)
        };
        drop(h);
        let ret = HxcfeError::n(ret).unwrap_or(HxcfeError::HXCFE_INTERNALERROR);
        if ret == HxcfeError::HXCFE_NOERROR {
            Ok(())
        } else {
            Err(ret)
        }
    }

    pub fn sector_access(&self) -> Option<SectorAccess<'_>> {
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
        unsafe { hxcfe_getNumberOfTrack(*self.hxcfe.as_ref().unwrap().lock_handler(), self.floppydisk) }
    }

    pub fn nb_tracks_per_head(&self) -> i32 {
        unsafe { self.floppydisk.as_ref().unwrap().floppyNumberOfTrack }
    }

    pub fn nb_sides(&self) -> i32 {
        unsafe { hxcfe_getNumberOfSide(*self.hxcfe.as_ref().unwrap().lock_handler(), self.floppydisk) }
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
                *self.hxcfe.as_ref().unwrap().lock_handler(),
                self.floppydisk,
                &mut nbofsector,
                &mut nbbadsector,
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
        let new_floppy =
            unsafe { hxcfe_floppyDuplicate(*self.hxcfe.as_ref().unwrap().lock_handler(), self.floppydisk) };

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
                *self.hxcfe.as_ref().unwrap().lock_handler(),
                self.floppydisk,
                src.floppydisk,
                0,
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
        let hxcfe = unsafe { self.img.hxcfe.as_ref().unwrap() };
        let h = hxcfe.lock_handler();
        let mode_id = self.ifmode.id(*h);
        let res = unsafe { hxcfe_getFloppyInterfaceModeName(*h, mode_id) };
        if res.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(res) }.to_str().unwrap_or("")
    }

    pub fn description(&self) -> &str {
        let hxcfe = unsafe { self.img.hxcfe.as_ref().unwrap() };
        let h = hxcfe.lock_handler();
        let mode_id = self.ifmode.id(*h);
        let res = unsafe { hxcfe_getFloppyInterfaceModeDesc(*h, mode_id) };
        if res.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(res) }.to_str().unwrap_or("")
    }
}
