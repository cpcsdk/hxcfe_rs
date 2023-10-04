use std::{marker::PhantomData, ffi::CStr};

use hxcfe_sys::{hxcfe_initXmlFloppy, HXCFE_XMLLDR, hxcfe_deinitXmlFloppy, hxcfe_numberOfXmlLayout, hxcfe_getXmlLayoutName, hxcfe_getXmlLayoutDesc};

use crate::Hxcfe;

pub struct LayoutManager<'hfe>{
    handler: *mut HXCFE_XMLLDR,
    phantom: PhantomData<&'hfe Hxcfe>
}

impl<'hfe> Drop for LayoutManager<'hfe> {
    fn drop(&mut self) {
        unsafe{hxcfe_deinitXmlFloppy(self.handler);}
    }
}

impl<'hfe> LayoutManager<'hfe> {
    pub fn new(hxcfe: &'hfe Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_XMLLDR = unsafe{hxcfe_initXmlFloppy(hxcfe.handler)};

		if handler.is_null() {
			None
		} else {
			Some(Self{handler, phantom: PhantomData})
		}
    }

	pub fn nb_layouts(&self) -> i32 {
		let numberofloader = unsafe{hxcfe_numberOfXmlLayout(self.handler)};
		numberofloader as _
	}

	pub fn layout_name(&self, at: i32) -> &str {
		let name = unsafe{hxcfe_getXmlLayoutName(self.handler, at)};
		unsafe{CStr::from_ptr(name)}.to_str().unwrap()
	}

	pub fn layout_description(&self, at: i32) ->  &str  {
		let desc = unsafe {
			hxcfe_getXmlLayoutDesc(self.handler, at)
		};
		unsafe{CStr::from_ptr(desc)}.to_str().unwrap()

	}
}
