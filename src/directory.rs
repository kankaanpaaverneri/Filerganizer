use crate::file::File;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Directory {
    directories: Option<BTreeMap<OsString, Directory>>,
    files: Option<BTreeMap<OsString, File>>,
    metadata: Option<Metadata>,
}

impl Directory {
    pub fn new(metadata: Option<Metadata>) -> Self {
        Directory {
            directories: None,
            files: None,
            metadata,
        }
    }

    pub fn read_path(
        &mut self,
        path: &PathBuf,
        new_directory: &mut Directory,
    ) -> std::io::Result<()> {
        match std::fs::read_dir(path) {
            Ok(read_dir) => {
                let metadata = self.read_parent(path);
                let mut directories = BTreeMap::new();
                let mut files = BTreeMap::new();
                insert_entries(&mut directories, &mut files, read_dir);

                if let None = self.directories {
                    new_directory.directories = Some(directories);
                    new_directory.files = Some(files);
                    new_directory.metadata = metadata;
                } else {
                    if let Some(last_dir) = self.get_mut_directory_by_path(path) {
                        last_dir.directories = Some(directories);
                        last_dir.files = Some(files);
                        last_dir.metadata = metadata;
                    }
                }
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn get_mut_directory_by_path(&mut self, path: &PathBuf) -> Option<&mut Directory> {
        let mut current_directory = self;
        if let Ok(striped_path) = remove_prefix_from_path(path) {
            for path_directory in striped_path {
                if let Some(sub_directories) = &mut current_directory.directories {
                    if let Some(sub_directory) = sub_directories.get_mut(path_directory) {
                        current_directory = sub_directory;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            return Some(current_directory);
        }
        None
    }

    pub fn get_directory_by_path(&self, path: &PathBuf) -> &Directory {
        let mut current_directory = self;
        for path_directory in path {
            if let Some(ref sub_directories) = current_directory.directories {
                if let Some(sub_directory) = sub_directories.get(path_directory) {
                    current_directory = sub_directory;
                }
            }
        }
        current_directory
    }

    pub fn clear_directory_content(&mut self) {
        if let Some(directories) = self.directories.as_mut() {
            directories.clear();
        }
        if let Some(files) = self.files.as_mut() {
            files.clear();
        }
        self.directories = None;
        self.files = None;
    }

    pub fn get_directories(&self) -> &Option<BTreeMap<OsString, Directory>> {
        &self.directories
    }

    pub fn get_files(&self) -> &Option<BTreeMap<OsString, File>> {
        &self.files
    }

    pub fn get_metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }

    fn read_parent(&self, path: &PathBuf) -> Option<Metadata> {
        if let Some(last) = path.iter().last() {
            let parent_path: PathBuf = path
                .iter()
                .filter_map(|directory| {
                    if directory == last {
                        return None;
                    }
                    Some(directory)
                })
                .collect();
            if !parent_path.as_os_str().is_empty() {
                if let Ok(metadata) = read_parent_entry(&parent_path, last) {
                    return metadata;
                }
            }
        }
        None
    }
}

fn read_parent_entry(path: &PathBuf, last_directory: &OsStr) -> std::io::Result<Option<Metadata>> {
    match std::fs::read_dir(path) {
        Ok(read_dir) => {
            for entry in read_dir {
                if let Some(ok_entry) = entry.ok() {
                    if ok_entry.file_name() == last_directory {
                        if let Some(parent) = write_directory_entry(&ok_entry) {
                            return Ok(parent.get_metadata().clone());
                        }
                    }
                }
            }
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn insert_entries(
    directories: &mut BTreeMap<OsString, Directory>,
    files: &mut BTreeMap<OsString, File>,
    read_dir: ReadDir,
) {
    for entry in read_dir {
        if let Some(ok_entry) = entry.ok() {
            let file_name = ok_entry.file_name();
            if let Some(directory) = write_directory_entry(&ok_entry) {
                directories.insert(OsString::from(file_name.as_os_str()), directory);
            }
            if let Some(file) = write_file_entry(&ok_entry) {
                files.insert(OsString::from(file_name.as_os_str()), file);
            }
        }
    }
}

fn write_directory_entry(entry: &DirEntry) -> Option<Directory> {
    match entry.metadata() {
        Ok(metadata) => {
            let created = metadata.created().ok().take();
            let accessed = metadata.accessed().ok().take();
            let modified = metadata.modified().ok().take();
            let readonly = metadata.permissions().readonly();
            if metadata.is_dir() {
                return Some(Directory::new(Some(Metadata::build(
                    Some(entry.file_name()),
                    created,
                    accessed,
                    modified,
                    None,
                    readonly,
                ))));
            }
            None
        }
        _ => None,
    }
}

fn write_file_entry(entry: &DirEntry) -> Option<File> {
    match entry.metadata() {
        Ok(metadata) => {
            let created = metadata.created().ok().take();
            let accessed = metadata.accessed().ok().take();
            let modified = metadata.modified().ok().take();
            let size = metadata.len() as f64;
            let readonly = metadata.permissions().readonly();
            if metadata.is_file() {
                return Some(File::new(Metadata::build(
                    Some(entry.file_name()),
                    created,
                    accessed,
                    modified,
                    Some(size),
                    readonly,
                )));
            }
            None
        }
        _ => None,
    }
}

fn remove_prefix_from_path(path: &PathBuf) -> Result<&Path, std::path::StripPrefixError> {
    match std::env::consts::OS {
        "windows" => {
            path.strip_prefix(identify_prefix(path))
        }
        _ => path.strip_prefix(OsString::from("/"))
    }
}

fn identify_prefix(path: &PathBuf) -> String {
    let first_two_components: Vec<_> = path.iter().take(2).filter_map(|component| {
        if let Some(element) = component.to_str() {
            return Some(element);
        }
        None
    }).collect();
    first_two_components.join("/")
}
