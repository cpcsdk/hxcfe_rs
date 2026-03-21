use std::ffi::CStr;

use hxcfe_sys::{
    HXCFE_XMLLDR, hxcfe_deinitXmlFloppy, hxcfe_getXmlLayoutDesc, hxcfe_getXmlLayoutName,
    hxcfe_initXmlFloppy, hxcfe_numberOfXmlLayout,
};

use crate::{DiskLayout, Hxcfe};

pub struct LayoutManager<'hfe> {
    handler: *mut HXCFE_XMLLDR,
    hxcfe: &'hfe Hxcfe,
}

impl<'hfe> Drop for LayoutManager<'hfe> {
    fn drop(&mut self) {
        let _h = self.hxcfe.lock_handler();
        unsafe {
            hxcfe_deinitXmlFloppy(self.handler);
        }
    }
}

impl<'hfe> LayoutManager<'hfe> {
    pub fn new(hxcfe: &'hfe Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_XMLLDR = unsafe { hxcfe_initXmlFloppy(*hxcfe.lock_handler()) };

        if handler.is_null() {
            None
        } else {
            Some(Self {
                handler,
                hxcfe,
            })
        }
    }

    pub fn nb_layouts(&self) -> i32 {
        let _h = self.hxcfe.lock_handler();
        let numberofloader = unsafe { hxcfe_numberOfXmlLayout(self.handler) };
        numberofloader as _
    }

    pub fn layout_name(&self, layout: DiskLayout) -> &str {
        let _h = self.hxcfe.lock_handler();
        let name = unsafe { hxcfe_getXmlLayoutName(self.handler, layout as _) };
        if name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("")
    }

    pub fn layout_description(&self, layout: DiskLayout) -> &str {
        let _h = self.hxcfe.lock_handler();
        let desc = unsafe { hxcfe_getXmlLayoutDesc(self.handler, layout as _) };
        if desc.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(desc) }.to_str().unwrap_or("")
    }
}
