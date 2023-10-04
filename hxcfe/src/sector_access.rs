use hxcfe_sys::{
    hxcfe_freeSectorConfig, hxcfe_getAllTrackSectors, hxcfe_getNextSector, hxcfe_getSectorData,
    hxcfe_getSectorSize, hxcfe_initSectorAccess, hxcfe_resetSearchTrackPosition,
    hxcfe_searchSector, hxcfe_setSectorAccessFlags, hxcfe_writeSectorData, HXCFE_SECTCFG,
    HXCFE_SECTORACCESS,
};

use crate::{Img, TrackEncoding};

pub struct SectorAccess< 'img> {
    img: &'img Img,
    access: *mut HXCFE_SECTORACCESS,
}

pub struct SectorConfig< 'access, 'img> {
    access: &'access SectorAccess<'img>,
    cfg: *mut HXCFE_SECTCFG,
	track: i32
}

impl Drop for SectorConfig<'_,  '_> {
    fn drop(&mut self) {
        unsafe { hxcfe_freeSectorConfig(self.access.access, self.cfg) }
    }
}

pub struct SectorConfigArray<'access, 'img> {
    nb_sectors: i32,
    sca: *mut *mut HXCFE_SECTCFG,
    access: &'access SectorAccess< 'img>,
	track: i32
}

impl SectorConfigArray<'_,'_> {
    pub fn nb_sectors(&self) -> i32 {
        self.nb_sectors
    }

    pub fn sector_config(&self, pos: i32) -> SectorConfig {
        assert!(pos < self.nb_sectors());
        SectorConfig {
            access: self.access,
            cfg: unsafe { *self.sca.wrapping_add(pos as usize) },
			track: self.track
        }
    }
}

impl< 'img> SectorAccess< 'img> {
    pub fn new(img: &'img Img) -> Option<Self> {
        let access = unsafe { hxcfe_initSectorAccess(img.hxcfe.as_ref().unwrap().handler, img.floppydisk) };
        if access.is_null() {
            None
        } else {
            Some(SectorAccess { img, access })
        }
    }

    pub fn set_flags(&self, flags: u32) {
        unsafe { hxcfe_setSectorAccessFlags(self.access, flags) };
    }

    pub fn get_next_sector(
        &self,
        head: i32,
        track: i32,
        r#type: TrackEncoding,
    ) -> Option<SectorConfig> {
        let sector = unsafe { hxcfe_getNextSector(self.access, track, head, r#type as _) };
        if sector.is_null() {
            None
        } else {
            Some(SectorConfig {
                access: self,
                cfg: sector,
                track,
        //        side: head,
        //        sector: None,
            })
        }
    }

    pub fn search_sector(
        &self,
        head: i32,
        track: i32,
        id: i32,
        r#type: TrackEncoding,
    ) -> Option<SectorConfig> {
        let sector = unsafe { hxcfe_searchSector(self.access, track, head, id, r#type as _) };
        if sector.is_null() {
            None
        } else {
            Some(SectorConfig {
                access: self,
                cfg: sector,
                track,
    //            side: head,
    //            sector: Some(id),
            })
        }
    }

    pub fn all_track_sectors(
        &self,
        head: i32,
        track: i32,
        r#type: TrackEncoding,
    ) -> Option<SectorConfigArray> {
        let mut nb_sectors_found = 0;
        let sca = unsafe {
            hxcfe_getAllTrackSectors(self.access, track, head, r#type as _, &mut nb_sectors_found)
        };

        if sca.is_null() {
            None
        } else {
            Some(SectorConfigArray {
				access: self,
                nb_sectors: nb_sectors_found,
                sca,
				track
            })
        }
    }

    pub fn reset_search_track_position(&self) {
        unsafe { hxcfe_resetSearchTrackPosition(self.access) };
    }
}

impl SectorConfig<'_,  '_> {

	pub fn head(&self) -> i32 {
		unsafe { self.cfg.as_ref().unwrap().head }
	}

	pub fn sector_id(&self) -> i32 {
		unsafe { self.cfg.as_ref().unwrap().sector }
	}

	pub fn sector_size(&self) -> i32 {
		unsafe { self.cfg.as_ref().unwrap().sectorsize }
	}

	pub fn sectors_left(&self) -> i32 {
		unsafe { self.cfg.as_ref().unwrap().sectorsleft }
	}

	pub fn track_encoding(&self) -> TrackEncoding {
		let encoding = unsafe { self.cfg.as_ref().unwrap().trackencoding };
		assert!(encoding >= 0);
		TrackEncoding::n(encoding as _).unwrap()
	} 

    pub fn len(&self) -> i32 {
        unsafe { hxcfe_getSectorSize(self.access.access, self.cfg) }
    }
    pub fn read(&self) -> &[u8] {
        let len = self.len();
        let data = unsafe { hxcfe_getSectorData(self.access.access, self.cfg) };
        unsafe { std::slice::from_raw_parts_mut(data, len as usize) }
    }

    /// TODO handle error (res + fdcstatus)
    pub fn write(&mut self, r#type: TrackEncoding, data: &[u8]) {
        let len = self.len();
        assert_eq!(len as usize, data.len());
        let mut fdcstatus = 0;
        let mut data = data.to_owned();


		let track = self.track;
		let side =  (unsafe { *self.cfg }).head;
		let sector =  (unsafe { *self.cfg }).sector;

        let res = unsafe {
            hxcfe_writeSectorData(
                self.access.access,
                track,
                side,
                sector,
                1,
                self.len(),
                r#type as _,
                data.as_mut_ptr(),
                &mut fdcstatus,
            )
        };
    }
}
