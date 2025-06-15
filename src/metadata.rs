use chrono::{DateTime, Local};
use std::{ffi::OsString, path::PathBuf, time::SystemTime};

#[derive(Debug, Clone)]
pub struct Metadata {
    name: Option<OsString>,
    created: Option<DateTime<Local>>,
    accessed: Option<DateTime<Local>>,
    modified: Option<DateTime<Local>>,
    size: Option<f64>,
    readonly: bool,
    origin_path: Option<PathBuf>,
    destination_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateType {
    Created,
    Accessed,
    Modified,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            name: None,
            created: None,
            accessed: None,
            modified: None,
            size: None,
            readonly: false,
            origin_path: None,
            destination_path: None,
        }
    }

    pub fn get_formated_date(&self, date_type: DateType) -> Option<String> {
        match date_type {
            DateType::Created => {
                if let Some(created) = self.created {
                    let formated = created.format("%Y%m%d").to_string();
                    return Some(formated);
                }
                None
            }
            DateType::Accessed => {
                if let Some(accessed) = self.accessed {
                    let formated = accessed.format("%Y%m%d").to_string();
                    return Some(formated);
                }
                None
            }
            DateType::Modified => {
                if let Some(modified) = self.accessed {
                    let formated = modified.format("%Y%m%d").to_string();
                    return Some(formated);
                }
                None
            }
        }
    }

    pub fn set_destination_path(&mut self, destination_path: PathBuf) {
        self.destination_path = Some(destination_path);
    }

    pub fn get_name(&self) -> Option<OsString> {
        self.name.clone()
    }

    pub fn get_created(&self) -> Option<DateTime<Local>> {
        self.created
    }

    pub fn get_accessed(&self) -> Option<DateTime<Local>> {
        self.accessed
    }

    pub fn get_modified(&self) -> Option<DateTime<Local>> {
        self.modified
    }

    pub fn get_size(&self) -> Option<f64> {
        self.size
    }

    pub fn get_readonly(&self) -> bool {
        self.readonly
    }

    pub fn get_origin_path(&self) -> Option<PathBuf> {
        self.origin_path.clone()
    }

    pub fn get_destination_path(&self) -> Option<PathBuf> {
        self.destination_path.clone()
    }

    pub fn build_local_time(
        name: Option<OsString>,
        created: Option<DateTime<Local>>,
        accessed: Option<DateTime<Local>>,
        modified: Option<DateTime<Local>>,
        size: Option<f64>,
        readonly: bool,
        origin_path: Option<PathBuf>,
        destination_path: Option<PathBuf>,
    ) -> Self {
        Self {
            name,
            created,
            accessed,
            modified,
            size,
            readonly,
            origin_path,
            destination_path,
        }
    }

    pub fn build(
        name: Option<OsString>,
        created: Option<SystemTime>,
        accessed: Option<SystemTime>,
        modified: Option<SystemTime>,
        size: Option<f64>,
        readonly: bool,
        origin_path: Option<PathBuf>,
        destination_path: Option<PathBuf>,
    ) -> Self {
        Self::convert_metadata_to_datetime(
            name,
            created,
            accessed,
            modified,
            size,
            readonly,
            origin_path,
            destination_path,
        )
    }

    fn convert_metadata_to_datetime(
        name: Option<OsString>,
        created: Option<SystemTime>,
        accessed: Option<SystemTime>,
        modified: Option<SystemTime>,
        size: Option<f64>,
        readonly: bool,
        origin_path: Option<PathBuf>,
        destination_path: Option<PathBuf>,
    ) -> Self {
        let mut metadata = Self::new();
        metadata.name = name;
        if let Some(c) = created {
            metadata.created = Some(c.into());
        }
        if let Some(a) = accessed {
            metadata.accessed = Some(a.into());
        }
        if let Some(m) = modified {
            metadata.modified = Some(m.into());
        }
        metadata.size = size;
        metadata.readonly = readonly;
        metadata.origin_path = origin_path;
        metadata.destination_path = destination_path;
        metadata
    }
}
