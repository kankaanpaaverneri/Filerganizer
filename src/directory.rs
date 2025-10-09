use crate::file::File;
use crate::metadata::Metadata;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{DirEntry, ReadDir};
use std::io::ErrorKind;
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

    pub fn insert_file(&mut self, file_name: OsString, file: File) {
        if let Some(mut files) = self.files.take() {
            files.insert(file_name, file);
            self.files = Some(files);
        } else {
            let mut files = BTreeMap::new();
            files.insert(file_name, file);
            let _result = self.files.insert(files);
        }
    }

    pub fn insert_empty_files(&mut self) {
        self.files = Some(BTreeMap::new());
    }

    pub fn insert_directory(&mut self, new_directory: Directory, directory_name: &str) {
        if let Some(mut directories) = self.directories.take() {
            directories.insert(OsString::from(directory_name), new_directory);
            self.directories = Some(directories);
        } else {
            let mut new_directories = BTreeMap::new();
            new_directories.insert(OsString::from(directory_name), new_directory);
            let _result = self.directories.insert(new_directories);
        }
    }

    pub fn contains_unique_files(
        &self,
        files_holder: &BTreeMap<OsString, File>,
    ) -> std::io::Result<()> {
        if let Some(files) = &self.files {
            for key in files.keys() {
                if files_holder.contains_key(key) {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        "Duplicate files found in directory",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn read_path(
        &mut self,
        path: &PathBuf,
        new_directory: &mut Directory,
    ) -> std::io::Result<()> {
        let read_dir = std::fs::read_dir(path)?;
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

    pub fn get_file_count(&self) -> usize {
        if let Some(files) = &self.files {
            return files.len();
        }
        0
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

    pub fn insert_new_directories(&mut self, new_dirs: BTreeMap<OsString, Directory>) {
        for (dir_name, new_dir) in new_dirs {
            if let Some(dir_name) = dir_name.to_str() {
                self.insert_directory(new_dir, dir_name);
            }
        }
    }

    pub fn filter_duplicate_directories(&self, directories: &mut BTreeMap<OsString, Directory>) {
        if let Some(selected_dirs) = self.get_directories() {
            *directories = directories
                .iter()
                .filter_map(|(key, dir)| {
                    if selected_dirs.contains_key(key) {
                        return None;
                    }
                    return Some((OsString::from(key.as_os_str()), (*dir).clone()));
                })
                .collect();
        }
    }

    pub fn file_already_exists_in_directory(&self, filename: &OsStr) -> std::io::Result<()> {
        if let Some(files) = &self.files {
            for key in files.keys() {
                if key == filename {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "File name already exists in directory",
                    ));
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> Option<OsString> {
        if let Some(metadata) = self.get_metadata() {
            return metadata.get_name();
        }
        None
    }

    pub fn get_directories(&self) -> &Option<BTreeMap<OsString, Directory>> {
        &self.directories
    }

    pub fn get_mut_directories(&mut self) -> &mut Option<BTreeMap<OsString, Directory>> {
        &mut self.directories
    }

    pub fn get_files(&self) -> &Option<BTreeMap<OsString, File>> {
        &self.files
    }

    pub fn get_mut_files(&mut self) -> &mut Option<BTreeMap<OsString, File>> {
        &mut self.files
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
    let read_dir = std::fs::read_dir(path)?;
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
    let origin_path = entry.path();
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
                    Some(origin_path),
                    None,
                ))));
            }
            None
        }
        _ => None,
    }
}

fn write_file_entry(entry: &DirEntry) -> Option<File> {
    let origin_path = entry.path();
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
                    Some(origin_path),
                    None,
                )));
            }
            None
        }
        _ => None,
    }
}

fn remove_prefix_from_path(path: &PathBuf) -> Result<&Path, std::path::StripPrefixError> {
    match std::env::consts::OS {
        "windows" => path.strip_prefix(identify_prefix(path)),
        "macos" => path.strip_prefix(OsString::from("/")),
        "linux" => path.strip_prefix(OsString::from("/")),
        _ => path.strip_prefix(OsString::from("/")),
    }
}

fn identify_prefix(path: &PathBuf) -> String {
    let first_two_components: Vec<_> = path
        .iter()
        .take(2)
        .filter_map(|component| {
            if let Some(element) = component.to_str() {
                return Some(element);
            }
            None
        })
        .collect();
    first_two_components.join("/")
}

pub mod system_dir {
    use std::path::PathBuf;
    pub fn get_home_directory() -> Option<PathBuf> {
        let environment_var = match std::env::consts::OS {
            "windows" => std::env::var_os("USERPROFILE"),
            "macos" | "linux" => std::env::var_os("HOME"),
            _ => None,
        };

        if let Some(key) = environment_var {
            return Some(PathBuf::from(key));
        }
        None
    }
    pub fn get_current_dir() -> Option<PathBuf> {
        let result = std::env::current_dir();
        match result {
            Ok(current_dir) => Some(current_dir),
            Err(error) => {
                eprintln!("Could not locate working directory: {}", error);
                None
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_identify_prefix() {
        let path = match std::env::consts::OS {
            "windows" => PathBuf::from("C:/home/verneri/rust"),
            "macos" | "linux" => PathBuf::from("/home/verneri/rust"),
            _ => PathBuf::new(),
        };
        let prefix = identify_prefix(&path);
        match std::env::consts::OS {
            "windows" => assert_eq!(prefix, String::from("C:/\\")),
            "macos" | "linux" => assert_eq!(prefix, String::from("//home")),
            _ => panic!("Not supported operating system"),
        };
        let path = PathBuf::from("C:/Users/verneri");
        let prefix = identify_prefix(&path);
        match std::env::consts::OS {
            "windows" => assert_eq!(prefix, String::from("C:/\\")),
            "macos" | "linux" => assert_eq!(prefix, String::from("C:/Users")),
            _ => panic!("Not supported operating system"),
        };
    }

    #[test]
    fn test_file_already_exists_in_directory() {
        let mut dir = Directory::new(None);
        dir.insert_file(OsString::from("file.txt"), File::new(Metadata::new()));
        match dir.file_already_exists_in_directory(&OsString::from("file.txt")) {
            Ok(_) => {
                panic!("File exists in directory");
            }
            Err(error) => {
                assert_eq!(
                    error.to_string(),
                    String::from("File name already exists in directory")
                );
            }
        }
        let dir = Directory::new(None);
        match dir.file_already_exists_in_directory(&OsString::from("file.txt")) {
            Ok(_) => {}
            Err(error) => {
                panic!("File didn't exist in directory, {}", error);
            }
        }
    }

    #[test]
    fn test_filter_duplicate_directories() {
        let mut new_directories = BTreeMap::new();
        new_directories.insert(OsString::from("content"), Directory::new(None));
        new_directories.insert(OsString::from("text_files"), Directory::new(None));
        new_directories.insert(OsString::from("images"), Directory::new(None));
        let mut directory = Directory::new(None);
        directory.filter_duplicate_directories(&mut new_directories);
        assert_eq!(
            new_directories.contains_key(&OsString::from("content")),
            true
        );
        assert_eq!(
            new_directories.contains_key(&OsString::from("text_files")),
            true
        );
        assert_eq!(
            new_directories.contains_key(&OsString::from("images")),
            true
        );
        directory.insert_directory(Directory::new(None), "text_files");
        directory.filter_duplicate_directories(&mut new_directories);
        assert_eq!(
            new_directories.contains_key(&OsString::from("text_files")),
            false
        );
        assert_eq!(
            new_directories.contains_key(&OsString::from("content")),
            true
        );
        assert_eq!(
            new_directories.contains_key(&OsString::from("images")),
            true
        );
    }

    #[test]
    fn test_insert_new_directories() {
        let mut directory = Directory::new(None);
        let mut directories = BTreeMap::new();
        directories.insert(OsString::from("content"), Directory::new(None));
        directories.insert(OsString::from("text_files"), Directory::new(None));
        directories.insert(OsString::from("images"), Directory::new(None));
        directory.insert_new_directories(directories);
        if let Some(directories) = directory.get_directories() {
            assert_eq!(directories.contains_key(&OsString::from("content")), true);
            assert_eq!(
                directories.contains_key(&OsString::from("text_files")),
                true
            );
            assert_eq!(directories.contains_key(&OsString::from("images")), true);
        } else {
            panic!("Could not get directories");
        }
    }

    #[test]
    fn test_remove_sub_directory() {
        let mut directory = Directory::new(None);
        directory.insert_directory(Directory::new(None), "content");
        directory.insert_directory(Directory::new(None), "text_files");
        if let Some(directories) = directory.get_directories() {
            assert_eq!(
                directories.contains_key(&OsString::from("text_files")),
                true
            );
            assert_eq!(directories.contains_key(&OsString::from("content")), false);
        } else {
            panic!("Could not get directories");
        }
    }

    #[test]
    fn test_clear_directory_content() {
        let mut directory = Directory::new(None);
        directory.insert_directory(Directory::new(None), "content");
        directory.insert_directory(Directory::new(None), "text_files");
        directory.insert_file(OsString::from("file.txt"), File::new(Metadata::new()));
        directory.clear_directory_content();
        if let Some(_directories) = directory.get_directories() {
            panic!("There should not be any directories after clear");
        }
        if let Some(_files) = directory.get_files() {
            panic!("There should not be any files after clear");
        }
    }

    #[test]
    fn test_get_file_count() {
        let mut directory = Directory::new(None);
        assert_eq!(directory.get_file_count(), 0);
        directory.insert_file(OsString::from("file.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file1.txt"), File::new(Metadata::new()));
        assert_eq!(directory.get_file_count(), 2);
        directory.insert_file(OsString::from("file2.txt"), File::new(Metadata::new()));
        assert_eq!(directory.get_file_count(), 3);
    }

    fn create_dummy_metadata_with_name(name: OsString) -> Option<Metadata> {
        Some(Metadata::build(
            Some(name),
            None,
            None,
            None,
            Some(15.5),
            false,
            None,
            None,
        ))
    }

    fn get_dummy_directory_tree() -> Directory {
        let mut root_identifier = match std::env::consts::OS {
            "windows" => OsString::from("C:/"),
            "macos" | "linux" => OsString::from("/"),
            _ => OsString::new(),
        };
        let mut directory = Directory::new(create_dummy_metadata_with_name(OsString::from(
            &root_identifier,
        )));
        directory.insert_directory(
            Directory::new(create_dummy_metadata_with_name(OsString::from(
                "text_files",
            ))),
            "text_files",
        );
        directory.insert_directory(
            Directory::new(create_dummy_metadata_with_name(OsString::from("content"))),
            "content",
        );
        directory.insert_directory(
            Directory::new(create_dummy_metadata_with_name(OsString::from("images"))),
            "images",
        );

        directory.insert_file(OsString::from("file1.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file2.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file3.txt"), File::new(Metadata::new()));
        root_identifier.push("text_files");
        if let Some(text_files) =
            directory.get_mut_directory_by_path(&PathBuf::from(&root_identifier))
        {
            text_files.insert_directory(
                Directory::new(create_dummy_metadata_with_name(OsString::from("docx"))),
                "docx",
            );
            text_files.insert_directory(
                Directory::new(create_dummy_metadata_with_name(OsString::from("txt"))),
                "txt",
            );
            text_files.insert_file(OsString::from("file4.txt"), File::new(Metadata::new()));
            text_files.insert_file(OsString::from("file5.txt"), File::new(Metadata::new()));
        }
        directory
    }

    #[test]
    fn test_get_directory_by_path() {
        let directory = get_dummy_directory_tree();
        let search_path = match std::env::consts::OS {
            "windows" => PathBuf::from("C:/text_files/txt"),
            "macos" | "linux" => PathBuf::from("/text_files/txt"),
            _ => PathBuf::new(),
        };
        let dir = directory.get_directory_by_path(&search_path);
        if let Some(name) = dir.get_name() {
            assert_eq!(name, OsString::from("txt"));
        } else {
            panic!("Didn't get name from directory");
        }

        let search_path = match std::env::consts::OS {
            "windows" => PathBuf::from("C:/content/video"),
            "macos" | "linux" => PathBuf::from("/content/video"),
            _ => PathBuf::new(),
        };
        let dir = directory.get_directory_by_path(&search_path);
        if let Some(root) = dir.get_name() {
            assert_eq!(root, OsString::from("content"));
        } else {
            panic!("Didn't find any directory");
        }
    }

    #[test]
    fn test_get_mut_directory_by_path() {
        let mut directory = get_dummy_directory_tree();
        let search_path = match std::env::consts::OS {
            "windows" => PathBuf::from("C:/text_files/txt"),
            "macos" | "linux" => PathBuf::from("/text_files/txt"),
            _ => PathBuf::new(),
        };

        if let Some(txt) = directory.get_mut_directory_by_path(&search_path) {
            if let Some(txt_name) = txt.get_name() {
                assert_eq!(txt_name, OsString::from("txt"));
            } else {
                panic!("txt name was incorrect");
            }
        } else {
            panic!("Didn't get directory by path");
        }

        let search_path = match std::env::consts::OS {
            "windows" => PathBuf::from("C:/text_files/docx"),
            "macos" | "linux" => PathBuf::from("/text_files/docx"),
            _ => PathBuf::new(),
        };

        if let Some(docx) = directory.get_mut_directory_by_path(&search_path) {
            if let Some(docx_name) = docx.get_name() {
                assert_eq!(docx_name, OsString::from("docx"));
            } else {
                panic!("docx name was incorrect");
            }
        }

        if let Some(_dir) = directory.get_mut_directory_by_path(&PathBuf::from("/content/video")) {
            panic!("Some directory was found when path was incorrect")
        }
    }

    #[test]
    fn test_contains_unique_files() {
        let directory = get_dummy_directory_tree();
        let mut files = BTreeMap::new();
        files.insert(OsString::from("file01.txt"), File::new(Metadata::new()));
        files.insert(OsString::from("file02.txt"), File::new(Metadata::new()));
        files.insert(OsString::from("file1.txt"), File::new(Metadata::new()));
        if let Err(error) = directory.contains_unique_files(&files) {
            assert_eq!(
                error.to_string(),
                String::from("Duplicate files found in directory")
            );
        } else {
            panic!("Failed to detect duplicate files in directory");
        }
        let mut files = BTreeMap::new();
        files.insert(OsString::from("file01.txt"), File::new(Metadata::new()));
        files.insert(OsString::from("file02.txt"), File::new(Metadata::new()));
        if let Err(error) = directory.contains_unique_files(&files) {
            panic!("Failed to detect duplicate files in directory: {}", error);
        }
    }

    #[test]
    fn test_insert_directory() {
        let mut directory = Directory::new(None);
        directory.insert_directory(
            Directory::new(create_dummy_metadata_with_name(OsString::from("content"))),
            "content",
        );
        if let Some(directories) = directory.get_directories() {
            assert_eq!(directories.contains_key(&OsString::from("content")), true);
        } else {
            panic!("Inserting to directory failed.");
        }
    }

    #[test]
    fn test_insert_file() {
        let mut directory = Directory::new(None);
        directory.insert_file(OsString::from("file1.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file2.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file3.txt"), File::new(Metadata::new()));
        directory.insert_file(OsString::from("file4.txt"), File::new(Metadata::new()));
        if let Some(files) = directory.get_files() {
            assert_eq!(files.contains_key(&OsString::from("file1.txt")), true);
            assert_eq!(files.contains_key(&OsString::from("file2.txt")), true);
            assert_eq!(files.contains_key(&OsString::from("file3.txt")), true);
            assert_eq!(files.contains_key(&OsString::from("file4.txt")), true);
            assert_eq!(files.contains_key(&OsString::from("file5.txt")), false);
        }
    }
}
