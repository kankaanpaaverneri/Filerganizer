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

    pub fn get_metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }
}
