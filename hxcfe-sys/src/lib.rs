#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![deny(clippy::as_conversions)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Include generated ImageFormat enum
include!(concat!(env!("OUT_DIR"), "/image_format.rs"));

// Include generated InterfaceMode enum
include!(concat!(env!("OUT_DIR"), "/interface_mode.rs"));

// Include generated TrackEncoding enum
include!(concat!(env!("OUT_DIR"), "/track_encoding.rs"));

// Include generated DiskLayout enum
include!(concat!(env!("OUT_DIR"), "/disk_layout.rs"));

/// Core HxCFloppyEmulator library functions
///
/// This module contains functions for floppy disk image manipulation,
/// including loading, saving, track/sector access, filesystem operations,
/// and format conversion.
pub mod hxcfe {
    pub use crate::{
        hxcfe_FDC_FORMAT, hxcfe_FDC_READSECTOR, hxcfe_FDC_SCANSECTOR, hxcfe_FDC_WRITESECTOR,
        hxcfe_FxStream_AddIndex, hxcfe_FxStream_AnalyzeAndGetTrack, hxcfe_FxStream_ChangeSpeed,
        hxcfe_FxStream_ExportToBmp, hxcfe_FxStream_FreeStream,
        hxcfe_FxStream_GetMeanRevolutionPeriod, hxcfe_FxStream_GetNumberOfRevolution,
        hxcfe_FxStream_GetRevolutionIndex, hxcfe_FxStream_GetRevolutionPeriod,
        hxcfe_FxStream_ImportHxCStreamBuffer, hxcfe_FxStream_ImportStream,
        hxcfe_FxStream_SetIndexLength, hxcfe_FxStream_setBitrate,
        hxcfe_FxStream_setFilterParameters, hxcfe_FxStream_setPhaseCorrectionFactor,
        hxcfe_FxStream_setResolution, hxcfe_addSector, hxcfe_addSectors, hxcfe_addTrack,
        hxcfe_allocSide1, hxcfe_clearTrackCache, hxcfe_closeDir, hxcfe_closeFile, hxcfe_createDir,
        hxcfe_createFile, hxcfe_deinit, hxcfe_deinitFDC, hxcfe_deinitFsManager,
        hxcfe_deinitFxStream, hxcfe_deinitScript, hxcfe_deinitSectorAccess, hxcfe_deinitXmlFloppy,
        hxcfe_deleteFile, hxcfe_deleteSide1, hxcfe_duplicateSide, hxcfe_execScriptFile,
        hxcfe_execScriptLine, hxcfe_execScriptRam, hxcfe_floppyDuplicate,
        hxcfe_floppyGetDoubleStep, hxcfe_floppyGetFlags, hxcfe_floppyGetInterfaceMode,
        hxcfe_floppyGetSetParams, hxcfe_floppySectorBySectorCopy, hxcfe_floppySetDoubleStep,
        hxcfe_floppySetFlags, hxcfe_floppySetInterfaceMode, hxcfe_floppyUnload,
        hxcfe_foundMatchingXmlFileFloppy, hxcfe_freeFloppy, hxcfe_freeSectorConfig,
        hxcfe_freeSectorConfigData, hxcfe_freeSide, hxcfe_fseek, hxcfe_ftell, hxcfe_generateDisk,
        hxcfe_generateFloppy, hxcfe_generateXmlFileFloppy, hxcfe_generateXmlFloppy,
        hxcfe_getAllTrackISOSectors, hxcfe_getAllTrackSectors, hxcfe_getCellBitrate,
        hxcfe_getCellFlakeyState, hxcfe_getCellIndexState, hxcfe_getCellState,
        hxcfe_getCurrentNumberOfSector, hxcfe_getCurrentNumberOfSide,
        hxcfe_getCurrentNumberOfTrack, hxcfe_getCurrentRPM, hxcfe_getCurrentSectorSize,
        hxcfe_getCurrentSkew, hxcfe_getCurrentTrackType, hxcfe_getEnvVar, hxcfe_getEnvVarIndex,
        hxcfe_getEnvVarValue, hxcfe_getFSDesc, hxcfe_getFSID, hxcfe_getFSName, hxcfe_getFirstFile,
        hxcfe_getFloppy, hxcfe_getFloppyInterfaceModeDesc, hxcfe_getFloppyInterfaceModeID,
        hxcfe_getFloppyInterfaceModeName, hxcfe_getFloppySize, hxcfe_getFreeFsSpace, hxcfe_getHash,
        hxcfe_getLicense, hxcfe_getNextFile, hxcfe_getNextSector, hxcfe_getNumberOfSide,
        hxcfe_getNumberOfTrack, hxcfe_getSectorConfigDCRC, hxcfe_getSectorConfigDCRCStatus,
        hxcfe_getSectorConfigDataMark, hxcfe_getSectorConfigEncoding,
        hxcfe_getSectorConfigEndSectorIndex, hxcfe_getSectorConfigHCRC,
        hxcfe_getSectorConfigHCRCStatus, hxcfe_getSectorConfigInputData,
        hxcfe_getSectorConfigSectorID, hxcfe_getSectorConfigSectorSize,
        hxcfe_getSectorConfigSideID, hxcfe_getSectorConfigSizeID,
        hxcfe_getSectorConfigStartDataIndex, hxcfe_getSectorConfigStartSectorIndex,
        hxcfe_getSectorConfigTrackID, hxcfe_getSectorData, hxcfe_getSectorSize, hxcfe_getSide,
        hxcfe_getTotalFsSpace, hxcfe_getTrackBitrate, hxcfe_getTrackEncoding,
        hxcfe_getTrackEncodingName, hxcfe_getTrackLength, hxcfe_getTrackNumberOfSide,
        hxcfe_getTrackRPM, hxcfe_getVersion, hxcfe_getXmlLayoutDesc, hxcfe_getXmlLayoutID,
        hxcfe_getXmlLayoutName, hxcfe_imgAutoSetectLoader, hxcfe_imgCallProgressCallback,
        hxcfe_imgCheckFileCompatibility, hxcfe_imgDeInitLoader, hxcfe_imgExport,
        hxcfe_imgGetLoaderAccess, hxcfe_imgGetLoaderDesc, hxcfe_imgGetLoaderExt,
        hxcfe_imgGetLoaderID, hxcfe_imgGetLoaderName, hxcfe_imgGetNumberOfLoader,
        hxcfe_imgInitLoader, hxcfe_imgLoad, hxcfe_imgLoadEx, hxcfe_imgSetProgressCallback,
        hxcfe_imgUnload, hxcfe_init, hxcfe_initFDC, hxcfe_initFloppy, hxcfe_initFsManager,
        hxcfe_initFxStream, hxcfe_initScript, hxcfe_initSectorAccess, hxcfe_initXmlFloppy,
        hxcfe_insertCell, hxcfe_insertDiskFDC, hxcfe_insertTrack, hxcfe_localRepair,
        hxcfe_mountImage, hxcfe_numberOfFS, hxcfe_numberOfXmlLayout, hxcfe_openDir, hxcfe_openFile,
        hxcfe_popSector, hxcfe_popTrack, hxcfe_preloadImgInfos, hxcfe_pushSector, hxcfe_pushTrack,
        hxcfe_pushTrackPFS, hxcfe_readDir, hxcfe_readFile, hxcfe_readSectorData,
        hxcfe_readSectorFDC, hxcfe_removeCell, hxcfe_removeDir, hxcfe_removeLastTrack,
        hxcfe_removeOddTracks, hxcfe_removeTrack, hxcfe_replaceSide,
        hxcfe_resetSearchTrackPosition, hxcfe_reverseFloppy, hxcfe_reverseTrackData,
        hxcfe_rotateFloppy, hxcfe_searchSector, hxcfe_sectorRepair, hxcfe_selectFS,
        hxcfe_selectXmlFloppyLayout, hxcfe_setCellBitrate, hxcfe_setCellFlakeyState,
        hxcfe_setCellIndexState, hxcfe_setCellState, hxcfe_setDiskFlags,
        hxcfe_setDiskSectorsHeadID, hxcfe_setEnvVar, hxcfe_setEnvVarValue, hxcfe_setIndexLength,
        hxcfe_setIndexPosition, hxcfe_setInterfaceMode, hxcfe_setNumberOfSector,
        hxcfe_setNumberOfSide, hxcfe_setNumberOfTrack, hxcfe_setOutputFunc, hxcfe_setRPM,
        hxcfe_setScriptOutputFunc, hxcfe_setSectorAccessFlags, hxcfe_setSectorBitrate,
        hxcfe_setSectorData, hxcfe_setSectorDataCRC, hxcfe_setSectorDataMark,
        hxcfe_setSectorEncoding, hxcfe_setSectorFill, hxcfe_setSectorGap3, hxcfe_setSectorHeadID,
        hxcfe_setSectorHeaderCRC, hxcfe_setSectorID, hxcfe_setSectorSize, hxcfe_setSectorSizeID,
        hxcfe_setSectorTrackID, hxcfe_setSideSkew, hxcfe_setStartSectorID, hxcfe_setTrackBitrate,
        hxcfe_setTrackInterleave, hxcfe_setTrackPreGap, hxcfe_setTrackRPM, hxcfe_setTrackSkew,
        hxcfe_setTrackType, hxcfe_setXmlFloppyLayoutFile, hxcfe_shiftTrackData,
        hxcfe_td_activate_analyzer, hxcfe_td_deinit, hxcfe_td_draw_disk,
        hxcfe_td_draw_stream_track, hxcfe_td_draw_track, hxcfe_td_draw_trkstream,
        hxcfe_td_exportToBMP, hxcfe_td_get_view_mode_name, hxcfe_td_getframebuffer,
        hxcfe_td_getframebuffer_xres, hxcfe_td_getframebuffer_yres, hxcfe_td_getlastpulselist,
        hxcfe_td_getlastsectorlist, hxcfe_td_init, hxcfe_td_select_view_type, hxcfe_td_set_marker,
        hxcfe_td_setName, hxcfe_td_setProgressCallback, hxcfe_td_setparams,
        hxcfe_td_stream_to_sound, hxcfe_td_virt_xres, hxcfe_td_virt_yres, hxcfe_td_window_xpos,
        hxcfe_td_window_ypos, hxcfe_td_zoom_area, hxcfe_umountImage, hxcfe_writeFile,
        hxcfe_writeSectorData, hxcfe_writeSectorFDC,
    };
}

/// Platform abstraction layer functions
///
/// This module provides cross-platform file I/O, threading, synchronization,
/// and string manipulation functions used by the HxC library.
pub mod hxcadaptor {
    pub use crate::{
        hxc_checkfileext, hxc_createcriticalsection, hxc_createevent, hxc_createthread,
        hxc_destroycriticalsection, hxc_dyn_sprintfcat, hxc_dyn_strcat, hxc_entercriticalsection,
        hxc_fclose, hxc_fgets, hxc_fgetsize, hxc_find_close, hxc_find_first_file,
        hxc_find_next_file, hxc_fopen, hxc_fread, hxc_getcurrentdirectory, hxc_getfilenamebase,
        hxc_getfilenameext, hxc_getfilenamewext, hxc_getfilesize, hxc_getpathfolder,
        hxc_leavecriticalsection, hxc_mkdir, hxc_open, hxc_pause, hxc_ram_fclose, hxc_ram_fopen,
        hxc_ram_fwrite, hxc_setevent, hxc_stat, hxc_strlower, hxc_strupper, hxc_waitevent,
    };
}

/// USB hardware interface functions (requires 'usb' feature)
///
/// This module provides functions to interface with physical USB floppy emulator hardware.
/// Only available when the `usb` feature is enabled.
///
/// To use this module, add to your `Cargo.toml`:
/// ```toml
/// [dependencies]
/// hxcfe-sys = { version = "*", features = ["usb"] }
/// ```
#[cfg(feature = "usb")]
pub mod usbhxcfe {
    pub use crate::{
        libusbhxcfe_deInit, libusbhxcfe_ejectFloppy, libusbhxcfe_getCurTrack,
        libusbhxcfe_getDoubleStep, libusbhxcfe_getDrive, libusbhxcfe_getInterfaceMode,
        libusbhxcfe_getStats, libusbhxcfe_init, libusbhxcfe_loadFloppy,
        libusbhxcfe_setInterfaceMode, libusbhxcfe_setUSBBufferSize,
    };
}

#[cfg(test)]
mod test {
    use crate::{hxcfe_deinit, hxcfe_init};

    #[test]
    fn nothing() {
        unsafe {
            let res = hxcfe_init();
            hxcfe_deinit(res);
        }
    }

    #[test]
    fn module_hxcfe() {
        // Test that hxcfe module exports work
        unsafe {
            let res = crate::hxcfe::hxcfe_init();
            crate::hxcfe::hxcfe_deinit(res);
        }
    }

    #[test]
    fn module_hxcadaptor() {
        // Test that hxcadaptor module exports work
        use crate::hxcadaptor::hxc_strupper;
        let _ = hxc_strupper; // Just verify it exists
    }
}
