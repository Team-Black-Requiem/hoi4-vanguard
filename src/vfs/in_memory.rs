use std::{
    collections::{HashMap, HashSet}, error::Error, hash::Hash, path::{Path, PathBuf}, str
};
use hash32::{FnvHasher, Hasher};
use serde::{Deserialize, Serialize};
//use xxhash_rust::xxh3::xxh3_64;

use crate::{parser::sharedparsers::{parse, AllResult}, utility::util::StringResourceManager};

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
        self.files.insert(id, VfsFile::new(data, AllResult::default()));
        self.path_mapper(&path, id);
        id
    }

    pub fn parse_file<P: Into<PathBuf>>(&mut self, path: P, string_manager: &StringResourceManager) -> u32 {
        let path = path.into();
        let id = self.resolve_path(&path).unwrap_or_else(|_| self.generate_id(&path));
        if let Some(file) = self.files.get_mut(&id) {
            let parseresult = parse(str::from_utf8(&file.raw_data).unwrap_or_default(), path, string_manager);
            file.set_parsed_data(parseresult);

        }
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
            //log::info!("Found id: {}", id);
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

    pub fn write_file<P: Into<PathBuf>>(&mut self, path: P, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let path = path.into();
        let id = self.resolve_path(&path)?;
        if let Some(file) = self.files.get_mut(&id) {
            file.raw_data = data;
            Ok(())
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

    pub fn read_parseresult(&self, path: &Path) -> Result<&AllResult, Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(data) = self.files.get(&id) {
            Ok(data.parsed_data().get_parsetree())
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
    pub fn new(raw_data: Vec<u8>, parseresult: AllResult) -> Self {
        Self {
            raw_data,
            parsed_data: ParseTree::new(parseresult),
        }
    }

    // Method to retrieve raw binary data
    pub fn raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    pub fn parsed_data(&self) -> &ParseTree {
        &self.parsed_data
    }

    pub fn set_raw_data(&mut self, data: Vec<u8>) {
        self.raw_data = data;
    }

    pub fn set_parsed_data(&mut self, parseresult: AllResult) {
        self.parsed_data = ParseTree::new(parseresult);
    }
}

// parsed output type, e.g., ParseTree
// Replace this with actual parsed output type
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ParseTree {
    parsetree: AllResult,
}

impl ParseTree {
    pub fn new(parseresult: AllResult) -> Self {
        Self {
            parsetree: parseresult
        }
    }
    pub fn default() -> Self {
        Self {
            parsetree: AllResult::default()
        }
    }

    pub fn get_parsetree(&self) -> &AllResult {
        &self.parsetree
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