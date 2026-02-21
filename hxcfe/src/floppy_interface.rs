use std::marker::PhantomData;

use crate::{Hxcfe, InterfaceMode};

pub struct FloppyInterface<'hfe> {
    mode: InterfaceMode,
    phantom: PhantomData<&'hfe Hxcfe>,
}

impl<'hfe> FloppyInterface<'hfe> {
    pub fn new(hfe: &'hfe Hxcfe, mode: InterfaceMode) -> Option<FloppyInterface<'hfe>> {
        // InterfaceMode is already validated at compile time, so always return Some
        Some(FloppyInterface {
            mode,
            phantom: PhantomData,
        })
    }

    pub fn mode(&self) -> InterfaceMode {
        self.mode
    }

    pub fn name(&self) -> &str {
        self.mode.mode_name()
    }

    pub fn description(&self) -> &str {
        self.mode.description()
    }
}
