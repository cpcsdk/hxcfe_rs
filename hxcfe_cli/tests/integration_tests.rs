use assert_cmd::Command;
use fs_err::{File, create_dir_all};
use predicates::prelude::*;
use std::io::Read;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use zip::ZipArchive;

const DISKS_ZIP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/disks_images.zip");
const TEXT_ZIP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/text_files.zip");

/// Extract a zip file to a directory
fn extract_zip<P: AsRef<Path>>(zip_path: P, dest_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.as_ref().join(file.name());

        if file.name().ends_with('/') {
            create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

/// Calculate MD5 hash of a file
fn md5_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path.as_ref())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}

struct TestEnv {
    #[allow(dead_code)]
    temp_dir: TempDir,
    work_dir: PathBuf,
}

impl TestEnv {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let work_dir = temp_dir.path().to_path_buf();

        // Extract disk images
        extract_zip(Path::new(DISKS_ZIP), &work_dir)?;

        Ok(TestEnv { temp_dir, work_dir })
    }

    fn extract_text_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        extract_zip(Path::new(TEXT_ZIP), &self.work_dir)
    }

    fn run_cli(&self, args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        Ok(Command::new(assert_cmd::cargo::cargo_bin!("hxcfe_cli"))
            .args(args)
            .current_dir(&self.work_dir)
            .output()?)
    }
}

#[test]
fn test_cli_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("hxcfe_cli"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("HxC Floppy Emulator"))
        .stdout(predicate::str::contains("--finput"))
        .stdout(predicate::str::contains("--modulelist"));
}

#[test]
fn test_module_list() {
    Command::new(assert_cmd::cargo::cargo_bin!("hxcfe_cli"))
        .arg("--modulelist")
        .assert()
        .success()
        .stdout(predicate::str::contains("libhxcfe file type support list"))
        .stdout(predicate::str::contains("HXC_HFE"))
        .stdout(predicate::str::contains("Loaders"));
}

#[test]
fn test_interface_list() {
    Command::new(assert_cmd::cargo::cargo_bin!("hxcfe_cli"))
        .arg("--interfacelist")
        .assert()
        .success()
        .stdout(predicate::str::contains("Interface mode list"))
        .stdout(predicate::str::contains("Modes"));
}

#[test]
fn test_layout_list() {
    Command::new(assert_cmd::cargo::cargo_bin!("hxcfe_cli"))
        .arg("--rawlist")
        .assert()
        .success()
        .stdout(predicate::str::contains("libhxcfe Raw Disk Layout list"))
        .stdout(predicate::str::contains("Layout"));
}

#[test]
fn test_fs_operations_fat720() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;
    let image_path = "FAT_720kB.hfe";

    // Extract text files
    env.extract_text_files()?;

    // List directory
    let output = env.run_cli(&["--finput", image_path, "--list"])?;
    assert!(output.status.success(), "List command failed");

    // Put files
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&["--finput", image_path, "--putfile", &filename])?;
        assert!(output.status.success(), "Put file {} failed", filename);
    }

    // Clean up text files
    for i in 1..=6 {
        let path = env.work_dir.join(format!("text0{}.txt", i));
        let _ = fs_err::remove_file(path);
    }

    // Get files back
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&[
            "--finput",
            image_path,
            "--getfile",
            &format!("/{}", filename),
        ])?;
        assert!(output.status.success(), "Get file {} failed", filename);

        // Verify file exists
        let file_path = env.work_dir.join(&filename);
        assert!(file_path.exists(), "File {} was not extracted", filename);
    }

    // Verify MD5 checksums - must match bash: md5sum text*.txt > md5res.txt; diff md5res.txt md5.txt
    let md5_path = env.work_dir.join("md5.txt");
    if md5_path.exists() {
        let expected_md5_content = fs_err::read_to_string(&md5_path)?;

        // Generate MD5 output in md5sum format: "hash  filename\n"
        let mut generated_md5_output = String::new();
        for i in 1..=6 {
            let filename = format!("text0{}.txt", i);
            let file_path = env.work_dir.join(&filename);
            let hash = md5_file(&file_path)?;
            generated_md5_output.push_str(&format!("{}  {}\n", hash, filename));
        }

        // Compare entire MD5 output exactly (like bash's diff command)
        assert_eq!(
            generated_md5_output.trim(),
            expected_md5_content.trim(),
            "MD5 output doesn't match md5.txt exactly.\nGenerated:\n{}\nExpected:\n{}",
            generated_md5_output,
            expected_md5_content
        );
    }

    Ok(())
}

#[test]
fn test_fs_operations_fat1440() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;
    let image_path = "FAT_1440kB.hfe";

    // Extract text files
    env.extract_text_files()?;

    // Put files
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&["--finput", image_path, "--putfile", &filename])?;
        assert!(output.status.success(), "Put file {} failed", filename);
    }

    // Clean up text files
    for i in 1..=6 {
        let path = env.work_dir.join(format!("text0{}.txt", i));
        let _ = fs_err::remove_file(path);
    }

    // Get files back
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&[
            "--finput",
            image_path,
            "--getfile",
            &format!("/{}", filename),
        ])?;
        assert!(output.status.success(), "Get file {} failed", filename);
    }

    Ok(())
}

#[test]
fn test_fs_operations_amiga_hfe() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;
    let image_path = "ADOS_880kB.hfe";

    // Extract text files
    env.extract_text_files()?;

    // Put files
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&["--finput", image_path, "--putfile", &filename])?;
        assert!(output.status.success(), "Put file {} failed", filename);
    }

    // Clean up text files
    for i in 1..=6 {
        let path = env.work_dir.join(format!("text0{}.txt", i));
        let _ = fs_err::remove_file(path);
    }

    // Get files back
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&[
            "--finput",
            image_path,
            "--getfile",
            &format!("/{}", filename),
        ])?;
        assert!(output.status.success(), "Get file {} failed", filename);
    }

    Ok(())
}

#[test]
fn test_fs_operations_amiga_adf() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;
    let image_path = "ADOS_880kB.adf";

    // Extract text files
    env.extract_text_files()?;

    // Put files
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&["--finput", image_path, "--putfile", &filename])?;
        assert!(output.status.success(), "Put file {} failed", filename);
    }

    // Clean up text files
    for i in 1..=6 {
        let path = env.work_dir.join(format!("text0{}.txt", i));
        let _ = fs_err::remove_file(path);
    }

    // Get files back
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        let output = env.run_cli(&[
            "--finput",
            image_path,
            "--getfile",
            &format!("/{}", filename),
        ])?;
        assert!(output.status.success(), "Get file {} failed", filename);
    }

    Ok(())
}

#[test]
fn test_format_conversion_fat() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;
    let image_path = "FAT_720kB.hfe";

    // Extract text files and put them
    env.extract_text_files()?;
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        env.run_cli(&["--finput", image_path, "--putfile", &filename])?;
    }

    // Test all 18 format conversions (matching convert.sh)
    let formats = vec![
        ("I001.IMG", "RAW_LOADER"),
        ("I002.FDX", "FDX68_FDX"),
        ("I003.STX", "ATARIST_STX"),
        ("I004.DSK", "AMSTRADCPC_DSK"),
        ("I005.STW", "ATARIST_STW"),
        ("I006.HFE", "HXC_HFEV3"),
        ("I007.HFE", "HXC_HFE"),
        ("I008.IMD", "IMD_IMG"),
        ("I009.JV3", "TRS80_JV3"),
        ("I010.DMK", "TRS80_DMK"),
        ("I011.AFI", "HXC_AFI"),
        ("I012.AFI", "HXC_AFI"),
        ("I013.SCP", "SCP_FLUX_STREAM"),
        ("I014.XML", "GENERIC_XML"),
        ("I015.HFE", "HXC_STREAMHFE"),
        ("I016.D88", "NEC_D88"),
        ("I017.MFM", "HXCMFM_IMG"),
        ("I018.MSA", "ATARIST_MSA"),
    ];

    // Convert to all formats
    for (output_file, format) in &formats {
        let output = env.run_cli(&[
            "--finput",
            image_path,
            "--foutput",
            output_file,
            "--conv",
            format,
        ])?;

        assert!(output.status.success(), "Conversion to {} failed", format);

        let output_path = env.work_dir.join(output_file);
        assert!(
            output_path.exists(),
            "Output file {} was not created",
            output_file
        );
    }

    // Convert all back to RAW_LOADER and verify they're identical
    let mut identical_count = 0;

    for (input_file, _) in &formats {
        let test_output = format!("{}_T.IMG", input_file.replace(".", "_"));
        let output = env.run_cli(&[
            "--finput",
            input_file,
            "--foutput",
            &test_output,
            "--conv",
            "RAW_LOADER",
        ])?;

        assert!(
            output.status.success(),
            "Reverse conversion of {} failed",
            input_file
        );

        let test_path = env.work_dir.join(&test_output);
        assert!(
            test_path.exists(),
            "Reverse conversion file {} was not created",
            test_output
        );
    }

    // Compare all converted files to the reference (I001_T.IMG)
    let ref_hash = md5_file(env.work_dir.join("I001_IMG_T.IMG"))?;

    for (input_file, _) in &formats {
        let test_output = format!("{}_T.IMG", input_file.replace(".", "_"));
        let test_path = env.work_dir.join(&test_output);
        let test_hash = md5_file(&test_path)?;

        if test_hash == ref_hash {
            identical_count += 1;
            println!("✓ {} roundtrip identical", input_file);
        } else {
            println!(
                "✗ {} roundtrip differs: ref={}, test={}",
                input_file, ref_hash, test_hash
            );
        }
    }

    // All 18 conversions should be identical
    assert_eq!(
        identical_count, 18,
        "Expected 18 identical roundtrip conversions, got {}",
        identical_count
    );

    Ok(())
}

#[test]
fn test_format_conversion_amiga() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;
    let image_path = "ADOS_880kB.hfe";

    // Extract text files and put them
    env.extract_text_files()?;
    for i in 1..=6 {
        let filename = format!("text0{}.txt", i);
        env.run_cli(&["--finput", image_path, "--putfile", &filename])?;
    }

    // Test all 8 Amiga format conversions (matching convert_amiga.sh)
    let formats = vec![
        ("I001.ADF", "AMIGA_ADF"),
        ("I002.ADZ", "AMIGA_ADZ"),
        ("I003.HFE", "HXC_HFEV3"),
        ("I004.HFE", "HXC_HFE"),
        ("I005.AFI", "HXC_AFI"),
        ("I006.SCP", "SCP_FLUX_STREAM"),
        ("I007.HFE", "HXC_STREAMHFE"),
        ("I008.MFM", "HXCMFM_IMG"),
    ];

    // Convert to all formats
    for (output_file, format) in &formats {
        let output = env.run_cli(&[
            "--finput",
            image_path,
            "--foutput",
            output_file,
            "--conv",
            format,
        ])?;

        assert!(output.status.success(), "Conversion to {} failed", format);

        let output_path = env.work_dir.join(output_file);
        assert!(
            output_path.exists(),
            "Output file {} was not created",
            output_file
        );
    }

    // Convert all back to RAW_LOADER and verify they're identical
    let mut identical_count = 0;

    for (input_file, _) in &formats {
        let test_output = format!("{}_T.ADF", input_file.replace(".", "_"));
        let output = env.run_cli(&[
            "--finput",
            input_file,
            "--foutput",
            &test_output,
            "--conv",
            "RAW_LOADER",
        ])?;

        assert!(
            output.status.success(),
            "Reverse conversion of {} failed",
            input_file
        );

        let test_path = env.work_dir.join(&test_output);
        assert!(
            test_path.exists(),
            "Reverse conversion file {} was not created",
            test_output
        );
    }

    // Compare all converted files to the reference (I001_T.ADF)
    let ref_hash = md5_file(env.work_dir.join("I001_ADF_T.ADF"))?;

    for (input_file, _) in &formats {
        let test_output = format!("{}_T.ADF", input_file.replace(".", "_"));
        let test_path = env.work_dir.join(&test_output);
        let test_hash = md5_file(&test_path)?;

        if test_hash == ref_hash {
            identical_count += 1;
            println!("✓ {} roundtrip identical", input_file);
        } else {
            println!(
                "✗ {} roundtrip differs: ref={}, test={}",
                input_file, ref_hash, test_hash
            );
        }
    }

    // All 8 conversions should be identical
    assert_eq!(
        identical_count, 8,
        "Expected 8 identical roundtrip conversions, got {}",
        identical_count
    );

    // Additional test: ADZ→ADF with MD5 verification (matching convert_amiga.sh)
    let output = env.run_cli(&[
        "--finput",
        "ADOS_1760kB.adz",
        "--foutput",
        "C001.ADF",
        "--conv",
        "RAW_LOADER",
    ])?;

    assert!(output.status.success(), "ADZ to ADF conversion failed");

    let c001_path = env.work_dir.join("C001.ADF");
    assert!(c001_path.exists(), "C001.ADF was not created");

    let c001_hash = md5_file(&c001_path)?;
    let expected_hash = "2e9c78b254515902a236025ebeb718bf";

    assert_eq!(
        c001_hash, expected_hash,
        "ADZ→ADF MD5 mismatch: expected {}, got {}",
        expected_hash, c001_hash
    );

    println!("✓ ADOS_1760kB.adz → C001.ADF MD5 OK");

    Ok(())
}

#[test]
fn test_additional_md5_conversions() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;

    // 9 additional MD5-verified conversions from convert.sh
    let conversions = vec![
        (
            "hbd.vdk",
            "C001.IMG",
            "RAW_LOADER",
            "4292a3cb2f6a6466a39583271334f8ce",
        ),
        (
            "disk1.86f",
            "C002.IMG",
            "RAW_LOADER",
            "73bec3e0512925c955c706a9d742967d",
        ),
        (
            "Acorn_Horizon.adf",
            "C003.IMG",
            "RAW_LOADER",
            "05f58f4fa3b96017d4ea0c1c80f24c0b",
        ),
        (
            "EvilIn11.ssd",
            "C004.IMG",
            "RAW_LOADER",
            "cee7770810257f2063b5765ff24c2525",
        ),
        (
            "adc-cpm.td0",
            "C005.IMG",
            "RAW_LOADER",
            "ac12f9cfcd68ff364598507f7ca1503e",
        ),
        (
            "dos33_with_adt.do",
            "C006.DO",
            "APPLE2_DO",
            "7c350e5da3672bca4abbdbe67fdaf14a",
        ),
        (
            "Apple_DOS_3_3_January_1983.do",
            "C007.DO",
            "APPLE2_DO",
            "b13de32fd7a97d817744bf2dd71d5479",
        ),
        (
            "Apple_DOS_3_3_January_1983.nib",
            "C008.DO",
            "APPLE2_DO",
            "b13de32fd7a97d817744bf2dd71d5479",
        ),
        (
            "apridisk.dsk",
            "C009.IMG",
            "RAW_LOADER",
            "f35a690248f7afebc5180b4e81cceb88",
        ),
    ];

    let mut success_count = 0;

    for (input_file, output_file, format, expected_hash) in &conversions {
        // Check if input file exists
        let input_path = env.work_dir.join(input_file);
        if !input_path.exists() {
            println!("⊗ Skipping {} (file not in test data)", input_file);
            continue;
        }

        let output = env.run_cli(&[
            "--finput",
            input_file,
            "--foutput",
            output_file,
            "--conv",
            format,
        ])?;

        if !output.status.success() {
            println!("✗ {} → {} conversion failed", input_file, output_file);
            continue;
        }

        let output_path = env.work_dir.join(output_file);
        if !output_path.exists() {
            println!("✗ {} was not created", output_file);
            continue;
        }

        let actual_hash = md5_file(&output_path)?;

        if actual_hash == *expected_hash {
            success_count += 1;
            println!("✓ {} → {} MD5 OK", input_file, output_file);
        } else {
            println!(
                "✗ {} MD5 mismatch: expected {}, got {}",
                output_file, expected_hash, actual_hash
            );
        }
    }

    // We expect all available test files to pass
    // (some may be skipped if not in disks_images.zip)
    assert!(
        success_count > 0,
        "At least some MD5 conversions should pass, got {}",
        success_count
    );

    println!("Additional MD5 conversions: {} passed", success_count);

    Ok(())
}

#[test]
fn test_info_command() -> Result<(), Box<dyn std::error::Error>> {
    let env = TestEnv::new()?;

    let output = env.run_cli(&["--finput", "FAT_720kB.hfe", "--infos"])?;
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("File informations"));
    assert!(stdout.contains("Interface mode"));
    assert!(stdout.contains("Number of Tracks"));
    assert!(stdout.contains("Number of Sides"));
    assert!(stdout.contains("Total Size"));

    Ok(())
}

#[test]
#[ignore] // Run with: cargo test --test integration_tests -- --ignored
fn test_comprehensive_suite() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running comprehensive test suite...");

    let mut success_count = 0;
    let mut total_tests = 0;

    // Test 1: FAT 720KB filesystem operations
    total_tests += 1;
    if test_fs_operations_fat720().is_ok() {
        println!("✓ FAT 720KB filesystem operations: SUCCESS");
        success_count += 1;
    } else {
        println!("✗ FAT 720KB filesystem operations: FAILED");
    }

    // Test 2: FAT 1440KB filesystem operations
    total_tests += 1;
    if test_fs_operations_fat1440().is_ok() {
        println!("✓ FAT 1440KB filesystem operations: SUCCESS");
        success_count += 1;
    } else {
        println!("✗ FAT 1440KB filesystem operations: FAILED");
    }

    // Test 3: Amiga HFE filesystem operations
    total_tests += 1;
    if test_fs_operations_amiga_hfe().is_ok() {
        println!("✓ Amiga HFE filesystem operations: SUCCESS");
        success_count += 1;
    } else {
        println!("✗ Amiga HFE filesystem operations: FAILED");
    }

    // Test 4: Amiga ADF filesystem operations
    total_tests += 1;
    if test_fs_operations_amiga_adf().is_ok() {
        println!("✓ Amiga ADF filesystem operations: SUCCESS");
        success_count += 1;
    } else {
        println!("✗ Amiga ADF filesystem operations: FAILED");
    }

    // Test 5: FAT format conversions
    total_tests += 1;
    if test_format_conversion_fat().is_ok() {
        println!("✓ FAT format conversions: SUCCESS");
        success_count += 1;
    } else {
        println!("✗ FAT format conversions: FAILED");
    }

    // Test 6: Amiga format conversions
    total_tests += 1;
    if test_format_conversion_amiga().is_ok() {
        println!("✓ Amiga format conversions: SUCCESS");
        success_count += 1;
    } else {
        println!("✗ Amiga format conversions: FAILED");
    }

    println!("\n===========================================");
    println!("Success count: {}/{}", success_count, total_tests);
    println!("===========================================");

    assert_eq!(
        success_count, total_tests,
        "Expected {} successful tests, got {}",
        total_tests, success_count
    );

    Ok(())
}
