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

