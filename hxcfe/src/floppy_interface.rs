use std::{ffi::CStr, marker::PhantomData};

use hxcfe_sys::{hxcfe_getFloppyInterfaceModeDesc, hxcfe_getFloppyInterfaceModeName};

use crate::Hxcfe;

pub struct FloppyInterface<'hfe> {
    hfe: &'hfe Hxcfe,
    idx: i32,
    phantom: PhantomData<&'hfe Hxcfe>,
}

impl<'hfe> FloppyInterface<'hfe> {
    pub fn new(hfe: &'hfe Hxcfe, idx: i32) -> Option<FloppyInterface<'hfe>> {
        if unsafe { hxcfe_getFloppyInterfaceModeName(hfe.handler, idx) }.is_null() {
            None
        } else {
            Some(FloppyInterface {
                hfe,
                idx,
                phantom: PhantomData,
            })
        }
    }

    pub fn name(&self) -> &str {
        let name = unsafe { hxcfe_getFloppyInterfaceModeName(self.hfe.handler, self.idx) };
        unsafe { CStr::from_ptr(name) }.to_str().unwrap()
    }

    pub fn description(&self) -> &str {
        let name = unsafe { hxcfe_getFloppyInterfaceModeDesc(self.hfe.handler, self.idx) };
        unsafe { CStr::from_ptr(name) }.to_str().unwrap()
    }
}
