use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf}
};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::in_memory::Directory;
use super::filesystem::*;

use crate::{parser::sharedparsers::AllResult, utility::{position::{Pos, Range}, util::StringResourceManager}};

// Define the vfs struct
#[pyclass]
#[derive(Debug)]
pub(crate) struct Vfs {
    layers: Vec<Directory>, // Box<dyn FileSystem> is also possible, but it's not supported by pyo3 and we don't need it
    load_order: Vec<usize>,
    skip_rules: Vec<SkipRule>,
    string_manager: StringResourceManager,
}

#[pymethods]
impl Vfs {
    #[new]
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            load_order: Vec::new(),
            skip_rules: Vec::new(),
            string_manager: StringResourceManager::new(),
        }
    }

    pub fn scan_new_layer(&mut self, path: PathBuf, ignore_list: Vec<String>) {
        Vfs::add_layer(
            self,
            super::scanner::scan_directory(
                &self.string_manager,
                &path,
                &ignore_list //&[".vscode".to_string(), ".gitattributes".to_string(), ".git".to_string(), ".gitignore".to_string()]
            ).expect("Failed to scan directory for vfs setup"), path
        );
    }

    //these might be a little chunky for getting a bool, as we're essentially reading a file and then discarding the contents for an OK/Fail
    pub fn is_directory(&self, path: PathBuf) -> bool {
        self.read_dir(path).is_ok()
    }

    pub fn is_file(&self, path: PathBuf) -> bool {
        self.read_file(path).is_ok()
    }

    pub fn read_file(&self, path: PathBuf) -> PyResult<Vec<u8>> {
        for (index, layer) in self.layers.iter().enumerate().rev() {
            let current_layer = self.layers.len() - index - 1;
            if self.skip_dir(current_layer, &path) {
                continue; // Skip this layer if it's excluded by a rule
            }
            if let Ok(data) = layer.read_file(&path) {
                return Ok(data);
            }
        }
        Err(PyValueError::new_err(format!("File not found: {}", path.display())))
    }

    pub fn read_dir(&self, path: PathBuf) -> PyResult<HashSet<PathBuf>> {
        let mut result = std::collections::HashSet::new();
        for (index, layer) in self.layers.iter().enumerate().rev() {
            let current_layer = self.layers.len() - index - 1;
            if self.skip_dir(current_layer, &path) {
                continue; // Skip this layer if it's excluded by a rule
            }
    
            if let Ok(layer_contents) = layer.read_dir(&path) {
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

}

impl Vfs {
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

    pub fn overwrite_layer<P: Into<PathBuf>>(&mut self, layer: Directory, index: usize, path: P) {
        let path = path.into();
        let mod_file_path = path.join("descriptor.mod");
    
        self.layers[index] = layer;

        let skip_rules = parse_mod_file(&mod_file_path, index);
        for rule in skip_rules {
            self.add_skip_rule(rule);
        }
    }

    /// Find the highest-priority layer and file containing the given path.
    pub fn resolve_file_path(&self, path: &Path) -> Option<(usize, u32)> {
        for (index, layer) in self.layers.iter().enumerate().rev() {
            let current_layer = self.layers.len() - index - 1;
            if self.skip_dir(current_layer, path) {
                continue; // Skip this layer if it's excluded by a rule
            }

            if let Ok(file_id) = layer.resolve_path(path) {
                return Some((index, file_id));
            }
        }
        None
    }

    /// Resolve a unique file ID to a path irrespective of the layer or skip rules.
    pub fn resolve_file_id(&self, layer_index: usize, file_id: u32) -> Option<String> {
        self.layers.get(layer_index).and_then(|layer| {
            layer.path_mappings.iter().find_map(|(path, &id)| {
                if id == file_id {
                    Some(path.to_string_lossy().to_string())
                } else {
                    None
                }
            })
        })
    }

    pub fn add_skip_rule(&mut self, rule: SkipRule) {
        self.skip_rules.push(rule);
    }

    fn skip_dir(&self, current_layer: usize, directory: &Path) -> bool {
        self.skip_rules.iter().any(|rule| {
            rule.should_skip(current_layer) && directory.starts_with(&rule.directory)
                && directory.components().count() <= rule.directory.components().count() + 1
        })
    }

    pub fn file_metadata(&self, path: PathBuf) -> Result<(u32, FileCategory), Box<dyn Error>> {
        for (index, layer) in self.layers.iter().enumerate().rev() {
            let current_layer = self.layers.len() - index - 1;
            if self.skip_dir(current_layer, &path) {
                continue; // Skip this layer if it's excluded by a rule
            }
    
            if let Ok(metadata) = layer.file_metadata(&path) {
                return Ok(metadata);
            }
        }
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        )))
    }

    pub fn get_string_manager(&self) -> &StringResourceManager {
        &self.string_manager
    }
    
    pub fn read_parseresult(&self, path: PathBuf) -> Result<&AllResult, Box<dyn Error>> {
        for (index, layer) in self.layers.iter().enumerate().rev() {
            let current_layer = self.layers.len() - index - 1;
            if self.skip_dir(current_layer, &path) {
                continue; // Skip this layer if it's excluded by a rule
            }
            if let Ok(data) = layer.read_parseresult(&path) {
                return Ok(data);
            }
        }
        Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Parse result not found")))
    }

    /// Serialize a layer0 to a file
    pub fn serialize_basegame(&self, basegame: &str) -> Result<(), Box<dyn Error>> {
        let serialized_data = bincode::serialize(&self.layers[0]).unwrap();
        let file = File::create(format!("cache/{}_basegame.arse", basegame))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&serialized_data)?;
        writer.flush()?;
        Ok(())
    }

    pub fn deserialize_basegame(basegame: &str) -> Result<Directory, Box<dyn Error>> {
        let file = File::open(format!("cache/{}_basegame.arse", basegame))?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let layer = bincode::deserialize(&data)?;
        Ok(layer)
    }

}