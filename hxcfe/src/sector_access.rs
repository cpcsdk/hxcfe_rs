use hxcfe_sys::{
    HXCFE_SECTCFG, HXCFE_SECTORACCESS, hxcfe_deinitSectorAccess, hxcfe_freeSectorConfig,
    hxcfe_getAllTrackSectors, hxcfe_getNextSector, hxcfe_getSectorData, hxcfe_getSectorSize,
    hxcfe_initSectorAccess, hxcfe_resetSearchTrackPosition, hxcfe_searchSector,
    hxcfe_setSectorAccessFlags, hxcfe_writeSectorData,
};

unsafe extern "C" {
    fn free(ptr: *mut ::std::ffi::c_void);
}

use crate::{HeadId, Hxcfe, Img, SectorId, TrackEncoding, TrackId};
use std::marker::PhantomData;

pub struct SectorAccess<'img> {
    access: *mut HXCFE_SECTORACCESS,
    /// Raw pointer to the parent HXCFE context (always the 'static singleton).
    /// Used to acquire the global lock before every C call on this context.
    hxcfe: *const Hxcfe,
    _phantom: PhantomData<&'img Img>,
}

impl SectorAccess<'_> {
    fn hxcfe_ref(&self) -> &Hxcfe {
        unsafe { &*self.hxcfe }
    }
}

pub struct SectorConfig<'access, 'img> {
    access: &'access SectorAccess<'img>,
    cfg: *mut HXCFE_SECTCFG,
    track: i32,
    /// Whether this SectorConfig owns the `cfg` pointer and should free it on drop.
    /// Standalone configs (from `get_next_sector`/`search_sector`) are owned.
    /// Configs borrowed from a `SectorConfigArray` are NOT owned — the array frees them.
    owned: bool,
}

impl Drop for SectorConfig<'_, '_> {
    fn drop(&mut self) {
        if self.owned {
            let _h = self.access.hxcfe_ref().lock_handler();
            unsafe { hxcfe_freeSectorConfig(self.access.access, self.cfg) }
        }
    }
}

pub struct SectorConfigArray<'access, 'img> {
    nb_sectors: i32,
    sca: *mut *mut HXCFE_SECTCFG,
    access: &'access SectorAccess<'img>,
    track: i32,
}

impl Drop for SectorConfigArray<'_, '_> {
    fn drop(&mut self) {
        // Free each individual SECTCFG under the HXCFE lock.
        {
            let _h = self.access.hxcfe_ref().lock_handler();
            for i in 0..self.nb_sectors as usize {
                let cfg = unsafe { *self.sca.wrapping_add(i) };
                if !cfg.is_null() {
                    unsafe { hxcfe_freeSectorConfig(self.access.access, cfg) };
                }
            }
        }
        // Free the malloc'd pointer array itself (libc free, no HXCFE lock needed).
        unsafe { free(self.sca as *mut ::std::ffi::c_void) };
    }
}

impl SectorConfigArray<'_, '_> {
    pub fn nb_sectors(&self) -> i32 {
        self.nb_sectors
    }

    pub fn sector_config(&self, pos: i32) -> SectorConfig<'_, '_> {
        assert!(pos < self.nb_sectors());
        SectorConfig {
            access: self.access,
            cfg: unsafe { *self.sca.wrapping_add(pos as usize) },
            track: self.track,
            owned: false, // array owns the pointer; freeing is done in SectorConfigArray::Drop
        }
    }
}

impl Drop for SectorAccess<'_> {
    fn drop(&mut self) {
        let _h = self.hxcfe_ref().lock_handler();
        unsafe { hxcfe_deinitSectorAccess(self.access) };
    }
}

impl<'img> SectorAccess<'img> {
    pub fn new(img: &'img Img) -> Option<Self> {
        let hxcfe = img.hxcfe;
        let access = unsafe {
            hxcfe_initSectorAccess(*img.hxcfe.as_ref().unwrap().lock_handler(), img.floppydisk)
        };
        if access.is_null() {
            None
        } else {
            Some(SectorAccess {
                hxcfe,
                access,
                _phantom: PhantomData,
            })
        }
    }

    pub fn set_flags(&self, flags: u32) {
        let _h = self.hxcfe_ref().lock_handler();
        unsafe { hxcfe_setSectorAccessFlags(self.access, flags) };
    }

    pub fn get_next_sector(
        &self,
        head: HeadId,
        track: TrackId,
        r#type: TrackEncoding,
    ) -> Option<SectorConfig<'_, '_>> {
        let _h = self.hxcfe_ref().lock_handler();
        let sector =
            unsafe { hxcfe_getNextSector(self.access, track.get(), head.get(), r#type as _) };
        if sector.is_null() {
            None
        } else {
            Some(SectorConfig {
                access: self,
                cfg: sector,
                track: track.get(),
                owned: true,
            })
        }
    }

    pub fn search_sector(
        &self,
        head: HeadId,
        track: TrackId,
        id: SectorId,
        r#type: TrackEncoding,
    ) -> Option<SectorConfig<'_, '_>> {
        let _h = self.hxcfe_ref().lock_handler();
        let sector = unsafe {
            hxcfe_searchSector(self.access, track.get(), head.get(), id.get(), r#type as _)
        };
        if sector.is_null() {
            None
        } else {
            Some(SectorConfig {
                access: self,
                cfg: sector,
                track: track.get(),
                owned: true,
            })
        }
    }

    pub fn all_track_sectors(
        &self,
        head: HeadId,
        track: TrackId,
        r#type: TrackEncoding,
    ) -> Option<SectorConfigArray<'_, '_>> {
        let _h = self.hxcfe_ref().lock_handler();
        let mut nb_sectors_found = 0;
        let sca = unsafe {
            hxcfe_getAllTrackSectors(
                self.access,
                track.get(),
                head.get(),
                r#type as _,
                &mut nb_sectors_found,
            )
        };

        if sca.is_null() {
            None
        } else {
            Some(SectorConfigArray {
                access: self,
                nb_sectors: nb_sectors_found,
                sca,
                track: track.get(),
            })
        }
    }

    pub fn reset_search_track_position(&self) {
        let _h = self.hxcfe_ref().lock_handler();
        unsafe { hxcfe_resetSearchTrackPosition(self.access) };
    }
}

impl SectorConfig<'_, '_> {
    pub fn head(&self) -> HeadId {
        HeadId::new(unsafe { self.cfg.as_ref().unwrap().head })
    }

    pub fn sector_id(&self) -> SectorId {
        SectorId::new(unsafe { self.cfg.as_ref().unwrap().sector })
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
        TrackEncoding::from_u32(encoding as u32).unwrap()
    }

    pub fn len(&self) -> i32 {
        let _h = self.access.hxcfe_ref().lock_handler();
        unsafe { hxcfe_getSectorSize(self.access.access, self.cfg) }
    }

    pub fn read(&self) -> &[u8] {
        // Acquire the lock once for both C calls to avoid re-entrancy.
        let _h = self.access.hxcfe_ref().lock_handler();
        let len = unsafe { hxcfe_getSectorSize(self.access.access, self.cfg) };
        let data = unsafe { hxcfe_getSectorData(self.access.access, self.cfg) };
        if data.is_null() || len <= 0 {
            // a sector without a data field (missing data address mark)
            return &[];
        }
        unsafe { std::slice::from_raw_parts_mut(data, len as usize) }
    }

    fn cfg(&self) -> &HXCFE_SECTCFG {
        unsafe { self.cfg.as_ref().unwrap() }
    }

    /// The C (cylinder) of the sector's ID.
    pub fn cylinder(&self) -> i32 {
        self.cfg().cylinder
    }

    /// The N (size) byte of the sector's ID, as recorded.
    pub fn size_id(&self) -> u8 {
        self.cfg().alternate_sector_size_id as u8
    }

    /// The CRC of the sector's ID field is right.
    pub fn header_crc_ok(&self) -> bool {
        self.cfg().use_alternate_header_crc == 0
    }

    /// The data address mark found after the ID (0xFB normal, 0xF8 deleted on ISO/IBM tracks);
    /// None when no data field follows the ID.
    pub fn data_mark(&self) -> Option<u8> {
        let cfg = self.cfg();
        (cfg.use_alternate_datamark != 0).then_some(cfg.alternate_datamark as u8)
    }

    /// The CRC of the data field is right (meaningless without a data mark).
    pub fn data_crc_ok(&self) -> bool {
        self.cfg().use_alternate_data_crc == 0
    }

    /// Bit (cell) offset in the track of the ID field (its first sync byte).
    pub fn start_sector_index(&self) -> i32 {
        self.cfg().startsectorindex
    }

    /// Bit (cell) offset in the track of the data field (its first sync byte).
    pub fn start_data_index(&self) -> i32 {
        self.cfg().startdataindex
    }

    /// Bit (cell) offset in the track of the end of the sector.
    pub fn end_sector_index(&self) -> i32 {
        self.cfg().endsectorindex
    }

    /// Bit rate of the sector (bits per second).
    pub fn bitrate(&self) -> i32 {
        self.cfg().bitrate
    }

    /// TODO handle error (res + fdcstatus)
    pub fn write(&mut self, r#type: TrackEncoding, data: &[u8]) {
        // Acquire the lock once for all C calls in this method.
        let _h = self.access.hxcfe_ref().lock_handler();
        let len = unsafe { hxcfe_getSectorSize(self.access.access, self.cfg) };
        assert_eq!(len as usize, data.len());
        let mut fdcstatus = 0;
        let mut data = data.to_owned();

        let track = self.track;
        let side = unsafe { (*self.cfg).head };
        let sector = unsafe { (*self.cfg).sector };

        let _res = unsafe {
            hxcfe_writeSectorData(
                self.access.access,
                track,
                side,
                sector,
                1,
                len,
                r#type as _,
                data.as_mut_ptr(),
                &mut fdcstatus,
            )
        };
    }
}
