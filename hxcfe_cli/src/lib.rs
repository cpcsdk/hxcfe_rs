use anyhow::{Context, Result};
use clap::Parser;
#[cfg(feature = "usb")]
use hxcfe::DriveId;
use hxcfe::{DiskLayout, FileSystemId, Hxcfe, ImageFormat, InterfaceMode};
use std::path::PathBuf;

pub use hxcfe;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "hxcfe")]
#[command(version = VERSION)]
#[command(about = "HxC Floppy Emulator: Floppy image file converter", long_about = None)]
pub struct HxcfeCli {
    /// Input file image
    #[arg(short = 'i', long = "finput", value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file image
    #[arg(short = 'o', long = "foutput", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Convert the input file to specified format
    #[arg(short = 'c', long = "conv", value_name = "FORMAT")]
    convert: Option<String>,

    /// Sector to sector copy mode: specify the format reference image
    #[arg(short = 'r', long = "reffile", value_name = "FILE")]
    reffile: Option<PathBuf>,

    /// Use the specified disk layout
    #[arg(short = 'u', long = "uselayout", value_name = "LAYOUT")]
    layout: Option<String>,

    /// Select the floppy interface mode
    #[arg(long = "ifmode", value_name = "MODE")]
    interface_mode: Option<String>,

    /// Force single step mode
    #[arg(long = "singlestep")]
    single_step: bool,

    /// Force double step mode
    #[arg(long = "doublestep")]
    double_step: bool,

    /// Execute script file
    #[arg(short = 's', long = "script", value_name = "FILE")]
    script: Option<PathBuf>,

    /// Enable verbose mode
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Print license information
    #[arg(long = "license")]
    license: bool,

    /// List modules in libhxcfe (format loaders)
    #[arg(long = "modulelist")]
    module_list: bool,

    /// List disk layouts
    #[arg(long = "rawlist")]
    raw_list: bool,

    /// List floppy interface modes
    #[arg(long = "interfacelist")]
    interface_list: bool,

    /// Print information about the input file
    #[arg(long = "infos")]
    infos: bool,

    /// List the content of the floppy image
    #[arg(short = 'l', long = "list")]
    list: bool,

    /// Get a file from the floppy image
    #[arg(long = "getfile", value_name = "FILE")]
    getfile: Option<String>,

    /// Put a file to the floppy image
    #[arg(long = "putfile", value_name = "FILE")]
    putfile: Option<PathBuf>,

    /// Start the USB floppy emulator with specified drive (0-3)
    #[cfg(feature = "usb")]
    #[arg(long = "usb", value_name = "DRIVE")]
    usb: Option<u8>,
}

pub fn run(cli: &HxcfeCli) -> Result<()> {
    println!(
        "HxC Floppy Emulator : Floppy image file converter v{}",
        VERSION
    );
    println!("Copyright (C) 2006-2026 Jean-Francois DEL NERO");
    println!(
        "Rust version. Differs slightly from the original C version AND has not been deeply tested. So expect issues, report them, they will be fixed."
    );
    println!("This program comes with ABSOLUTELY NO WARRANTY");
    println!("This is free software, and you are welcome to redistribute it");
    println!("under certain conditions;\n");

    let hxc = Hxcfe::get();
    println!("libhxcfe version : {}\n", hxc.version());

    // License
    if cli.license {
        println!("License : GPL v3\n");
        return Ok(());
    }

    // Verbose mode
    if cli.verbose {
        println!("verbose mode");
    }

    // Module list
    if cli.module_list {
        print_module_list(hxc)?;
        return Ok(());
    }

    // Interface list
    if cli.interface_list {
        print_interface_list(hxc)?;
        return Ok(());
    }

    // Raw/Layout list
    if cli.raw_list {
        print_disk_layout(hxc)?;
        return Ok(());
    }

    // Script execution
    if let Some(_script) = &cli.script {
        panic!("Script execution not yet implemented in the rust version");
        // hxc.exec_script_file(script)?;
    }

    // Input file handling
    if let Some(input) = &cli.input {
        println!("Input file : {}", input.display());

        // Info command
        if cli.infos {
            print_file_info(hxc, input)?;
            return Ok(());
        }

        // List command
        if cli.list {
            list_directory(hxc, input)?;
            return Ok(());
        }

        // Get file command
        if let Some(filename) = &cli.getfile {
            get_file(hxc, input, filename)?;
            return Ok(());
        }

        // Put file command
        if let Some(file_to_put) = &cli.putfile {
            put_file(hxc, input, file_to_put)?;
            return Ok(());
        }

        // USB emulation
        #[cfg(feature = "usb")]
        if let Some(drive) = cli.usb {
            let drive_id = DriveId::from_u8(drive)
                .ok_or_else(|| anyhow::anyhow!("Invalid drive number: {} (must be 0-3)", drive))?;
            usb_load(input, drive_id, &cli)?;
            return Ok(());
        }

        // Conversion
        if let Some(format_str) = &cli.convert {
            let format = ImageFormat::from_str(format_str).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown format: {}. Use --listmodules to see available formats",
                    format_str
                )
            })?;

            let img = hxc
                .load(input)
                .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

            let output = if let Some(out) = &cli.output {
                out.clone()
            } else {
                // Generate output filename from input
                let mut output_path = input.clone();
                output_path.set_extension(format.extension());
                output_path
            };

            println!("Output file : {}", output.display());

            if let Some(reffile) = &cli.reffile {
                // Sector by sector copy mode
                sector_by_sector_copy(hxc, &img, reffile, &output, format)?;
            } else {
                // Standard conversion
                img.save(&output, format)
                    .map_err(|e| anyhow::anyhow!("Failed to save: {}", e))?;
            }

            return Ok(());
        }
    }

    // If we reach here and no command was executed, print help
    if cli.input.is_none()
        && !cli.license
        && !cli.module_list
        && !cli.interface_list
        && !cli.raw_list
        && cli.script.is_none()
    {
        println!("Use --help for usage information");
    }

    Ok(())
}

fn print_module_list(hxc: &Hxcfe) -> Result<()> {
    println!("---------------------------------------------------------------------------");
    println!("-                   libhxcfe file type support list                       -");
    println!("---------------------------------------------------------------------------");
    println!("MODULE ID          ACCESS    DESCRIPTION                         Extension\n");

    let manager = hxc
        .loaders_manager()
        .context("Failed to initialize loader manager")?;

    let nb_loaders = manager.nb_loaders();
    for i in 0..nb_loaders {
        if let Some(loader) = manager.loader_for_id(i) {
            let access = loader.access();
            println!(
                "{};{};{};*.{};",
                loader.name(),
                access,
                loader.description(),
                loader.ext()
            );
        }
    }

    println!("\n{} Loaders\n", nb_loaders);

    Ok(())
}

fn print_disk_layout(hxc: &Hxcfe) -> Result<()> {
    println!("---------------------------------------------------------------------------");
    println!("-                     libhxcfe Raw Disk Layout list                       -");
    println!("---------------------------------------------------------------------------\n");

    let layout_manager = hxc
        .layout_manager()
        .context("Failed to initialize layout manager")?;

    for layout in DiskLayout::all() {
        let name = layout_manager.layout_name(*layout);
        let desc = layout_manager.layout_description(*layout);
        println!("{:<20} :  {}", name, desc);
    }

    println!("\n{} Layout\n", DiskLayout::all().len());

    Ok(())
}

fn print_interface_list(hxc: &Hxcfe) -> Result<()> {
    println!("---------------------------------------------------------------------------");
    println!("-                        Interface mode list                              -");
    println!("---------------------------------------------------------------------------");
    println!("Interface ID                  (code)   DESCRIPTION                         \n");

    for mode in InterfaceMode::all() {
        if let Some(interface) = hxc.floppy_interface(*mode) {
            println!(
                "{:<30}(0x{:02X}) : {}",
                interface.name(),
                *mode as i32,
                interface.description()
            );
        }
    }

    println!("\n{} Modes\n", InterfaceMode::all().len());

    Ok(())
}

fn print_file_info(hxc: &Hxcfe, input: &PathBuf) -> Result<()> {
    println!("---------------------------------------------------------------------------");
    println!("-                        File informations                                -");
    println!("---------------------------------------------------------------------------");
    println!("File: {}", input.display());

    let img = hxc
        .load(input)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

    let interface_mode = img
        .interface_mode()
        .expect("Could not determine interface mode");
    println!("Interface mode : {}", interface_mode.name());

    let num_tracks = img.nb_tracks();
    let num_sides = img.nb_sides();
    println!("Number of Tracks : {}", num_tracks);
    println!("Number of Sides : {}", num_sides);

    let size = img.size();
    println!("Total Size : {} bytes", size);

    Ok(())
}

fn list_directory(hxc: &Hxcfe, input: &PathBuf) -> Result<()> {
    let img = hxc
        .load(input)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

    println!("---------------------------------------------------------------------------");
    println!("-                        Directory Listing                                -");
    println!("---------------------------------------------------------------------------");

    let fs_manager = hxc
        .file_system_manager()
        .context("Failed to initialize filesystem manager")?;

    // Try to mount the image
    let mount_ret = fs_manager.mount(&img);
    if mount_ret < 0 {
        println!("Failed to mount image (code: {})", mount_ret);
        println!("(Filesystem support may not be available for this image format)");
        return Ok(());
    }

    // Open root directory
    let dir_result = fs_manager.open_dir("/");
    match dir_result {
        Ok(dir) => {
            let mut count = 0;
            loop {
                match dir.read() {
                    Ok(entry) => {
                        let type_char = if entry.is_dir() { "d" } else { "-" };
                        println!("{} {:8} {}", type_char, entry.size(), entry.entry_name());
                        count += 1;
                    }
                    Err(_) => break, // No more entries
                }
            }
            dir.close();
            println!("\nTotal: {} entries", count);
        }
        Err(e) => {
            println!("Failed to open directory (code: {})", e);
        }
    }

    fs_manager.umount();

    Ok(())
}

fn get_file(hxc: &Hxcfe, image_path: &PathBuf, filename: &str) -> Result<()> {
    let img = hxc
        .load(image_path)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

    println!("Getting file: {}", filename);

    let fs_manager = hxc
        .file_system_manager()
        .context("Failed to initialize filesystem manager")?;

    // Select filesystem (Atari 720KB = auto-detect)
    fs_manager.select_fs(FileSystemId::Atari720KbFat12);

    let mount_ret = fs_manager.mount(&img);
    if mount_ret < 0 {
        return Err(anyhow::anyhow!("Failed to mount image"));
    }

    // Open file for reading
    let file_handle = fs_manager
        .open_file(filename)
        .map_err(|e| anyhow::anyhow!("Failed to open file: {}", e))?;

    // Read file contents
    let mut buffer = Vec::new();
    loop {
        let mut chunk = vec![0u8; 512];
        match fs_manager.read_file(file_handle, &mut chunk) {
            Ok(bytes_read) if bytes_read > 0 => {
                buffer.extend_from_slice(&chunk[..bytes_read as usize]);
            }
            _ => break,
        }
    }

    fs_manager.close_file(file_handle);
    fs_manager.umount();

    // Write to local file (strip leading "/" from filename)
    let output_name = filename.strip_prefix("/").unwrap_or(filename);
    fs_err::write(output_name, &buffer).context("Failed to write output file")?;

    println!("File extracted successfully ({} bytes)", buffer.len());

    Ok(())
}

fn put_file(hxc: &Hxcfe, image_path: &PathBuf, file_to_put: &PathBuf) -> Result<()> {
    let img = hxc
        .load(image_path)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

    let filename = file_to_put
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?;

    println!("Putting file: {}", file_to_put.display());

    let fs_manager = hxc
        .file_system_manager()
        .context("Failed to initialize filesystem manager")?;

    // Select filesystem (Atari 720KB = auto-detect)
    fs_manager.select_fs(FileSystemId::Atari720KbFat12);

    let mount_ret = fs_manager.mount(&img);
    if mount_ret < 0 {
        return Err(anyhow::anyhow!("Failed to mount image"));
    }

    // Read local file
    let contents = fs_err::read(file_to_put).context("Failed to read input file")?;

    // Create file on image (prepend "/" for absolute path)
    let fullpath = format!("/{}", filename);
    let file_handle = fs_manager
        .create_file(&fullpath)
        .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;

    // Write contents
    fs_manager
        .write_file(file_handle, &contents)
        .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;

    fs_manager.close_file(file_handle);
    fs_manager.umount();

    // Save the modified image (auto-detect format)
    let loader_manager = hxc
        .loaders_manager()
        .context("Failed to get loader manager")?;
    let loader = loader_manager
        .loader_for_fname(image_path)
        .context("Failed to detect image format")?;

    let format = ImageFormat::from_str(loader.name())
        .ok_or_else(|| anyhow::anyhow!("Unsupported format for saving: {}", loader.name()))?;

    img.save(image_path, format)
        .map_err(|e| anyhow::anyhow!("Failed to save image: {}", e))?;

    println!("File added successfully");

    Ok(())
}

fn sector_by_sector_copy(
    hxc: &Hxcfe,
    source: &hxcfe::Img,
    reffile: &PathBuf,
    output: &PathBuf,
    format: ImageFormat,
) -> Result<()> {
    println!("Sector by sector copy mode");
    println!("Reference file: {}", reffile.display());

    let reference = hxc
        .load(reffile)
        .map_err(|e| anyhow::anyhow!("Failed to load reference image: {}", e))?;

    // Duplicate the reference image
    let mut target = reference
        .duplicate()
        .map_err(|e| anyhow::anyhow!("Failed to duplicate reference image: {:?}", e))?;

    // Copy sectors from source to target
    target
        .copy_sectors_from(source)
        .map_err(|e| anyhow::anyhow!("Failed to perform sector by sector copy: {:?}", e))?;

    // Save the target
    target
        .save(output, format)
        .map_err(|e| anyhow::anyhow!("Failed to save: {}", e))?;

    println!("Sector by sector copy completed");

    Ok(())
}

#[cfg(feature = "usb")]
fn usb_load(input: &PathBuf, drive: DriveId, cli: &HxcfeCli) -> Result<()> {
    use std::io::{self, Write};

    println!("Starting USB emulation - {}", input.display());

    let hxc = Hxcfe::get();

    // Initialize USB connection
    let usb = hxcfe::UsbHxcfe::init(hxc)
        .context("Failed to initialize USB hardware. Is the HxC connected?")?;

    // Load the floppy image
    // TODO: Add support for raw layout mode with -uselayout option
    let img = hxc
        .load(input)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

    // Get or determine interface mode
    let interface_mode = if let Some(ifmode_name) = &cli.interface_mode {
        hxc.get_interface_mode_id(ifmode_name)
            .context(format!("Invalid interface mode: {}", ifmode_name))?
    } else {
        img.interface_mode()
            .ok_or_else(|| anyhow::anyhow!("Could not determine interface mode"))?
            .ifmode
    };

    // Determine double step
    let double_step = if cli.double_step {
        true
    } else if cli.single_step {
        false
    } else {
        // Auto-detect from image
        img.nb_tracks() <= 42 // Double step for <=42 tracks
    };

    // Set interface mode
    usb.set_interface_mode(interface_mode, double_step, drive)
        .map_err(|e| anyhow::anyhow!("Failed to set interface mode: {:?}", e))?;

    // Load floppy to USB hardware
    usb.load_floppy(&img)
        .map_err(|e| anyhow::anyhow!("Failed to load floppy to USB hardware: {:?}", e))?;

    if let Some(interface) = img.interface_mode() {
        println!("Interface mode : {}", interface.name());
    } else {
        println!("Interface mode : Unknown");
    }
    println!("Select line : {}", drive);
    println!("Double Step : {}", if double_step { "yes" } else { "no" });
    println!("\nFloppy image loaded to USB hardware.");
    println!("Type 'q' and press Enter to quit\n");

    // Wait for user to type 'q'
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("q") {
            break;
        }
    }

    // Eject floppy
    usb.eject_floppy()
        .map_err(|e| anyhow::anyhow!("Failed to eject floppy: {:?}", e))?;

    println!("USB emulation stopped");

    Ok(())
}
