use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use super::{filesystem::*, fileorg::{FileCategory, parse_mod_file}};


// Define the UnionFileSystem struct
#[derive(Debug)]
pub(crate) struct UnionFileSystem {
    layers: Vec<Box<dyn FileSystem>>,
    load_order: Vec<usize>,
    skip_rules: Vec<SkipRule>,
}

impl UnionFileSystem {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            load_order: Vec::new(),
            skip_rules: Vec::new()
        }
    }

    pub fn add_layer<P: Into<PathBuf>>(&mut self, layer: Box<dyn FileSystem>, path: P) {
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

    //these might be a little chunky for getting a bool, so we'll look at setting up a better solution
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

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut contents = Vec::new();
        for (index, layer) in self.layers.iter().enumerate().rev() {
            if let Ok(layer_contents) = layer.read_dir(path) {
                // Calculate the current layer based on the depth in the filesystem hierarchy
                let current_layer = self.layers.len() - index - 1;
                for file in layer_contents {
                    if self.skip_dir(current_layer, &file) {
                        continue;
                    }
                    contents.push(file);
                }
            }
        }
        Ok(contents)
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