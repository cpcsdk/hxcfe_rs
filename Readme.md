# hxc_rs

This is a minimal rust wrapper over the libxcfe library <https://github.com/jfdelnero/HxCFloppyEmulator>.
I have only implemented the functionalities needed for my other project <https://github.com/cpcsdk/rust.cpclib> (mainly to allow  [basm assembler](https://cpcsdk.github.io/rust.cpclib/basm/) to write in HFE image discs).
I have not made sound choices regarding mutability: all non mutable objects on the rust-side are still mutable on the c-side.
I may have memory leaks, even if I tried to avoid them.


Feel free to provide patches to improve the cover of the wrapper, fix mistakes, or anything else.
I can gladly provide the ownership of the repository to someone more motivated than me to continue this task (I will only add what I need for my main project).

## Platform Support

- ✅ **Linux** - Full support with GNU make
- ✅ **macOS** - Full support with GNU make
- ✅ **Windows MSVC** - Full support with cc crate
- ✅ **Windows MinGW** - Full support with cc crate and static libgcc
- ✅ **WebAssembly** - Core library support (no USB hardware access)

### WebAssembly Support

The `hxcfe` and `hxcfe-sys` crates can be compiled to WebAssembly for browser-based floppy image manipulation. This enables:
- Loading and converting floppy disk images in the browser
- Filesystem operations (FAT12, AmigaDOS) in JavaScript/TypeScript
- All image loaders and format converters available in native builds

**Note**: USB hardware features are automatically disabled for WASM builds.

See [WASM_SUPPORT.md](WASM_SUPPORT.md) for detailed documentation, examples, and usage instructions.

## Supported Image Formats

The library supports a wide range of floppy disk image formats through the underlying libhxcfe:

### Common Formats
- **HFE** - HxC Floppy Emulator format (HXC_HFE, HXC_HFEV3, HXC_STREAMHFE)
- **DSK** - Amstrad CPC/Spectrum disk images (AMSTRADCPC_DSK)
- **ADF** - Amiga Disk File
- **IMG** - Raw sector images (PC floppy images)
- **D64/D81** - Commodore 64/128 disk images
- **STX** - Atari ST Pasti disk images
- **MSA** - Atari ST disk images
- **IPF** - Interchangeable Preservation Format (SPS)

### Other Supported Formats
IMD, TRD, SCL, SAP, D88, FDI, DMS, CopyQM, TeleDisk, IMZ, JVC, DMK, SCP, Kryoflux Stream, NIB, WOZ (Apple II), MFI, PRI, and many more...

Use the CLI tool `hxcfe --modulelist` to see the complete list of supported formats with read/write capabilities.

## Disc Management Features

The library provides comprehensive functionality for working with floppy disk images:

### Image Conversion
- Convert between different disk image formats
- Sector-by-sector copy with layout preservation
- Support for various disk layouts and interface modes

### File System Operations
Supported file systems:
- **FAT12** (MS-DOS, Atari ST)
- **AmigaDOS** (Amiga OFS/FFS)
- And others depending on the image format

Operations:
- **List directory** - Browse files in disk images
- **Extract files** - Get files from disk images
- **Add files** - Put files into disk images
- **Delete files** - Remove files from disk images

### Disk Information
- Query disk geometry (tracks, sides, sectors)
- Get interface mode information
- View disk layout details
- Calculate total disk size

### Advanced Features
- Multiple interface modes (Amstrad CPC, Atari ST, IBM PC, etc.)
- Custom disk layouts
- Raw track access for low-level operations
- Sector access with encoding support

## Command-Line Tool

The `hxcfe_cli` crate provides a command-line interface:

```bash
# Convert disk image formats
hxcfe -i input.dsk -o output.hfe -c HXC_HFE

# List files in a disk image
hxcfe -i disk.hfe --list

# Extract a file from disk image
hxcfe -i disk.dsk --getfile "/file.txt"

# Add a file to disk image
hxcfe -i disk.hfe --putfile localfile.txt

# Get disk information
hxcfe -i disk.dsk --infos

# List all supported formats
hxcfe --modulelist

# List available disk layouts
hxcfe --rawlist

# List interface modes
hxcfe --interfacelist
```

