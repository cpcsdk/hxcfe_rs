#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Core HxCFloppyEmulator library functions
/// 
/// This module contains functions for floppy disk image manipulation,
/// including loading, saving, track/sector access, filesystem operations,
/// and format conversion.
pub mod hxcfe {
    pub use crate::{
        hxcfe_preloadImgInfos, hxcfe_imgCheckFileCompatibility, hxcfe_imgCallProgressCallback,
        hxcfe_freeSide, hxcfe_init, hxcfe_deinit, hxcfe_getVersion, hxcfe_getLicense,
        hxcfe_setEnvVar, hxcfe_getEnvVar, hxcfe_getEnvVarIndex, hxcfe_getEnvVarValue,
        hxcfe_setEnvVarValue, hxcfe_initScript, hxcfe_setScriptOutputFunc, hxcfe_execScriptFile,
        hxcfe_execScriptRam, hxcfe_execScriptLine, hxcfe_deinitScript, hxcfe_setOutputFunc,
        hxcfe_imgInitLoader, hxcfe_imgGetNumberOfLoader, hxcfe_imgGetLoaderID,
        hxcfe_imgGetLoaderAccess, hxcfe_imgGetLoaderDesc, hxcfe_imgGetLoaderName,
        hxcfe_imgGetLoaderExt, hxcfe_imgAutoSetectLoader, hxcfe_imgLoad, hxcfe_imgLoadEx,
        hxcfe_imgUnload, hxcfe_imgExport, hxcfe_imgSetProgressCallback, hxcfe_imgDeInitLoader,
        hxcfe_getNumberOfTrack, hxcfe_getNumberOfSide, hxcfe_floppyUnload, hxcfe_floppyDuplicate,
        hxcfe_floppySectorBySectorCopy, hxcfe_freeFloppy, hxcfe_initFloppy, hxcfe_setNumberOfTrack,
        hxcfe_setNumberOfSide, hxcfe_setNumberOfSector, hxcfe_setSectorSize, hxcfe_setStartSectorID,
        hxcfe_setTrackType, hxcfe_pushTrack, hxcfe_pushTrackPFS, hxcfe_setTrackInterleave,
        hxcfe_setTrackSkew, hxcfe_setSideSkew, hxcfe_setTrackPreGap, hxcfe_setIndexPosition,
        hxcfe_setIndexLength, hxcfe_setTrackBitrate, hxcfe_addSector, hxcfe_addSectors,
        hxcfe_pushSector, hxcfe_setSectorBitrate, hxcfe_setSectorGap3, hxcfe_setSectorSizeID,
        hxcfe_setSectorFill, hxcfe_setSectorTrackID, hxcfe_setSectorHeadID,
        hxcfe_setDiskSectorsHeadID, hxcfe_setSectorID, hxcfe_setSectorEncoding,
        hxcfe_setSectorDataCRC, hxcfe_setSectorHeaderCRC, hxcfe_setSectorDataMark,
        hxcfe_setSectorData, hxcfe_popSector, hxcfe_popTrack, hxcfe_setRPM,
        hxcfe_getCurrentNumberOfSector, hxcfe_getCurrentNumberOfSide, hxcfe_getCurrentNumberOfTrack,
        hxcfe_getCurrentSectorSize, hxcfe_getCurrentTrackType, hxcfe_getCurrentRPM,
        hxcfe_getCurrentSkew, hxcfe_setInterfaceMode, hxcfe_getFloppy, hxcfe_setDiskFlags,
        hxcfe_generateDisk, hxcfe_getFloppySize, hxcfe_initXmlFloppy, hxcfe_numberOfXmlLayout,
        hxcfe_getXmlLayoutID, hxcfe_getXmlLayoutDesc, hxcfe_getXmlLayoutName,
        hxcfe_selectXmlFloppyLayout, hxcfe_setXmlFloppyLayoutFile, hxcfe_generateXmlFloppy,
        hxcfe_generateXmlFileFloppy, hxcfe_foundMatchingXmlFileFloppy, hxcfe_deinitXmlFloppy,
        hxcfe_initSectorAccess, hxcfe_setSectorAccessFlags, hxcfe_getNextSector,
        hxcfe_searchSector, hxcfe_resetSearchTrackPosition, hxcfe_getAllTrackSectors,
        hxcfe_getAllTrackISOSectors, hxcfe_getSectorSize, hxcfe_getSectorData,
        hxcfe_readSectorData, hxcfe_writeSectorData, hxcfe_freeSectorConfigData,
        hxcfe_freeSectorConfig, hxcfe_clearTrackCache, hxcfe_deinitSectorAccess,
        hxcfe_initFDC, hxcfe_insertDiskFDC, hxcfe_readSectorFDC, hxcfe_writeSectorFDC,
        hxcfe_deinitFDC, hxcfe_FDC_READSECTOR, hxcfe_FDC_WRITESECTOR, hxcfe_FDC_FORMAT,
        hxcfe_FDC_SCANSECTOR, hxcfe_floppyGetSetParams, hxcfe_floppyGetInterfaceMode,
        hxcfe_floppySetInterfaceMode, hxcfe_floppyGetDoubleStep, hxcfe_floppySetDoubleStep,
        hxcfe_floppyGetFlags, hxcfe_floppySetFlags, hxcfe_getFloppyInterfaceModeID,
        hxcfe_getFloppyInterfaceModeName, hxcfe_getFloppyInterfaceModeDesc,
        hxcfe_getTrackEncodingName, hxcfe_td_init, hxcfe_td_setparams,
        hxcfe_td_activate_analyzer, hxcfe_td_get_view_mode_name, hxcfe_td_select_view_type,
        hxcfe_td_set_marker, hxcfe_td_draw_track, hxcfe_td_draw_stream_track,
        hxcfe_td_draw_trkstream, hxcfe_td_getlastsectorlist, hxcfe_td_draw_disk,
        hxcfe_td_getframebuffer, hxcfe_td_getframebuffer_xres, hxcfe_td_getframebuffer_yres,
        hxcfe_td_setProgressCallback, hxcfe_td_setName, hxcfe_td_exportToBMP, hxcfe_td_deinit,
        hxcfe_td_getlastpulselist, hxcfe_td_stream_to_sound, hxcfe_td_zoom_area,
        hxcfe_td_virt_xres, hxcfe_td_virt_yres, hxcfe_td_window_xpos, hxcfe_td_window_ypos,
        hxcfe_initFxStream, hxcfe_FxStream_setResolution, hxcfe_FxStream_setBitrate,
        hxcfe_FxStream_setPhaseCorrectionFactor, hxcfe_FxStream_setFilterParameters,
        hxcfe_FxStream_ImportStream, hxcfe_FxStream_AddIndex, hxcfe_FxStream_SetIndexLength,
        hxcfe_FxStream_AnalyzeAndGetTrack, hxcfe_FxStream_ExportToBmp, hxcfe_FxStream_FreeStream,
        hxcfe_FxStream_GetNumberOfRevolution, hxcfe_FxStream_GetRevolutionIndex,
        hxcfe_FxStream_GetRevolutionPeriod, hxcfe_FxStream_GetMeanRevolutionPeriod,
        hxcfe_FxStream_ChangeSpeed, hxcfe_FxStream_ImportHxCStreamBuffer, hxcfe_deinitFxStream,
        hxcfe_getSide, hxcfe_duplicateSide, hxcfe_replaceSide, hxcfe_getTrackBitrate,
        hxcfe_getTrackEncoding, hxcfe_getTrackLength, hxcfe_getTrackRPM,
        hxcfe_getTrackNumberOfSide, hxcfe_getHash, hxcfe_shiftTrackData, hxcfe_rotateFloppy,
        hxcfe_reverseTrackData, hxcfe_reverseFloppy, hxcfe_setTrackRPM, hxcfe_removeOddTracks,
        hxcfe_removeLastTrack, hxcfe_addTrack, hxcfe_removeTrack, hxcfe_insertTrack,
        hxcfe_deleteSide1, hxcfe_allocSide1, hxcfe_getCellState, hxcfe_setCellState,
        hxcfe_removeCell, hxcfe_insertCell, hxcfe_getCellFlakeyState, hxcfe_setCellFlakeyState,
        hxcfe_getCellIndexState, hxcfe_setCellIndexState, hxcfe_getCellBitrate,
        hxcfe_setCellBitrate, hxcfe_localRepair, hxcfe_sectorRepair,
        hxcfe_getSectorConfigEncoding, hxcfe_getSectorConfigSectorID, hxcfe_getSectorConfigSideID,
        hxcfe_getSectorConfigSizeID, hxcfe_getSectorConfigTrackID, hxcfe_getSectorConfigHCRC,
        hxcfe_getSectorConfigDCRC, hxcfe_getSectorConfigSectorSize,
        hxcfe_getSectorConfigStartSectorIndex, hxcfe_getSectorConfigStartDataIndex,
        hxcfe_getSectorConfigEndSectorIndex, hxcfe_getSectorConfigInputData,
        hxcfe_getSectorConfigDataMark, hxcfe_getSectorConfigHCRCStatus,
        hxcfe_getSectorConfigDCRCStatus, hxcfe_getFSID, hxcfe_numberOfFS, hxcfe_getFSDesc,
        hxcfe_getFSName, hxcfe_generateFloppy, hxcfe_initFsManager, hxcfe_selectFS,
        hxcfe_mountImage, hxcfe_umountImage, hxcfe_getFreeFsSpace, hxcfe_getTotalFsSpace,
        hxcfe_openDir, hxcfe_readDir, hxcfe_closeDir, hxcfe_getFirstFile, hxcfe_getNextFile,
        hxcfe_openFile, hxcfe_createFile, hxcfe_writeFile, hxcfe_readFile, hxcfe_deleteFile,
        hxcfe_closeFile, hxcfe_fseek, hxcfe_ftell, hxcfe_createDir, hxcfe_removeDir,
        hxcfe_deinitFsManager,
    };
}

/// Platform abstraction layer functions
/// 
/// This module provides cross-platform file I/O, threading, synchronization,
/// and string manipulation functions used by the HxC library.
pub mod hxcadaptor {
    pub use crate::{
        hxc_setevent, hxc_createevent, hxc_waitevent, hxc_pause, hxc_createthread,
        hxc_createcriticalsection, hxc_entercriticalsection, hxc_leavecriticalsection,
        hxc_destroycriticalsection, hxc_strupper, hxc_strlower, hxc_dyn_strcat,
        hxc_dyn_sprintfcat, hxc_open, hxc_fopen, hxc_fread, hxc_fgets, hxc_fclose,
        hxc_fgetsize, hxc_stat, hxc_find_first_file, hxc_find_next_file, hxc_find_close,
        hxc_mkdir, hxc_getcurrentdirectory, hxc_getfilenamebase, hxc_getfilenameext,
        hxc_getfilenamewext, hxc_getpathfolder, hxc_checkfileext, hxc_getfilesize,
        hxc_ram_fopen, hxc_ram_fwrite, hxc_ram_fclose,
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
        libusbhxcfe_init, libusbhxcfe_deInit, libusbhxcfe_loadFloppy, libusbhxcfe_ejectFloppy,
        libusbhxcfe_getStats, libusbhxcfe_setInterfaceMode, libusbhxcfe_setUSBBufferSize,
        libusbhxcfe_getInterfaceMode, libusbhxcfe_getDoubleStep, libusbhxcfe_getDrive,
        libusbhxcfe_getCurTrack,
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
