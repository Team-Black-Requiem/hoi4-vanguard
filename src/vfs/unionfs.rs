use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use pyo3::prelude::*;

use super::in_memory::Directory;
use super::filesystem::*;


// Define the UnionFileSystem struct
#[pyclass]
#[derive(Debug)]
pub(crate) struct UnionFileSystem {
    layers: Vec<Directory>, // Box<dyn FileSystem> is also possible, but it's not supported by pyo3 and we don't need it
    load_order: Vec<usize>,
    skip_rules: Vec<SkipRule>,
}

#[pymethods]
impl UnionFileSystem {
    #[new]
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            load_order: Vec::new(),
            skip_rules: Vec::new()
        }
    }

    //pub fn new_layer(&mut self, path: PathBuf, ignore_list: Vec<String>) {
    //    UnionFileSystem::add_layer(
    //        self,
    //        super::scanner::scan_directory(
    //            &path,
    //            &ignore_list
    //        ).expect("Failed to scan directory for VFS setup"), path
    //    );
    //}
}

impl UnionFileSystem {
    pub fn add_layer<P: Into<PathBuf>>(&mut self, layer: Directory, path: P) {
        self.layers.push(layer);
        let index = self.layers.len() - 1;
        self.load_order.push(index);
    
        let path = path.into();
        let mod_file_path = path.join("descriptor.mod");
    
        let skip_rules = parse_mod_file(&mod_file_path, index);
        for rule in skip_rules {
            self.add_skip_rule(rule);
        }
    }

    pub fn add_skip_rule(&mut self, rule: SkipRule) {
        self.skip_rules.push(rule);
    }

    fn skip_dir(&self, current_layer: usize, directory: &Path) -> bool {
        for rule in &self.skip_rules {
            if rule.should_skip(current_layer) && directory.starts_with(&rule.directory) {
                //See if directory is not a direct subdirectory of the skip rule's directory by doing math on the path lenth
                if directory.components().count()  <= rule.directory.components().count() + 1{
                    return true; // Skip if the conditions are met
                }
            }
        }
        false
    }

    //these might be a little chunky for getting a bool, as we're essentially reading a file and then discarding the contents for an OK/Fail
    pub fn is_directory(&self, path: &Path) -> bool {
        self.read_dir(path).is_ok()
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.read_file(path).is_ok()
    }

}


impl FileSystem for UnionFileSystem {

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
        for (index, layer) in self.layers.iter().enumerate().rev() {
            if let Ok(data) = layer.read_file(path) {
                let current_layer = self.layers.len() - index - 1;
                if !self.skip_dir(current_layer, path) {
                    return Ok(data);
                }
            }
        }
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        )))
    }

    fn read_dir(&self, path: &Path) -> Result<HashSet<PathBuf>, Box<dyn Error>> {
        let mut result = std::collections::HashSet::new();
        for (index, layer) in self.layers.iter().enumerate().rev() {
            if let Ok(layer_contents) = layer.read_dir(path) {
                let current_layer = self.layers.len() - index - 1;
                for file in layer_contents {
                    if self.skip_dir(current_layer, &file) {
                        continue;
                    }
                    result.insert(file);
                }
            }
        }
        Ok(result)
    }

    fn file_metadata(&self, path: &Path) -> Result<(u64, FileCategory), Box<dyn Error>> {
        for (index, layer) in self.layers.iter().enumerate().rev() {
            if let Ok(metadata) = layer.file_metadata(path) {
                let current_layer = self.layers.len() - index - 1;
                if !self.skip_dir(current_layer, path) {
                    return Ok(metadata);
                }
            }
        }
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        )))
    }
}