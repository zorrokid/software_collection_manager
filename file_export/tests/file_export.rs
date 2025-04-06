use std::io::Write;
use std::{
    collections::HashMap,
    fs::{self, File},
};

use file_export::{export_files, ExportType};
use tempfile::tempdir;

#[test]
fn test_export_files() {
    // Create a temporary directory for input and output
    let temp_dir = tempdir().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    // Create a sample compressed file
    let file_name = "test_file";
    let test_file_content = "Hello, world!";
    let test_file_content_sha1 = "943a702d06f34599aee1f8da8ef9f7296031d699";
    let compressed_file_path = input_dir.join(format!("{}.zst", file_name));
    let mut encoder = zstd::Encoder::new(File::create(&compressed_file_path).unwrap(), 0).unwrap();
    write!(encoder, "{}", test_file_content).unwrap();
    encoder.finish().unwrap();

    // Prepare file mappings
    let mut output_file_name_mapping = HashMap::new();
    output_file_name_mapping.insert(file_name.to_string(), "output_file".to_string());
    let mut filename_checksum_mapping = HashMap::new();
    filename_checksum_mapping.insert(file_name.to_string(), test_file_content_sha1.to_string());

    export_files(
        &input_dir,
        &output_dir,
        output_file_name_mapping,
        filename_checksum_mapping,
        ExportType::IndividualFilesWithoutCompression,
        None,
    )
    .unwrap();

    let output_file_path = output_dir.join("output_file");
    assert!(output_file_path.exists());
    let content = fs::read_to_string(output_file_path).unwrap();
    assert_eq!(content, "Hello, world!");
}
