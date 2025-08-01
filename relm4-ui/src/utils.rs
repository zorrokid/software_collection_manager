use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use core_types::{FileType, Sha1Checksum};
use file_export::{FileSetExportModel, OutputFile};
use file_import::FileImportModel;
use service::view_models::FileSetViewModel;

use crate::file_importer::FileImporter;

pub fn resolve_file_type_path(root_path: &Path, file_type: &FileType) -> PathBuf {
    let mut path = PathBuf::from(root_path);
    path.push(file_type.dir_name());
    path
}

pub fn prepare_fileset_for_export(
    file_set: &FileSetViewModel,
    collection_root_dir: &Path,
    temp_dir: &Path,
    extract_files: bool,
) -> FileSetExportModel {
    let source_file_path = resolve_file_type_path(collection_root_dir, &file_set.file_type.into());

    let output_mapping = file_set
        .files
        .iter()
        .map(|f| {
            let checksum: Sha1Checksum = f
                .sha1_checksum
                .clone()
                .try_into()
                .expect("Failed to convert to Sha1Checksum");
            (
                f.archive_file_name.clone(),
                OutputFile {
                    output_file_name: f.file_name.clone(),
                    checksum,
                },
            )
        })
        .collect::<HashMap<String, OutputFile>>();

    let exported_zip_file_name = file_set.file_set_name.clone();

    FileSetExportModel {
        output_mapping,
        source_file_path,
        output_dir: temp_dir.to_path_buf(),
        extract_files,
        exported_zip_file_name,
    }
}

pub fn prepare_file_import(
    file_path: &Path,
    file_type: FileType,
    collection_root_dir: &Path,
    file_importer: &FileImporter,
) -> FileImportModel {
    let target_path = resolve_file_type_path(collection_root_dir, &file_type);
    let file_name_filter = file_importer
        .get_selected_files_from_current_picked_file_that_are_new()
        .iter()
        .map(|file| file.file_name.clone())
        .collect::<HashSet<String>>();

    let is_zip_file = file_importer.is_zip_file();
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    FileImportModel {
        file_path: file_path.to_path_buf(),
        file_type,
        output_dir: target_path.to_path_buf(),
        file_name_filter,
        file_name,
        is_zip_file,
    }
}
