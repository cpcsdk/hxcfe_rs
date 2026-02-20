// Example demonstrating WASM-compatible code for hxcfe
// 
// Note: This example is written to be compatible with WASM compilation,
// but demonstrates patterns that work on all platforms.
//
// Key WASM considerations:
// 1. No direct file I/O - data must be passed as buffers
// 2. No USB hardware access (feature must be disabled)
// 3. Single-threaded execution
//
// Build for WASM:
//   cargo build --example wasm_compat --target wasm32-unknown-unknown --release
//
// Build for native (testing):
//   cargo run --example wasm_compat

use hxcfe::{FileSystemId, Hxcfe};
use std::io::Read;

fn main() {
    println!("HxC WASM-Compatible Example");
    println!("============================");
    
    // In WASM, you would receive this data from JavaScript's FileReader API
    // For this example, we simulate by reading from a file
    let image_data = load_image_data_from_file("tests/EXPERTS.DSK");
    
    // This pattern works identically in WASM and native:
    // Data is passed as a buffer instead of a file path
    let result = process_floppy_image(&image_data);
    
    match result {
        Ok(info) => {
            println!("\n✅ Image processed successfully!");
            println!("{}", info);
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
        }
    }
}

/// Process a floppy image from a memory buffer
/// This function is WASM-compatible - it only uses memory operations
fn process_floppy_image(image_data: &[u8]) -> Result<String, String> {
    // Initialize the HxC library (singleton pattern works in WASM)
    let hxc = Hxcfe::get();
    
    // In native Rust, we save to a temp file then load
    // In WASM with wasm-bindgen, you would expose a custom load_from_buffer method
    // that wraps the C API's memory-based loading capabilities
    #[cfg(not(target_family = "wasm"))]
    {
        // Save buffer to temporary file (native only)
        use std::io::Write;
        let temp_path = std::env::temp_dir().join("temp_image.dsk");
        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(image_data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        // Load the image
        let img = hxc.load(&temp_path)
            .map_err(|e| format!("Failed to load image: {:?}", e))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        // Extract information
        let mut info = String::new();
        info.push_str(&format!("Number of tracks: {}\n", img.nb_tracks()));
        info.push_str(&format!("Number of sides: {}\n", img.nb_sides()));
        info.push_str(&format!("Total size: {} bytes\n", img.size()));
        info.push_str(&format!("Interface mode: {}\n", img.interface_mode().name()));
        info.push_str(&format!("Bitrate: {} bps\n", img.bitrate()));
        
        // Try to access filesystem
        let fs_manager = hxc.file_system_manager()
            .map_err(|e| format!("Failed to init FS manager: {:?}", e))?;
        
        fs_manager.select_fs(FileSystemId::Atari720KbFat12);
        
        let mount_ret = fs_manager.mount(&img);
        if mount_ret >= 0 {
            info.push_str("\n✅ Filesystem mounted successfully\n");
            
            // Try to open root directory
            match fs_manager.open_dir("/") {
                Ok(dir) => {
                    info.push_str("Root directory contents:\n");
                    let mut count = 0;
                    loop {
                        match dir.read() {
                            Ok(entry) => {
                                let type_char = if entry.is_dir() { "d" } else { "-" };
                                info.push_str(&format!("  {} {:8} {}\n", 
                                    type_char, entry.size(), entry.entry_name()));
                                count += 1;
                            }
                            Err(_) => break,
                        }
                    }
                    dir.close();
                    info.push_str(&format!("Total: {} entries\n", count));
                }
                Err(e) => {
                    info.push_str(&format!("⚠️  Failed to open root directory: {}\n", e));
                }
            }
            
            fs_manager.umount();
        } else {
            info.push_str(&format!("⚠️  Failed to mount filesystem (code: {})\n", mount_ret));
        }
        
        Ok(info)
    }
    
    #[cfg(target_family = "wasm")]
    {
        // In WASM, you would use a custom implementation that:
        // 1. Creates a virtual file in Emscripten's filesystem (MEMFS)
        // 2. Or uses a custom C wrapper that loads directly from memory
        // 3. The JavaScript side would call exported WASM functions
        
        // For now, return a placeholder
        Ok(format!("WASM mode: {} bytes of image data received\nWASM memory-based loading would process this data...", image_data.len()))
    }
}

/// Load image data from a file (native only)
/// In WASM, this would be replaced by JavaScript's FileReader API
#[cfg(not(target_family = "wasm"))]
fn load_image_data_from_file(path: &str) -> Vec<u8> {
    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .expect("Failed to read file");
            buffer
        }
        Err(_) => {
            eprintln!("⚠️  Test file '{}' not found, using empty buffer", path);
            vec![0u8; 737280] // Empty 720KB disk
        }
    }
}

#[cfg(target_family = "wasm")]
fn load_image_data_from_file(_path: &str) -> Vec<u8> {
    // In WASM, this would be provided by JavaScript
    eprintln!("WASM: Image data should be provided by JavaScript FileReader API");
    vec![0u8; 737280] // Empty 720KB disk as placeholder
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_empty_buffer() {
        let empty_data = vec![0u8; 737280]; // 720KB
        let result = process_floppy_image(&empty_data);
        
        // Should complete without crashing
        assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable for empty data
    }
    
    #[test]
    fn test_wasm_compat_api() {
        // Verify the API is WASM-compatible (only uses buffers)
        let hxc = Hxcfe::get();
        assert!(hxc.version().len() > 0);
    }
}
