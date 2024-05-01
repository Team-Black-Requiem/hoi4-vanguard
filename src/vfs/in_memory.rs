use std::{
    collections::{HashMap, hash_map::DefaultHasher, HashSet}, 
    path::{Path, PathBuf},
    hash::{Hash, Hasher},
    error::Error,
};
use xxhash_rust::xxh3::xxh3_64;

use super::{error::DirectoryError, filesystem::FileSystem, fileorg::FileCategory};

#[derive(Debug)]
pub(crate) struct Directory {
    files: HashMap<u64, Vec<u8>>,
    directories: HashMap<u64, HashSet<u64>>,
    pub path_mappings: HashMap<PathBuf, u64>,
}

impl Directory {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            directories: HashMap::new(),
            path_mappings: HashMap::new(),
        }
    }

    pub fn add_file<P: Into<PathBuf>>(&mut self, path: P, data: Vec<u8>) -> u64 {
        let path = path.into();
        let id = self.generate_id(&path);
        self.files.insert(id, data);
        self.path_mapper(&path, id);
        id
    }

    pub fn add_directory<P: Into<PathBuf>>(&mut self, path: P) -> u64 {
        let path = path.into();
        let id = self.generate_id(&path);
        self.directories.insert(id, HashSet::new());
        self.path_mapper(&path, id);
        id
    }

    fn path_mapper(&mut self, path: &Path, id: u64) {
        self.path_mappings.insert(path.to_path_buf(), id);
        if let Some(parent_path) = path.parent() {
            if let Some(&parent_id) = self.path_mappings.get(parent_path) {
                self.directories.entry(parent_id).or_default().insert(id);
            }
        }
    }

    pub fn file_hash<P: AsRef<Path>>(&self, path: P) -> Option<u64> {
        let path = path.as_ref();
        if let Some(id) = self.path_mappings.get(path) {
            if let Some(data) = self.files.get(id) {
                return Some(self.generate_id_xxh3(data));
            }
        }
        None
    }


    //pub fn detect_changes<P: AsRef<Path>>(&self, path: P) -> Option<bool> {
    //    let path = path.as_ref();
    //    if let Some(id) = self.path_mappings.get(path) {
    //        if let Some(data) = self.files.get(id) {
    //            // Calculate the current hash of the file contents
    //            let current_hash = self.generate_id_xxh3(&data);
    //            // Compare with the stored hash
    //            let stored_hash = self.generate_id_xxh3(path);
    //            return Some(current_hash != stored_hash);
    //        }
    //    }
    //    None
    //}

    fn generate_id_xxh3<T: AsRef<[u8]>>(&self, data: T) -> u64 {
        xxh3_64(data.as_ref())
    }

    fn generate_id<T: Hash>(&self, item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }

    fn resolve_path(&self, path: &Path) -> Result<u64, DirectoryError> {
        if let Some(id) = self.path_mappings.get(path) {
            log::info!("Found id: {}", id);
            Ok(*id)
        } else {
            Err(DirectoryError::NotFound)
        }
    }
}

impl FileSystem for Directory {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(data) = self.files.get(&id) {
            Ok(data.clone())
        } else {
            Err(Box::new(DirectoryError::NotFound))
        }
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(contents) = self.directories.get(&id) {
            let mut result = Vec::new();
            for &child_id in contents {
                if let Some(child_path) = self.path_mappings.iter().find_map(|(p, &i)| if i == child_id { Some(p.clone()) } else { None }) {
                    result.push(child_path);
                }
            }
            Ok(result)
        } else {
            Err(Box::new(DirectoryError::NotFound))
        }
    }

    fn file_metadata(&self, path: &Path) -> Result<(u64, FileCategory), Box<dyn Error>> {
        let id = self.resolve_path(path)?;
        if let Some(data) = self.files.get(&id) {
            let size = data.len() as u64;
            let category = FileCategory::categorize_file_extension(path);
            Ok((size, category))
        } else {
            Err(Box::new(DirectoryError::NotFound))
        }
    }
}