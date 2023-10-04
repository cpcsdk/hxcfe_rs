use std::ffi::CStr;

use std::path::Path;

use hxcfe_sys::hxcfe_floppyGetInterfaceMode;
use hxcfe_sys::hxcfe_getFloppyInterfaceModeDesc;
use hxcfe_sys::hxcfe_getFloppyInterfaceModeName;
use hxcfe_sys::hxcfe_getFloppySize;
use hxcfe_sys::hxcfe_getNumberOfSide;
use hxcfe_sys::hxcfe_getNumberOfTrack;
use hxcfe_sys::{HXCFE_FLOPPY};

use crate::sector_access::SectorAccess;
use crate::{Hxcfe};

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
        // TODO call
        //hxcfe_imgUnload( HXCFE_IMGLDR * imgldr_ctx, HXCFE_FLOPPY * floppydisk );
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
        let mut nbofsector = 0;
        let size = unsafe {
            hxcfe_getFloppySize(
                self.hxcfe.as_ref().unwrap().handler,
                self.floppydisk,
                &mut nbofsector,
            )
        };
        size
    }

    pub fn nb_sectors(&self) -> i32 {
        let mut nbofsector = 0;
        let _ = unsafe {
            hxcfe_getFloppySize(
                self.hxcfe.as_ref().unwrap().handler,
                self.floppydisk,
                &mut nbofsector,
            )
        };
        nbofsector
    }
}

impl<'img> Interface<'img> {
    pub fn name(&self) -> &str {
        let res = unsafe {
            hxcfe_getFloppyInterfaceModeName(self.img.hxcfe.as_ref().unwrap().handler, self.ifmode)
        };
        unsafe { CStr::from_ptr(res) }.to_str().unwrap()
    }

    pub fn desc(&self) -> &str {
        let res = unsafe {
            hxcfe_getFloppyInterfaceModeDesc(self.img.hxcfe.as_ref().unwrap().handler, self.ifmode)
        };
        unsafe { CStr::from_ptr(res) }.to_str().unwrap()
    }
}
