use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::SkipRule;

//enums for categorizing paradox file types
//TODO: make this a bit more robust for games other then HOI4
//TODO: use this for per-file data store as I can tie the parser to the file type
#[derive(Debug, Clone)]
pub enum FileCategory {
    Text,
    GUI,
    GFX,
    SFX,
    Asset,
    Map,
    YAML,
    CSV,
    Image,
    Shader,
    Lua,
    Mesh,
    Font,
    Sound,
    Dot_Mod,
    Other,
}

impl FileCategory {
    pub fn categorize_file_extension(path: &Path) -> FileCategory {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "txt" => FileCategory::Text,
                "gui" => FileCategory::GUI,
                "gfx" => FileCategory::GFX,
                "dds" | "tga" | "png" => FileCategory::Image,
                "yml" => FileCategory::YAML,
                "csv" => FileCategory::CSV,
                "lua" => FileCategory::Lua,
                "shader" => FileCategory::Shader,
                "asset" => FileCategory::Asset,
                "mesh" => FileCategory::Mesh,
                "sfx" => FileCategory::SFX,
                "wav" | "ogg" => FileCategory::Sound,
                "ttf" | "otf" => FileCategory::Font,
                "map" => FileCategory::Map,
                "mod" => FileCategory::Dot_Mod,
                _ => FileCategory::Other,
            }
        } else {
            FileCategory::Other
        }
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