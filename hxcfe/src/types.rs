/// Type-safe wrappers for IDs used in the public API.
///
/// These newtypes prevent accidentally mixing different ID types
/// and provide better type safety than raw i32 values.
use std::fmt;

/// Track number identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackId(i32);

impl TrackId {
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for TrackId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<TrackId> for i32 {
    fn from(id: TrackId) -> i32 {
        id.0
    }
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Head/Side identifier (typically 0 or 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeadId(i32);

impl HeadId {
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for HeadId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<HeadId> for i32 {
    fn from(id: HeadId) -> i32 {
        id.0
    }
}

impl fmt::Display for HeadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Sector identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectorId(i32);

impl SectorId {
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for SectorId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<SectorId> for i32 {
    fn from(id: SectorId) -> i32 {
        id.0
    }
}

impl fmt::Display for SectorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// USB drive identifier (0-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DriveId(i32);

impl DriveId {
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i32 {
        self.0
    }

    /// Create from u8 (validates range 0-3).
    pub const fn from_u8(id: u8) -> Option<Self> {
        if id <= 3 { Some(Self(id as i32)) } else { None }
    }
}

impl From<i32> for DriveId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<DriveId> for i32 {
    fn from(id: DriveId) -> i32 {
        id.0
    }
}

impl fmt::Display for DriveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// File handle from filesystem operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileHandle(i32);

impl FileHandle {
    pub(crate) const fn new(handle: i32) -> Self {
        Self(handle)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for FileHandle {
    fn from(handle: i32) -> Self {
        Self(handle)
    }
}

impl From<FileHandle> for i32 {
    fn from(handle: FileHandle) -> i32 {
        handle.0
    }
}

impl fmt::Display for FileHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Directory handle from filesystem operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DirHandle(i32);

impl DirHandle {
    pub(crate) const fn new(handle: i32) -> Self {
        Self(handle)
    }

    pub(crate) const fn get(self) -> i32 {
        self.0
    }
}

/// Interface mode identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterfaceModeId(i32);

impl InterfaceModeId {
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for InterfaceModeId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<InterfaceModeId> for i32 {
    fn from(id: InterfaceModeId) -> i32 {
        id.0
    }
}

impl fmt::Display for InterfaceModeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use hxcfe_sys::{
    FS_1_44MB_MSDOS_FAT12, FS_1_68MB_MSDOS_FAT12, FS_1_476MB_MSDOS_FAT12, FS_1_600MB_MSDOS_FAT12,
    FS_1_640MB_MSDOS_FAT12, FS_1_722MB_MSDOS_FAT12, FS_1_743MB_MSDOS_FAT12, FS_1_764MB_MSDOS_FAT12,
    FS_1_785MB_MSDOS_FAT12, FS_2_50MB_MSDOS_FAT12, FS_2_88MB_MSDOS_FAT12, FS_3_38MB_MSDOS_FAT12,
    FS_3_42MB_ATARI_FAT12, FS_3P5_DS_300RPM_640KB_MSDOS_FAT12, FS_4_50MB_MSDOS_FAT12,
    FS_5_35MB_B_MSDOS_FAT12, FS_5_35MB_MSDOS_FAT12, FS_5P25_300RPM_160KB_MSDOS_FAT12,
    FS_5P25_300RPM_180KB_MSDOS_FAT12, FS_5P25_300RPM_1200KB_MSDOS_FAT12,
    FS_5P25_300RPM_1230KB_MSDOS_FAT12, FS_5P25_360RPM_160KB_MSDOS_FAT12,
    FS_5P25_360RPM_180KB_MSDOS_FAT12, FS_5P25_DS_300RPM_320KB_MSDOS_FAT12,
    FS_5P25_DS_300RPM_360KB_MSDOS_FAT12, FS_5P25_DS_360RPM_320KB_MSDOS_FAT12,
    FS_5P25_DS_360RPM_360KB_MSDOS_FAT12, FS_5P25_SS_300RPM_320KB_MSDOS_FAT12,
    FS_5P25_SS_360RPM_320KB_MSDOS_FAT12, FS_6_78MB_MSDOS_FAT12, FS_16MB_MSDOS_FAT12,
    FS_360KB_ATARI_FAT12, FS_720KB_ATARI_FAT12, FS_720KB_MSDOS_FAT12, FS_738KB_MSDOS_FAT12,
    FS_800KB_MSDOS_FAT12, FS_820KB_MSDOS_FAT12, FS_880KB_AMIGADOS, FS_902KB_ATARI_FAT12,
    FS_1760KB_AMIGADOS,
};

/// Filesystem type identifier.
///
/// Corresponds to the filesystem types supported by libhxcfe.
/// These determine the format used when creating or mounting disk images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enumn::N)]
#[repr(u32)]
pub enum FileSystemId {
    // Atari formats
    Atari720KbFat12 = FS_720KB_ATARI_FAT12 as _,
    Atari902KbFat12 = FS_902KB_ATARI_FAT12 as _,
    Atari360KbFat12 = FS_360KB_ATARI_FAT12 as _,
    Atari3_42MbFat12 = FS_3_42MB_ATARI_FAT12 as _,

    // Amiga formats
    Amiga880KbDos = FS_880KB_AMIGADOS as _,
    Amiga1760KbDos = FS_1760KB_AMIGADOS as _,

    // MS-DOS 5.25" formats
    MsDos5P25_300Rpm_160KbFat12 = FS_5P25_300RPM_160KB_MSDOS_FAT12 as _,
    MsDos5P25_360Rpm_160KbFat12 = FS_5P25_360RPM_160KB_MSDOS_FAT12 as _,
    MsDos5P25_300Rpm_180KbFat12 = FS_5P25_300RPM_180KB_MSDOS_FAT12 as _,
    MsDos5P25_360Rpm_180KbFat12 = FS_5P25_360RPM_180KB_MSDOS_FAT12 as _,
    MsDos5P25Ss_300Rpm_320KbFat12 = FS_5P25_SS_300RPM_320KB_MSDOS_FAT12 as _,
    MsDos5P25Ss_360Rpm_320KbFat12 = FS_5P25_SS_360RPM_320KB_MSDOS_FAT12 as _,
    MsDos5P25Ds_300Rpm_320KbFat12 = FS_5P25_DS_300RPM_320KB_MSDOS_FAT12 as _,
    MsDos5P25Ds_360Rpm_320KbFat12 = FS_5P25_DS_360RPM_320KB_MSDOS_FAT12 as _,
    MsDos5P25Ds_300Rpm_360KbFat12 = FS_5P25_DS_300RPM_360KB_MSDOS_FAT12 as _,
    MsDos5P25Ds_360Rpm_360KbFat12 = FS_5P25_DS_360RPM_360KB_MSDOS_FAT12 as _,
    MsDos5P25_300Rpm_1200KbFat12 = FS_5P25_300RPM_1200KB_MSDOS_FAT12 as _,
    MsDos5P25_300Rpm_1230KbFat12 = FS_5P25_300RPM_1230KB_MSDOS_FAT12 as _,

    // MS-DOS 3.5" formats
    MsDos3P5Ds_300Rpm_640KbFat12 = FS_3P5_DS_300RPM_640KB_MSDOS_FAT12 as _,
    MsDos720KbFat12 = FS_720KB_MSDOS_FAT12 as _,
    MsDos738KbFat12 = FS_738KB_MSDOS_FAT12 as _,
    MsDos800KbFat12 = FS_800KB_MSDOS_FAT12 as _,
    MsDos820KbFat12 = FS_820KB_MSDOS_FAT12 as _,
    MsDos1_44MbFat12 = FS_1_44MB_MSDOS_FAT12 as _,
    MsDos1_476MbFat12 = FS_1_476MB_MSDOS_FAT12 as _,
    MsDos1_600MbFat12 = FS_1_600MB_MSDOS_FAT12 as _,
    MsDos1_640MbFat12 = FS_1_640MB_MSDOS_FAT12 as _,
    MsDos1_68MbFat12 = FS_1_68MB_MSDOS_FAT12 as _,
    MsDos1_722MbFat12 = FS_1_722MB_MSDOS_FAT12 as _,
    MsDos1_743MbFat12 = FS_1_743MB_MSDOS_FAT12 as _,
    MsDos1_764MbFat12 = FS_1_764MB_MSDOS_FAT12 as _,
    MsDos1_785MbFat12 = FS_1_785MB_MSDOS_FAT12 as _,
    MsDos2_50MbFat12 = FS_2_50MB_MSDOS_FAT12 as _,
    MsDos2_88MbFat12 = FS_2_88MB_MSDOS_FAT12 as _,
    MsDos3_38MbFat12 = FS_3_38MB_MSDOS_FAT12 as _,
    MsDos4_50MbFat12 = FS_4_50MB_MSDOS_FAT12 as _,
    MsDos5_35MbFat12 = FS_5_35MB_MSDOS_FAT12 as _,
    MsDos5_35MbBFat12 = FS_5_35MB_B_MSDOS_FAT12 as _,
    MsDos6_78MbFat12 = FS_6_78MB_MSDOS_FAT12 as _,
    MsDos16MbFat12 = FS_16MB_MSDOS_FAT12 as _,
}

impl FileSystemId {
    /// Create from raw filesystem ID, returning None if invalid.
    /// Accepts i32 for backward compatibility.
    pub fn from_i32(id: i32) -> Option<Self> {
        Self::n(id as u32)
    }

    /// Create from raw filesystem ID (u32).
    pub fn from_u32(id: u32) -> Option<Self> {
        Self::n(id)
    }

    /// Get the raw filesystem ID value as i32.
    pub const fn get(self) -> i32 {
        self as i32
    }

    /// Get the raw filesystem ID value as u32.
    pub const fn get_u32(self) -> u32 {
        self as u32
    }

    /// Get a human-readable description of this filesystem type.
    pub fn description(self) -> &'static str {
        match self {
            Self::Atari720KbFat12 => "720KB Atari FAT12",
            Self::Atari902KbFat12 => "902KB Atari FAT12",
            Self::Atari360KbFat12 => "360KB Atari FAT12",
            Self::Atari3_42MbFat12 => "3.42MB Atari FAT12",
            Self::Amiga880KbDos => "880KB AmigaDOS",
            Self::Amiga1760KbDos => "1760KB AmigaDOS",
            Self::MsDos5P25_300Rpm_160KbFat12 => "5.25\" 300RPM 160KB MS-DOS FAT12",
            Self::MsDos5P25_360Rpm_160KbFat12 => "5.25\" 360RPM 160KB MS-DOS FAT12",
            Self::MsDos5P25_300Rpm_180KbFat12 => "5.25\" 300RPM 180KB MS-DOS FAT12",
            Self::MsDos5P25_360Rpm_180KbFat12 => "5.25\" 360RPM 180KB MS-DOS FAT12",
            Self::MsDos5P25Ss_300Rpm_320KbFat12 => "5.25\" SS 300RPM 320KB MS-DOS FAT12",
            Self::MsDos5P25Ss_360Rpm_320KbFat12 => "5.25\" SS 360RPM 320KB MS-DOS FAT12",
            Self::MsDos5P25Ds_300Rpm_320KbFat12 => "5.25\" DS 300RPM 320KB MS-DOS FAT12",
            Self::MsDos5P25Ds_360Rpm_320KbFat12 => "5.25\" DS 360RPM 320KB MS-DOS FAT12",
            Self::MsDos5P25Ds_300Rpm_360KbFat12 => "5.25\" DS 300RPM 360KB MS-DOS FAT12",
            Self::MsDos5P25Ds_360Rpm_360KbFat12 => "5.25\" DS 360RPM 360KB MS-DOS FAT12",
            Self::MsDos5P25_300Rpm_1200KbFat12 => "5.25\" 300RPM 1200KB MS-DOS FAT12",
            Self::MsDos5P25_300Rpm_1230KbFat12 => "5.25\" 300RPM 1230KB MS-DOS FAT12",
            Self::MsDos3P5Ds_300Rpm_640KbFat12 => "3.5\" DS 300RPM 640KB MS-DOS FAT12",
            Self::MsDos720KbFat12 => "720KB MS-DOS FAT12",
            Self::MsDos738KbFat12 => "738KB MS-DOS FAT12",
            Self::MsDos800KbFat12 => "800KB MS-DOS FAT12",
            Self::MsDos820KbFat12 => "820KB MS-DOS FAT12",
            Self::MsDos1_44MbFat12 => "1.44MB MS-DOS FAT12",
            Self::MsDos1_476MbFat12 => "1.476MB MS-DOS FAT12",
            Self::MsDos1_600MbFat12 => "1.600MB MS-DOS FAT12",
            Self::MsDos1_640MbFat12 => "1.640MB MS-DOS FAT12",
            Self::MsDos1_68MbFat12 => "1.68MB MS-DOS FAT12",
            Self::MsDos1_722MbFat12 => "1.722MB MS-DOS FAT12",
            Self::MsDos1_743MbFat12 => "1.743MB MS-DOS FAT12",
            Self::MsDos1_764MbFat12 => "1.764MB MS-DOS FAT12",
            Self::MsDos1_785MbFat12 => "1.785MB MS-DOS FAT12",
            Self::MsDos2_50MbFat12 => "2.50MB MS-DOS FAT12",
            Self::MsDos2_88MbFat12 => "2.88MB MS-DOS FAT12",
            Self::MsDos3_38MbFat12 => "3.38MB MS-DOS FAT12",
            Self::MsDos4_50MbFat12 => "4.50MB MS-DOS FAT12",
            Self::MsDos5_35MbFat12 => "5.35MB MS-DOS FAT12",
            Self::MsDos5_35MbBFat12 => "5.35MB-B MS-DOS FAT12",
            Self::MsDos6_78MbFat12 => "6.78MB MS-DOS FAT12",
            Self::MsDos16MbFat12 => "16MB MS-DOS FAT12",
        }
    }
}

impl From<FileSystemId> for i32 {
    fn from(id: FileSystemId) -> i32 {
        id as i32
    }
}

impl From<FileSystemId> for u32 {
    fn from(id: FileSystemId) -> u32 {
        id as u32
    }
}

impl fmt::Display for FileSystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Layout index for selecting disk layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutIndex(i32);

impl LayoutIndex {
    pub const fn new(index: i32) -> Self {
        Self(index)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for LayoutIndex {
    fn from(index: i32) -> Self {
        Self(index)
    }
}

impl From<LayoutIndex> for i32 {
    fn from(index: LayoutIndex) -> i32 {
        index.0
    }
}

impl fmt::Display for LayoutIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
