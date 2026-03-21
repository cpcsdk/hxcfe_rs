use std::ffi::{CStr, CString};

use hxcfe_sys::{
    HXCFE_FSENTRY, HXCFE_FSMNG, hxcfe_closeDir, hxcfe_closeFile, hxcfe_createDir, hxcfe_createFile,
    hxcfe_deinitFsManager, hxcfe_deleteFile, hxcfe_getFreeFsSpace, hxcfe_getTotalFsSpace,
    hxcfe_initFsManager, hxcfe_mountImage, hxcfe_openDir, hxcfe_openFile, hxcfe_readDir,
    hxcfe_readFile, hxcfe_removeDir, hxcfe_selectFS, hxcfe_umountImage, hxcfe_writeFile,
};

use crate::{FileHandle, FileSystemId, Hxcfe, HxcfeError, img::Img, types::DirHandle};

/// Helper function to safely execute FFI calls that require C strings.
/// Converts a Rust string to CString, passes it to the closure, and ensures proper cleanup.
fn with_cstring<F, R>(s: &str, f: F) -> Result<R, i32>
where
    F: FnOnce(*mut i8) -> R,
{
    let cstring = CString::new(s).map_err(|_| HxcfeError::HXCFE_BADPARAMETER as i32)?;
    let raw = cstring.into_raw();
    let result = f(raw);
    let _ = unsafe { CString::from_raw(raw) };
    Ok(result)
}

#[derive(Debug)]
pub struct FileSystemManager<'hfe> {
    handler: *mut HXCFE_FSMNG,
    hxcfe: &'hfe Hxcfe,
}

#[derive(Debug)]
pub struct DirHandler<'hfe, 'manager> {
    dirhandle: DirHandle,
    fs_manager: &'manager FileSystemManager<'hfe>,
}

pub struct DirEntry /*<'hfe, 'mananger, 'dir>*/ {
    entry: HXCFE_FSENTRY,
}

impl<'hfe> Drop for FileSystemManager<'hfe> {
    fn drop(&mut self) {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_deinitFsManager(self.handler) };
    }
}

impl<'hfe> FileSystemManager<'hfe> {
    pub fn new(hxcfe: &'hfe Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_FSMNG = unsafe { hxcfe_initFsManager(*hxcfe.lock_handler()) };

        if handler.is_null() {
            None
        } else {
            Some(Self {
                handler,
                hxcfe,
            })
        }
    }

    pub fn select_fs(&self, fs_id: FileSystemId) -> i32 {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_selectFS(self.handler, fs_id.get()) }
    }

    pub fn mount(&self, img: &Img) -> i32 {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_mountImage(self.handler, img.floppydisk) }
    }

    pub fn umount(&self) -> i32 {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_umountImage(self.handler) }
    }

    pub fn free_space(&self) -> i32 {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_getFreeFsSpace(self.handler) }
    }

    pub fn total_space(&self) -> i32 {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_getTotalFsSpace(self.handler) }
    }

    pub fn open_dir(&self, folder: &str) -> Result<DirHandler<'_, '_>, i32> {
        let _h = self.hxcfe.lock_handler();
        let dirhandle = with_cstring(folder, |folder: *mut i8| unsafe {
            hxcfe_openDir(self.handler, folder)
        })?;

        if dirhandle > 0 {
            Ok(DirHandler {
                dirhandle: DirHandle::new(dirhandle),
                fs_manager: self,
            })
        } else {
            Err(dirhandle)
        }
    }

    pub fn open_file(&self, filename: &str) -> Result<FileHandle, i32> {
        let _h = self.hxcfe.lock_handler();
        let filehandle = with_cstring(filename, |filename: *mut i8| unsafe {
            hxcfe_openFile(self.handler, filename)
        })?;

        if filehandle > 0 {
            Ok(FileHandle::new(filehandle))
        } else {
            Err(filehandle)
        }
    }

    pub fn create_file(&self, filename: &str) -> Result<FileHandle, i32> {
        let _h = self.hxcfe.lock_handler();
        let filehandle = with_cstring(filename, |filename: *mut i8| unsafe {
            hxcfe_createFile(self.handler, filename)
        })?;

        if filehandle > 0 {
            Ok(FileHandle::new(filehandle))
        } else {
            Err(filehandle)
        }
    }

    pub fn read_file(&self, filehandle: FileHandle, buffer: &mut [u8]) -> Result<i32, i32> {
        let _h = self.hxcfe.lock_handler();
        let size = buffer.len() as i32;
        let ret =
            unsafe { hxcfe_readFile(self.handler, filehandle.get(), buffer.as_mut_ptr(), size) };

        if ret >= 0 { Ok(ret) } else { Err(ret) }
    }

    pub fn write_file(&self, filehandle: FileHandle, buffer: &[u8]) -> Result<i32, i32> {
        let _h = self.hxcfe.lock_handler();
        let size = buffer.len() as i32;
        let ret = unsafe {
            hxcfe_writeFile(
                self.handler,
                filehandle.get(),
                buffer.as_ptr() as *mut u8,
                size,
            )
        };

        if ret >= 0 { Ok(ret) } else { Err(ret) }
    }

    pub fn close_file(&self, filehandle: FileHandle) -> i32 {
        let _h = self.hxcfe.lock_handler();
        unsafe { hxcfe_closeFile(self.handler, filehandle.get()) }
    }

    pub fn delete_file(&self, filename: &str) -> Result<(), i32> {
        let _h = self.hxcfe.lock_handler();
        let ret = with_cstring(filename, |filename: *mut i8| unsafe {
            hxcfe_deleteFile(self.handler, filename)
        })?
        ;

        if ret >= 0 { Ok(()) } else { Err(ret) }
    }

    pub fn create_dir(&self, dirname: &str) -> Result<(), i32> {
        let _h = self.hxcfe.lock_handler();
        let ret = with_cstring(dirname, |dirname: *mut i8| unsafe {
            hxcfe_createDir(self.handler, dirname)
        })?
        ;

        if ret >= 0 { Ok(()) } else { Err(ret) }
    }

    pub fn remove_dir(&self, dirname: &str) -> Result<(), i32> {
        let _h = self.hxcfe.lock_handler();
        let ret = with_cstring(dirname, |dirname: *mut i8| unsafe {
            hxcfe_removeDir(self.handler, dirname)
        })?
        ;

        if ret >= 0 { Ok(()) } else { Err(ret) }
    }
}

impl<'hfe, 'manager> DirHandler<'hfe, 'manager> {
    pub fn read(&self) -> Result<DirEntry, i32> {
        let _h = self.fs_manager.hxcfe.lock_handler();
        let mut entry: HXCFE_FSENTRY = unsafe { std::mem::zeroed() };
        let ret =
            unsafe { hxcfe_readDir(self.fs_manager.handler, self.dirhandle.get(), &mut entry) };

        if ret > 0 {
            Ok(DirEntry { entry })
        } else {
            Err(ret)
        }
    }

    pub fn close(self) -> i32 {
        let _h = self.fs_manager.hxcfe.lock_handler();
        unsafe { hxcfe_closeDir(self.fs_manager.handler, self.dirhandle.get()) }
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
