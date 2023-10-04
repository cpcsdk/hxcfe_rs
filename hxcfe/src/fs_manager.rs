use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::MaybeUninit,
};

use hxcfe_sys::{
    hxcfe_closeDir, hxcfe_deinitFsManager, hxcfe_initFsManager, hxcfe_mountImage, hxcfe_openDir,
    hxcfe_readDir, hxcfe_selectFS, HXCFE_FSENTRY, HXCFE_FSMNG,
};

use crate::{img::Img, Hxcfe};

#[derive(Debug)]
pub struct FileSystemManager<'hfe> {
    handler: *mut HXCFE_FSMNG,
    phantom: PhantomData<&'hfe Hxcfe>,
}

#[derive(Debug)]
pub struct DirHandler<'hfe, 'manager> {
    dirhandle: i32,
    fs_manager: &'manager FileSystemManager<'hfe>,
}

pub struct DirEntry /*<'hfe, 'mananger, 'dir>*/ {
    entry: HXCFE_FSENTRY,
}

impl<'hfe> Drop for FileSystemManager<'hfe> {
    fn drop(&mut self) {
        unsafe { hxcfe_deinitFsManager(self.handler) };
    }
}

impl<'hfe> FileSystemManager<'hfe> {
    pub fn new(hxcfe: &'hfe Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_FSMNG = unsafe { hxcfe_initFsManager(hxcfe.handler) };

        if handler.is_null() {
            None
        } else {
            Some(Self {
                handler,
                phantom: PhantomData,
            })
        }
    }

    pub fn select_fs(&self, fs_id: i32) -> i32 {
        unsafe { hxcfe_selectFS(self.handler, fs_id) }
    }

    pub fn mount(&self, img: &Img) -> i32 {
        unsafe { hxcfe_mountImage(self.handler, img.floppydisk) }
    }

    pub fn open_dir(&self, folder: &str) -> Result<DirHandler, i32> {
        let folder = CString::new(folder).unwrap();
        let folder = folder.into_raw();
        let dirhandle = unsafe { hxcfe_openDir(self.handler, folder) };
        let _ = unsafe { CString::from_raw(folder) };

        if dirhandle > 0 {
            Ok(DirHandler {
                dirhandle,
                fs_manager: self,
            })
        } else {
            Err(dirhandle)
        }
    }
}

impl<'hfe, 'manager> DirHandler<'hfe, 'manager> {
    pub fn read(&self) -> Result<DirEntry, i32> {
        let mut entry: HXCFE_FSENTRY = unsafe { MaybeUninit::uninit().assume_init() };
        let ret = unsafe { hxcfe_readDir(self.fs_manager.handler, self.dirhandle, &mut entry) };

        if ret > 0 {
            Ok(DirEntry { entry: entry })
        } else {
            Err(ret)
        }
    }

    pub fn close(self) -> i32 {
        unsafe { hxcfe_closeDir(self.fs_manager.handler, self.dirhandle) }
    }
}

impl DirEntry {
    pub fn is_dir(&self) -> bool {
        self.entry.isdir != 0
    }

    pub fn entry_name(&self) -> &str {
        let name = unsafe { CStr::from_ptr(self.entry.entryname.as_ptr()) };
        name.to_str().unwrap()
    }

    pub fn size(&self) -> i32 {
        self.entry.size
    }
}
