use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::MaybeUninit,
};

use hxcfe_sys::{
    hxcfe_closeDir, hxcfe_closeFile, hxcfe_createDir, hxcfe_createFile, hxcfe_deinitFsManager,
    hxcfe_deleteFile, hxcfe_getFreeFsSpace, hxcfe_getTotalFsSpace, hxcfe_initFsManager,
    hxcfe_mountImage, hxcfe_openDir, hxcfe_openFile, hxcfe_readDir, hxcfe_readFile,
    hxcfe_removeDir, hxcfe_selectFS, hxcfe_umountImage, hxcfe_writeFile, HXCFE_FSENTRY,
    HXCFE_FSMNG,
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

    pub fn umount(&self) -> i32 {
        unsafe { hxcfe_umountImage(self.handler) }
    }

    pub fn free_space(&self) -> i32 {
        unsafe { hxcfe_getFreeFsSpace(self.handler) }
    }

    pub fn total_space(&self) -> i32 {
        unsafe { hxcfe_getTotalFsSpace(self.handler) }
    }

    pub fn open_dir(&self, folder: &str) -> Result<DirHandler<'_, '_>, i32> {
        let folder = CString::new(folder).map_err(|_| -4)?; // -4 = HXCFE_BADPARAMETER
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

    pub fn open_file(&self, filename: &str) -> Result<i32, i32> {
        let filename = CString::new(filename).map_err(|_| -4)?; // -4 = HXCFE_BADPARAMETER
        let filename = filename.into_raw();
        let filehandle = unsafe { hxcfe_openFile(self.handler, filename) };
        let _ = unsafe { CString::from_raw(filename) };

        if filehandle > 0 {
            Ok(filehandle)
        } else {
            Err(filehandle)
        }
    }

    pub fn create_file(&self, filename: &str) -> Result<i32, i32> {
        let filename = CString::new(filename).map_err(|_| -4)?;
        let filename = filename.into_raw();
        let filehandle = unsafe { hxcfe_createFile(self.handler, filename) };
        let _ = unsafe { CString::from_raw(filename) };

        if filehandle > 0 {
            Ok(filehandle)
        } else {
            Err(filehandle)
        }
    }

    pub fn read_file(&self, filehandle: i32, buffer: &mut [u8]) -> Result<i32, i32> {
        let size = buffer.len() as i32;
        let ret = unsafe { hxcfe_readFile(self.handler, filehandle, buffer.as_mut_ptr(), size) };

        if ret >= 0 {
            Ok(ret)
        } else {
            Err(ret)
        }
    }

    pub fn write_file(&self, filehandle: i32, buffer: &[u8]) -> Result<i32, i32> {
        let size = buffer.len() as i32;
        let ret = unsafe {
            hxcfe_writeFile(self.handler, filehandle, buffer.as_ptr() as *mut u8, size)
        };

        if ret >= 0 {
            Ok(ret)
        } else {
            Err(ret)
        }
    }

    pub fn close_file(&self, filehandle: i32) -> i32 {
        unsafe { hxcfe_closeFile(self.handler, filehandle) }
    }

    pub fn delete_file(&self, filename: &str) -> Result<(), i32> {
        let filename = CString::new(filename).map_err(|_| -4)?;
        let filename = filename.into_raw();
        let ret = unsafe { hxcfe_deleteFile(self.handler, filename) };
        let _ = unsafe { CString::from_raw(filename) };

        if ret >= 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    pub fn create_dir(&self, dirname: &str) -> Result<(), i32> {
        let dirname = CString::new(dirname).map_err(|_| -4)?;
        let dirname = dirname.into_raw();
        let ret = unsafe { hxcfe_createDir(self.handler, dirname) };
        let _ = unsafe { CString::from_raw(dirname) };

        if ret >= 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    pub fn remove_dir(&self, dirname: &str) -> Result<(), i32> {
        let dirname = CString::new(dirname).map_err(|_| -4)?;
        let dirname = dirname.into_raw();
        let ret = unsafe { hxcfe_removeDir(self.handler, dirname) };
        let _ = unsafe { CString::from_raw(dirname) };

        if ret >= 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }
}

impl<'hfe, 'manager> DirHandler<'hfe, 'manager> {
    pub fn read(&self) -> Result<DirEntry, i32> {
        let mut entry: HXCFE_FSENTRY = unsafe { MaybeUninit::zeroed().assume_init() };
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
