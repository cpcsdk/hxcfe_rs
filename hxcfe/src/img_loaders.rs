use std::{marker::PhantomData, ops::Deref, ffi::{CStr, CString}, path::Path, fmt::Display};

use hxcfe_sys::{HXCFE,HXCFE_IMGLDR, hxcfe_imgInitLoader, hxcfe_imgDeInitLoader, hxcfe_imgGetNumberOfLoader, hxcfe_imgGetLoaderName, hxcfe_imgGetLoaderAccess, hxcfe_imgGetLoaderDesc, hxcfe_imgGetLoaderID, hxcfe_imgAutoSetectLoader, hxcfe_imgLoad, HXCFE_FLOPPY, hxcfe_imgExport, hxcfe_imgGetLoaderExt};

use crate::{Hxcfe, img::Img, HxcfeError};

pub struct ImgLoaderManager {
    handler: *mut HXCFE_IMGLDR,
	hxcfe: *const Hxcfe,
}

#[derive(enumn::N, Debug)]
#[repr(i32)]
pub enum ImgLoaderAccess {
	Read=1,
	Write=2,
	ReadAndWrite=3
}

impl Display for ImgLoaderAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let access = match self {
            ImgLoaderAccess::Read => "R",
            ImgLoaderAccess::Write => "W",
            ImgLoaderAccess::ReadAndWrite => "RW",
        };
		write!(f, "{}", access)
    }
}

impl ImgLoaderAccess {
	pub fn readable(&self) -> bool {
		match self {
			Self::Write => false,
			_ => true
		}
	}

	pub fn writeable(&self) -> bool {
		match self {
			Self::Read => false,
			_ => true
		}
	}
}


pub struct ImgLoader<'mngr> {
    manager: &'mngr  ImgLoaderManager,
	idx: i32,

}

impl<'mngr> ImgLoader<'mngr> {
	pub fn name(&self) -> &str {
		let name = unsafe{hxcfe_imgGetLoaderName(self.manager.handler, self.idx)};
		unsafe{CStr::from_ptr(name)}.to_str().unwrap()		
	}

	pub fn ext(&self) -> &str {
		let ext = unsafe{hxcfe_imgGetLoaderExt(self.manager.handler, self.idx)};
		unsafe{CStr::from_ptr(ext)}.to_str().unwrap()	
	}

	pub fn access(&self) -> ImgLoaderAccess {
		let access = unsafe{hxcfe_imgGetLoaderAccess(self.manager.handler,self.idx)};
		ImgLoaderAccess::n(access).unwrap()
	}

	pub fn description(&self) ->  &str  {
		let desc = unsafe {
			hxcfe_imgGetLoaderDesc(self.manager.handler, self.idx)
		};
		unsafe{CStr::from_ptr(desc)}.to_str().unwrap()
	}

	pub fn load<P:AsRef<Path>>(&self, p: P) -> Result<Img, HxcfeError> {
		let p = p.as_ref().display().to_string();
		let p = CString::new(p).unwrap();
		let p = p.into_raw();

		let mut ret: i32=0;
		let floppydisk: *mut HXCFE_FLOPPY = unsafe { hxcfe_imgLoad(self.manager.handler,p,self.idx,&mut ret) };
		let _ = unsafe { CString::from_raw(p) };

		let ret = HxcfeError::n(ret).unwrap();
		if ret!=HxcfeError::HXCFE_NOERROR || floppydisk.is_null() {
			Err(ret)
		} else {
			Ok(Img{floppydisk, hxcfe: (unsafe { &*self.manager }).hxcfe})
		}
	}

	pub fn save<P:AsRef<Path>>(&self, p: P, img: &Img) -> Result<(), HxcfeError>{
		let p = p.as_ref().display().to_string();
		let p = CString::new(p).unwrap();
		let p = p.into_raw();

		let ret = unsafe { hxcfe_imgExport(self.manager.handler, img.floppydisk, p, self.idx ) };


		let _ = unsafe { CString::from_raw(p) };
		let ret = HxcfeError::n(ret).unwrap();
		if ret!=HxcfeError::HXCFE_NOERROR {
			Err(ret)
		} else {
			Ok(())
		}

	}

}


impl Drop for ImgLoaderManager {
    fn drop(&mut self) {
        unsafe{hxcfe_imgDeInitLoader(self.handler);}
    }
}

impl ImgLoaderManager {
    pub fn new(hxcfe: & Hxcfe) -> Option<Self> {
        let handler: *mut HXCFE_IMGLDR = unsafe{hxcfe_imgInitLoader(hxcfe.handler)};

		if handler.is_null() {
			None
		} else {
			Some(Self{handler, hxcfe: hxcfe })
		}
    }


	pub fn nb_loaders(&self) -> i32 {
		let numberofloader = unsafe{hxcfe_imgGetNumberOfLoader(self.handler)};
		numberofloader as _
	}





	fn get_loader_id_for_format(&self, format: &str) -> i32 {
		let format = CString::new(format).unwrap();
		
		let format = format.into_raw();
		let loaderid = unsafe { hxcfe_imgGetLoaderID(self.handler,format) };
		let _ = unsafe { CString::from_raw(format) }; // ensure memory is freed;

		loaderid
	}

	pub fn loader_for_format(&self, format: &str) -> Option<ImgLoader> {
		let idx = self.get_loader_id_for_format(format);
		Self::loader_for_id(&self, idx)
	}

	pub fn loader_for_fname<P: AsRef<Path>>(&self, p: P) -> Option<ImgLoader> {

		let p = p.as_ref();
		assert!(p.exists());
        let p = p.display().to_string();
		let p = CString::new(p).unwrap();
		let p = p.into_raw();
		let loaderid = unsafe { hxcfe_imgAutoSetectLoader(self.handler, p, 0) };
		let _ = unsafe { CString::from_raw(p) }; // ensure memory is freed;

		self.loader_for_id(loaderid)

	}

	pub fn loader_for_text_id<'mngr>(&'mngr self, text: &str) -> Option<ImgLoader<'mngr>> {
		let p = CString::new(text).unwrap();
		let p = p.into_raw();
		let loaderid = unsafe { hxcfe_imgGetLoaderID(self.handler, p) };
		let _ = unsafe { CString::from_raw(p) }; // ensure memory is freed;
		self.loader_for_id(loaderid)
	}

	pub fn loader_for_id<'mngr>(&'mngr self, idx: i32) -> Option<ImgLoader<'mngr>> {
		if idx >= 0 && idx < self.nb_loaders() {
			Some(ImgLoader { manager: self, idx})
		} else {
			None
		}
	}

}
