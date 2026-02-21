use std::{
    ffi::{CStr, CString},
    fmt::Display,
    path::Path,
};

use hxcfe_sys::{
    HXCFE_FLOPPY, HXCFE_IMGLDR, ImageFormat, hxcfe_imgAutoSetectLoader, hxcfe_imgDeInitLoader,
    hxcfe_imgExport, hxcfe_imgGetLoaderAccess, hxcfe_imgGetLoaderDesc, hxcfe_imgGetLoaderExt,
    hxcfe_imgGetLoaderID, hxcfe_imgGetLoaderName, hxcfe_imgGetNumberOfLoader, hxcfe_imgInitLoader,
    hxcfe_imgLoad,
};

use crate::{Hxcfe, HxcfeError, img::Img};

/// Manager for floppy disk image loaders.
///
/// Provides access to various image format loaders supported by the HxC library.
pub struct ImgLoaderManager {
    handler: *mut HXCFE_IMGLDR,
    hxcfe: *const Hxcfe,
}

#[derive(enumn::N, Debug)]
#[repr(i32)]
/// Access mode for an image loader (read-only, write-only, or read-write).
pub enum ImgLoaderAccess {
    /// Read-only access
    Read = 1,
    /// Write-only access
    Write = 2,
    /// Read and write access
    ReadAndWrite = 3,
}

impl Display for ImgLoaderAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let access = match self {
            ImgLoaderAccess::Read => "R",
            ImgLoaderAccess::Write => "W",
            ImgLoaderAccess::ReadAndWrite => "RW",
        };
        write!(f, "{}", access)
    }
}

impl ImgLoaderAccess {
    pub fn readable(&self) -> bool {
        match self {
            Self::Write => false,
            _ => true,
        }
    }

    pub fn writeable(&self) -> bool {
        match self {
            Self::Read => false,
            _ => true,
        }
    }
}

/// A specific image loader for a floppy disk format.
///
/// Represents a loader that can read and/or write a specific floppy disk image format.
pub struct ImgLoader<'mngr> {
    manager: &'mngr ImgLoaderManager,
    idx: i32,
}

impl<'mngr> ImgLoader<'mngr> {
    /// Get the name identifier of this loader (e.g., "HFE", "DSK", "RAW").
    pub fn name(&self) -> &str {
        let name = unsafe { hxcfe_imgGetLoaderName(self.manager.handler, self.idx) };
        if name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("")
    }

    /// Get the file extension associated with this loader (e.g., "hfe", "dsk").
    pub fn ext(&self) -> &str {
        let ext = unsafe { hxcfe_imgGetLoaderExt(self.manager.handler, self.idx) };
        if ext.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(ext) }.to_str().unwrap_or("")
    }

    /// Get the access mode of this loader (read, write, or read-write).
    pub fn access(&self) -> ImgLoaderAccess {
        let access = unsafe { hxcfe_imgGetLoaderAccess(self.manager.handler, self.idx) };
        ImgLoaderAccess::n(access).unwrap()
    }

    /// Get a human-readable description of this loader.
    pub fn description(&self) -> &str {
        let desc = unsafe { hxcfe_imgGetLoaderDesc(self.manager.handler, self.idx) };
        if desc.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(desc) }.to_str().unwrap_or("")
    }

    /// Load a floppy disk image from a file.
    ///
    /// # Arguments
    /// * `p` - Path to the image file
    ///
    /// # Returns
    /// `Ok(Img)` on success, `Err(HxcfeError)` on failure
    ///
    /// # Errors
    /// - `HXCFE_BADPARAMETER` if the path contains null bytes
    /// - `HXCFE_ACCESSERROR` if the file cannot be accessed
    /// - `HXCFE_BADFILE` if the file format is invalid
    pub fn load<P: AsRef<Path>>(&self, p: P) -> Result<Img, HxcfeError> {
        let p = p.as_ref().display().to_string();
        let p = CString::new(p).map_err(|_| HxcfeError::HXCFE_BADPARAMETER)?;
        let p = p.into_raw();

        let mut ret: i32 = 0;
        let floppydisk: *mut HXCFE_FLOPPY =
            unsafe { hxcfe_imgLoad(self.manager.handler, p, self.idx, &mut ret) };
        let _ = unsafe { CString::from_raw(p) };

        let ret = HxcfeError::n(ret).unwrap_or(HxcfeError::HXCFE_INTERNALERROR);
        if ret != HxcfeError::HXCFE_NOERROR || floppydisk.is_null() {
            Err(ret)
        } else {
            Ok(Img {
                floppydisk,
                hxcfe: self.manager.hxcfe,
            })
        }
    }

    /// Save a floppy disk image to a file.
    ///
    /// # Arguments
    /// * `p` - Path where to save the image
    /// * `img` - The image to save
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(HxcfeError)` on failure
    ///
    /// # Errors
    /// - `HXCFE_BADPARAMETER` if the path contains null bytes
    /// - `HXCFE_ACCESSERROR` if the file cannot be written
    pub fn save<P: AsRef<Path>>(&self, p: P, img: &Img) -> Result<(), HxcfeError> {
        let p = p.as_ref().display().to_string();
        let p = CString::new(p).map_err(|_| HxcfeError::HXCFE_BADPARAMETER)?;
        let p = p.into_raw();

        let ret = unsafe { hxcfe_imgExport(self.manager.handler, img.floppydisk, p, self.idx) };

        let _ = unsafe { CString::from_raw(p) };
        let ret = HxcfeError::n(ret).unwrap_or(HxcfeError::HXCFE_INTERNALERROR);
        if ret != HxcfeError::HXCFE_NOERROR {
            Err(ret)
        } else {
            Ok(())
        }
    }
}

impl Drop for ImgLoaderManager {
    fn drop(&mut self) {
        unsafe {
            hxcfe_imgDeInitLoader(self.handler);
        }
    }
}

impl ImgLoaderManager {
    pub fn new(hxcfe: &Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_IMGLDR = unsafe { hxcfe_imgInitLoader(hxcfe.handler) };

        if handler.is_null() {
            None
        } else {
            Some(Self { handler, hxcfe })
        }
    }
    
    /// Get the internal handler pointer for use with C library functions.
    ///
    /// # Safety
    /// This exposes the raw C pointer. The pointer is only valid as long as
    /// this ImgLoaderManager instance is alive.
    pub fn handler(&self) -> *mut HXCFE_IMGLDR {
        self.handler
    }

    /// Get the total number of available image loaders.
    pub fn nb_loaders(&self) -> i32 {
        let numberofloader = unsafe { hxcfe_imgGetNumberOfLoader(self.handler) };
        numberofloader as _
    }

    fn get_loader_id_for_format(&self, format: &str) -> Option<i32> {
        let format = CString::new(format).ok()?;

        let format = format.into_raw();
        let loaderid = unsafe { hxcfe_imgGetLoaderID(self.handler, format) };
        let _ = unsafe { CString::from_raw(format) }; // ensure memory is freed;

        Some(loaderid)
    }

    /// Find a loader by format name (e.g., "HFE", "DSK").
    ///
    /// # Returns
    /// `Some(ImgLoader)` if found, `None` if the format is not supported.
    pub fn loader_for_format(&self, format: &str) -> Option<ImgLoader<'_>> {
        let idx = self.get_loader_id_for_format(format)?;
        Self::loader_for_id(self, idx)
    }

    /// Auto-detect and find the appropriate loader for a file.
    ///
    /// # Arguments
    /// * `p` - Path to the image file
    ///
    /// # Returns
    /// `Some(ImgLoader)` if a compatible loader is found, `None` otherwise.
    pub fn loader_for_fname<P: AsRef<Path>>(&self, p: P) -> Option<ImgLoader<'_>> {
        let p = p.as_ref();
        if !p.exists() {
            return None;
        }
        let p = p.display().to_string();
        let p = CString::new(p).ok()?;
        let p = p.into_raw();
        let loaderid = unsafe { hxcfe_imgAutoSetectLoader(self.handler, p, 0) };
        let _ = unsafe { CString::from_raw(p) }; // ensure memory is freed;

        self.loader_for_id(loaderid)
    }

    /// Find a loader by its text identifier.
    pub fn loader_for_text_id<'mngr>(&'mngr self, text: &str) -> Option<ImgLoader<'mngr>> {
        let p = CString::new(text).ok()?;
        let p = p.into_raw();
        let loaderid = unsafe { hxcfe_imgGetLoaderID(self.handler, p) };
        let _ = unsafe { CString::from_raw(p) }; // ensure memory is freed;
        self.loader_for_id(loaderid)
    }

    /// Get a loader by its numeric ID.
    ///
    /// # Arguments
    /// * `idx` - The loader ID (0 to nb_loaders()-1)
    ///
    /// # Returns
    /// `Some(ImgLoader)` if the ID is valid, `None` otherwise.
    pub fn loader_for_id<'mngr>(&'mngr self, idx: i32) -> Option<ImgLoader<'mngr>> {
        if idx >= 0 && idx < self.nb_loaders() {
            Some(ImgLoader { manager: self, idx })
        } else {
            None
        }
    }
}
