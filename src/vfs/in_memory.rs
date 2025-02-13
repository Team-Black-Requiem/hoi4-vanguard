use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    hash::Hash,
    error::Error,
};
use hash32::{FnvHasher, Hasher};
use serde::{Deserialize, Serialize};
//use xxhash_rust::xxh3::xxh3_64;

use super::filesystem::*;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Directory {
    files: HashMap<u32, VfsFile>,
    directories: HashMap<u32, HashSet<u32>>,
    pub(super) path_mappings: HashMap<PathBuf, u32>,
}

impl Directory {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            directories: HashMap::new(),
            path_mappings: HashMap::new(),
        }
    }

    pub fn add_file<P: Into<PathBuf>>(&mut self, path: P, data: Vec<u8>) -> u32 {
        let path = path.into();
        let id = self.generate_id(&path);
        self.files.insert(id, VfsFile::new(data));
        self.path_mapper(&path, id);
        id
    }

    pub fn add_directory<P: Into<PathBuf>>(&mut self, path: P) -> u32 {
        let path = path.into();
        let id = self.generate_id(&path);
        self.directories.insert(id, HashSet::new());
        self.path_mapper(&path, id);
        id
    }

    fn path_mapper(&mut self, path: &Path, id: u32) {
        self.path_mappings.insert(path.to_path_buf(), id);
        if let Some(parent_path) = path.parent() {
            if let Some(&parent_id) = self.path_mappings.get(parent_path) {
                self.directories.entry(parent_id).or_default().insert(id);
            }
        }
    }

    fn generate_id<T: Hash>(&self, item: &T) -> u32 {
        let mut hasher = FnvHasher::default();
        item.hash(&mut hasher);
        hasher.finish32()
    }

    pub fn resolve_path(&self, path: &Path) -> Result<u32, DirectoryError> {
        if let Some(id) = self.path_mappings.get(path) {
            log::info!("Found id: {}", id);
            Ok(*id)
        } else {
            Err(DirectoryError::NotFound)
        }
    }

    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(data) = self.files.get(&id) {
            Ok(data.raw_data().to_vec())
        } else {
            Err(Box::new(DirectoryError::NotFound))
        }
    }

    pub fn read_dir(&self, path: &Path) -> Result<HashSet<PathBuf>, Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(contents) = self.directories.get(&id) {
            let mut result = HashSet::new();
            for &child_id in contents {
                if let Some(child_path) = self.path_mappings.iter().find_map(|(p, &i)| if i == child_id { Some(p.clone()) } else { None }) {
                    result.insert(child_path);
                }
            }
            Ok(result)
        } else {
            Err(Box::new(DirectoryError::NotFound))
        }
    }

    pub fn file_metadata(&self, path: &Path) -> Result<(u32, FileCategory), Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(data) = self.files.get(&id) {
            let size = data.raw_data().len() as u32;
            let category = FileCategory::categorize_file_extension(path);
            Ok((size, category))
        } else {
            Err(Box::new(DirectoryError::NotFound))
        }
    }
}

/// VFS file type
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct VfsFile {
    /// Raw binary data
    pub raw_data: Vec<u8>,    
    /// Parsed output, e.g., parse tree or AST
    pub parsed_data: ParseTree,
}

impl VfsFile {
    pub fn new(raw_data: Vec<u8>) -> Self {
        Self {
            raw_data,
            parsed_data: ParseTree::new(),
        }
    }

    // Method to retrieve raw binary data
    pub fn raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    pub fn set_raw_data(&mut self, data: Vec<u8>) {
        self.raw_data = data;
    }
}

// parsed output type, e.g., ParseTree
// Replace this with actual parsed output type
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ParseTree {
    // Define the structure of your parse tree or AST here
    // This should represent the parsed output of your binary text data
}

impl ParseTree {
    pub fn new() -> Self {
        Self {
            // Define the structure of your parse tree or AST here
            // This should represent the parsed output of your binary text data
        }
    }
}

// Define structure for common data derived from parsed output
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct IndexedData {
    // Define indexed data derived from parsed output
    // This could include any metadata or summary information
}

impl IndexedData {
    pub fn new() -> Self {
        Self {
            // Define indexed data derived from parsed output
            // This could include any metadata or summary information
        }
    }
}