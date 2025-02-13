use std::{
    error::Error, fmt, fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}
};

use regex::Regex;

use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;

// Custom error type for directory operations
#[derive(Debug)]
pub(crate) enum DirectoryError {
    NotFound,
    InvalidPath,
    NotAFile,
    NotADirectory,
}

impl Error for DirectoryError {}

impl std::fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DirectoryError::NotFound => write!(f, "File or directory not found"),
            DirectoryError::InvalidPath => write!(f, "Invalid path"),
            DirectoryError::NotAFile => write!(f, "Path is not a valid file"),
            DirectoryError::NotADirectory => write!(f, "Path is not a valid directory"),
        }
    }
}

impl std::convert::From<DirectoryError> for PyErr {
    fn from(err: DirectoryError) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

/// Define the SkipRule struct for folder skipping with replace_path
// This is a piece of bantha doodoo but it doth the job
#[derive(Debug)]
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

// Define a function to parse .mod files and extract skip rules
pub(crate) fn parse_mod_file(file_path: &Path, index: usize) -> Vec<SkipRule> {
    let mut skip_rules = Vec::new();
    // Regular expression to match a quoted path after "replace_path = "
    let re = Regex::new(r#"replace_path\s*=\s*"([^"]+)""#).unwrap();

    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim(); //this is new and I have no idea if it works
            if let Some(captures) = re.captures(&line) {
                // Extract the path from the capture group
                if let Some(path_str) = captures.get(1) {
                    let path = PathBuf::from(path_str.as_str());
                    skip_rules.push(SkipRule::new(path, index));
                }
            }
        }
    }

    skip_rules
}

///enums for categorizing paradox file types
//TODO: make this a bit more robust for games other then HOI4
//TODO: use this for per-file data store as I can tie the parser to the file type
#[derive(Debug, Clone)]
pub enum FileCategory {
    Text,
    Gui,
    Gfx,
    Sfx,
    Asset,
    Map,
    Yaml,
    Csv,
    Image,
    Shader,
    Lua,
    Mesh,
    Font,
    Sound,
    Mod,
    Other,
    Dir
}

impl FileCategory {
    pub fn categorize_file_extension(path: &Path) -> FileCategory {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "txt" => FileCategory::Text,
                "gui" => FileCategory::Gui,
                "gfx" => FileCategory::Gfx,
                "dds" | "tga" | "png" => FileCategory::Image,
                "yml" => FileCategory::Yaml,
                "csv" => FileCategory::Csv,
                "lua" => FileCategory::Lua,
                "shader" => FileCategory::Shader,
                "asset" => FileCategory::Asset,
                "mesh" => FileCategory::Mesh,
                "sfx" => FileCategory::Sfx,
                "wav" | "ogg" => FileCategory::Sound,
                "ttf" | "otf" => FileCategory::Font,
                "map" => FileCategory::Map,
                "mod" => FileCategory::Mod,
                _ => FileCategory::Other,
            }
        } else if path.is_file() {
            FileCategory::Other     //extensionless file
        } else {FileCategory::Dir}  //it shouldn't be possible to get here but just in case of catastrophic failure we'll return a directory
    }
}

impl fmt::Display for FileCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileCategory::Text => write!(f, "Text"),
            FileCategory::Gui => write!(f, "Gui"),
            FileCategory::Gfx => write!(f, "Gfx"),
            FileCategory::Sfx => write!(f, "Sfx"),
            FileCategory::Asset => write!(f, "Asset"),
            FileCategory::Map => write!(f, "Map"),
            FileCategory::Yaml => write!(f, "Yaml"),
            FileCategory::Csv => write!(f, "Csv"),
            FileCategory::Image => write!(f, "Image"),
            FileCategory::Shader => write!(f, "Shader"),
            FileCategory::Lua => write!(f, "Lua"),
            FileCategory::Mesh => write!(f, "Mesh"),
            FileCategory::Font => write!(f, "Font"),
            FileCategory::Sound => write!(f, "Sound"),
            FileCategory::Mod => write!(f, "Mod"),
            FileCategory::Other => write!(f, "Other: Unrecognized or extensionless file"),
            FileCategory::Dir => write!(f, "Dir: ERROR"),
        }
    }
}