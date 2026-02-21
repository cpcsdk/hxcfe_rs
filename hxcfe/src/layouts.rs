use std::{ffi::CStr, marker::PhantomData};

use hxcfe_sys::{
    HXCFE_XMLLDR, hxcfe_deinitXmlFloppy, hxcfe_getXmlLayoutDesc, hxcfe_getXmlLayoutName,
    hxcfe_initXmlFloppy, hxcfe_numberOfXmlLayout,
};

use crate::{DiskLayout, Hxcfe};

pub struct LayoutManager<'hfe> {
    handler: *mut HXCFE_XMLLDR,
    phantom: PhantomData<&'hfe Hxcfe>,
}

impl<'hfe> Drop for LayoutManager<'hfe> {
    fn drop(&mut self) {
        unsafe {
            hxcfe_deinitXmlFloppy(self.handler);
        }
    }
}

impl<'hfe> LayoutManager<'hfe> {
    pub fn new(hxcfe: &'hfe Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_XMLLDR = unsafe { hxcfe_initXmlFloppy(hxcfe.handler) };

        if handler.is_null() {
            None
        } else {
            Some(Self {
                handler,
                phantom: PhantomData,
            })
        }
    }

    pub fn nb_layouts(&self) -> i32 {
        let numberofloader = unsafe { hxcfe_numberOfXmlLayout(self.handler) };
        numberofloader as _
    }

    pub fn layout_name(&self, layout: DiskLayout) -> &str {
        let name = unsafe { hxcfe_getXmlLayoutName(self.handler, layout as _) };
        if name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("")
    }

    pub fn layout_description(&self, layout: DiskLayout) -> &str {
        let desc = unsafe { hxcfe_getXmlLayoutDesc(self.handler, layout as _) };
        if desc.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(desc) }.to_str().unwrap_or("")
    }
}
