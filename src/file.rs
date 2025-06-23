use std::path::{Path, PathBuf};

use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub struct File {
    metadata: Option<Metadata>,
}

impl File {
    pub fn new(metadata: Metadata) -> Self {
        File {
            metadata: Some(metadata),
        }
    }

    pub fn set_destination_path(&mut self, destination_path: PathBuf) {
        if let Some(metadata) = &mut self.metadata {
            metadata.set_destination_path(destination_path);
        } else {
            self.metadata = Some(Metadata::new());
            if let Some(metadata) = &mut self.metadata {
                metadata.set_destination_path(destination_path);
            }
        }
    }

    pub fn build_destination_path(&mut self, path_components: Vec<&str>) -> PathBuf {
        let mut path = PathBuf::new();
        for path_component in path_components {
            path.push(path_component);
        }
        path
    }

    pub fn get_metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }
}
