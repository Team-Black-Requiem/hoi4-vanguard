use std::{error::Error, path::{Path, PathBuf}};

// Define the FileSystem trait
pub(crate) trait FileSystem: std::fmt::Debug {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Box<dyn Error>>;
    fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>>;
    fn file_metadata(&self, path: &Path) -> Result<(u64, super::fileorg::FileCategory), Box<dyn Error>>;
}

// Define the SkipRule struct for folder skipping with replace_path
// This is a piece of bantha doodoo but it doth the job
pub(crate) struct SkipRule {
    pub(crate) directory: PathBuf,
    pub(crate) max_layer: usize,
}

impl SkipRule {
    pub(crate) fn new(directory: PathBuf, max_layer: usize) -> Self {
        Self { directory, max_layer }
    }

    pub(crate) fn should_skip(&self, current_layer: usize) -> bool {
        current_layer >= self.max_layer
    }
}


impl std::fmt::Debug for SkipRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkipRule")
            .field("directory", &self.directory)
            // Format the closure field by indicating it's a closure
            .field("predicate", &"<closure>")
            .finish()
    }
}