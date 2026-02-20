use crate::{DriveId, Hxcfe, HxcfeError, Img, InterfaceModeId, TrackId};
use hxcfe_sys::USBHXCFE;

/// USB HxC Floppy Emulator handle.
///
/// This struct provides access to the USB hardware interface of the HxC Floppy Emulator.
/// It allows loading and ejecting floppy disk images to/from the physical hardware.
///
/// The handle is automatically cleaned up when dropped.
pub struct UsbHxcfe {
    handler: *mut USBHXCFE,
    hxcfe: &'static Hxcfe,
}

impl UsbHxcfe {
    /// Initialize a USB connection to the HxC Floppy Emulator hardware.
    ///
    /// # Returns
    /// `Some(UsbHxcfe)` if the hardware is found and initialized successfully, `None` otherwise.
    ///
    /// # Example
    /// ```no_run
    /// # use hxcfe::{Hxcfe, UsbHxcfe};
    /// let hxcfe = Hxcfe::get();
    /// if let Some(usb) = UsbHxcfe::init(hxcfe) {
    ///     println!("USB hardware connected");
    /// } else {
    ///     println!("USB hardware not found");
    /// }
    /// ```
    pub fn init(hxcfe: &'static Hxcfe) -> Option<Self> {
        let handler = unsafe { hxcfe_sys::usbhxcfe::libusbhxcfe_init(**hxcfe) };
        if handler.is_null() {
            None
        } else {
            Some(UsbHxcfe { handler, hxcfe })
        }
    }

    /// Load a floppy disk image into the USB hardware.
    ///
    /// # Arguments
    /// * `img` - The floppy disk image to load
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(HxcfeError)` on failure.
    ///
    /// # Example
    /// ```no_run
    /// # use hxcfe::{Hxcfe, UsbHxcfe};
    /// let hxcfe = Hxcfe::get();
    /// let img = hxcfe.load("disk.hfe").unwrap();
    ///
    /// if let Some(usb) = UsbHxcfe::init(hxcfe) {
    ///     usb.load_floppy(&img).unwrap();
    ///     println!("Image loaded to USB hardware");
    /// }
    /// ```
    pub fn load_floppy(&self, img: &Img) -> Result<(), HxcfeError> {
        let ret = unsafe {
            hxcfe_sys::usbhxcfe::libusbhxcfe_loadFloppy(**self.hxcfe, self.handler, img.floppy())
        };

        if ret == 0 {
            Ok(())
        } else {
            Err(HxcfeError::HXCFE_ACCESSERROR)
        }
    }

    /// Eject the current floppy disk image from the USB hardware.
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(HxcfeError)` on failure.
    pub fn eject_floppy(&self) -> Result<(), HxcfeError> {
        let ret =
            unsafe { hxcfe_sys::usbhxcfe::libusbhxcfe_ejectFloppy(**self.hxcfe, self.handler) };

        if ret == 0 {
            Ok(())
        } else {
            Err(HxcfeError::HXCFE_ACCESSERROR)
        }
    }

    /// Set the floppy interface mode and drive parameters.
    ///
    /// # Arguments
    /// * `interface_mode` - Interface mode ID (e.g., IBMPC_DD, ATARIST_DD)
    /// * `double_step` - Enable double-step mode
    /// * `drive` - Drive select (0-3)
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(HxcfeError)` on failure.
    pub fn set_interface_mode(
        &self,
        interface_mode: InterfaceModeId,
        double_step: bool,
        drive: DriveId,
    ) -> Result<(), HxcfeError> {
        let ret = unsafe {
            hxcfe_sys::usbhxcfe::libusbhxcfe_setInterfaceMode(
                **self.hxcfe,
                self.handler,
                interface_mode.get(),
                if double_step { 1 } else { 0 },
                drive.get(),
            )
        };

        if ret == 0 {
            Ok(())
        } else {
            Err(HxcfeError::HXCFE_ACCESSERROR)
        }
    }

    /// Get the current interface mode.
    ///
    /// # Returns
    /// The interface mode ID.
    pub fn get_interface_mode(&self) -> InterfaceModeId {
        InterfaceModeId::new(unsafe {
            hxcfe_sys::usbhxcfe::libusbhxcfe_getInterfaceMode(**self.hxcfe, self.handler)
        })
    }

    /// Get the current double-step setting.
    ///
    /// # Returns
    /// `true` if double-step is enabled.
    pub fn get_double_step(&self) -> bool {
        unsafe { hxcfe_sys::usbhxcfe::libusbhxcfe_getDoubleStep(**self.hxcfe, self.handler) != 0 }
    }

    /// Get the current drive select.
    ///
    /// # Returns
    /// The drive number (0-3).
    pub fn get_drive(&self) -> DriveId {
        DriveId::new(unsafe {
            hxcfe_sys::usbhxcfe::libusbhxcfe_getDrive(**self.hxcfe, self.handler)
        })
    }

    /// Get the current track position.
    ///
    /// # Returns
    /// The track number.
    pub fn get_current_track(&self) -> TrackId {
        TrackId::new(unsafe {
            hxcfe_sys::usbhxcfe::libusbhxcfe_getCurTrack(**self.hxcfe, self.handler)
        })
    }
}

impl Drop for UsbHxcfe {
    fn drop(&mut self) {
        if !self.handler.is_null() {
            unsafe {
                hxcfe_sys::usbhxcfe::libusbhxcfe_deInit(**self.hxcfe, self.handler);
            }
        }
    }
}

unsafe impl Send for UsbHxcfe {}
unsafe impl Sync for UsbHxcfe {}
