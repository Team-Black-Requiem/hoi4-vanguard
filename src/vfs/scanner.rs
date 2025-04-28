// initial filescan 
use std::{
    fs::{self, File}, io::Read, path::{Path, PathBuf}, str
    };

use serde::Deserialize;
use jwalk::WalkDir;
use native_dialog::FileDialog;
extern crate winreg;
use winreg::{RegKey, enums::*};

use crate::parser;

use super::in_memory::Directory;
use super::filesystem::FileCategory;

    #[cfg(target_os = "windows")]                                    //acquire steam install from windows registry. ( ͡° ͜ʖ ͡°)
    fn find_hoi4_installation_path() -> Option<PathBuf> {

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let steam_key = hkcu.open_subkey_with_flags("SOFTWARE\\Valve\\Steam", KEY_READ).ok()?;
        let steam_path: String = steam_key.get_value("SteamPath").ok()?;
        let mut hoi4_path = Path::new(&steam_path).join("steamapps").join("common").join("Hearts of Iron IV");

        // Replace backslashes with forward slashes for consistent path format
        if let Some(path_str) = hoi4_path.to_str().map(|s| s.replace('\\', "/")) {
            hoi4_path = PathBuf::from(path_str);
        }

        // Check if the HOI4 executable exists in the installation path
        let hoi4_exe_path = hoi4_path.join("hoi4.exe");
        if hoi4_exe_path.exists() {
            Some(hoi4_path)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn find_hoi4_installation_path() -> Option<PathBuf> {
        None //Will need to investigate this later, for now auto-locating Steam will only support windows
    }

    fn select_hoi4_installation_path() -> Option<PathBuf> {
        // Use a file dialog to let the user select the HOI4 executable file
        // performance on this costs more then the registry method but that doesn't matter
        let result = FileDialog::new().add_filter("HOI4 Executable", &["exe"]).show_open_single_file();
        match result {
            Ok(Some(selected_file)) => {
                // Check if the selected file is actually 'hoi4.exe'
                if selected_file.file_name().map(|f| f == "hoi4.exe").unwrap_or(false) {
                    // Derive the installation folder by removing the executable file
                    let installation_path = selected_file.parent().map(PathBuf::from);
                    return installation_path;
                } else {
                    // Display an error message if the selected file is not 'hoi4.exe'
                    eprintln!("Selected file is not 'hoi4.exe'");
                }
            }
            _ => {
                // Either an error or user canceled the dialog
            }
        }
        None // Return None in case of an error or if the selected file is not 'hoi4.exe'
    }

    pub fn get_base_game_path() -> Result<PathBuf, Box<dyn std::error::Error>> { //annoys the user to death till a folder is selected or the program is closed
        loop {                                                                   //not the friendliest of approaches, but having the HOI4 folder is non-negotiable
            match find_hoi4_installation_path() {                                //TODO: Make this less assholish and add non-windows support
                Some(path) => {
                    log::info!("Retrieved Steam install at: {}", path.display());
                    return Ok(path);
                }
                None => {
                    log::warn!("Registry detection failed - falling back onto user.");
                    // Prompt the user to select the HOI4 installation path using a file dialog
                    match select_hoi4_installation_path() {
                        Some(selected_path) => {
                            log::info!("User selected HOI4 installation folder: {}", selected_path.display());
                            return Ok(selected_path);
                        }
                        None => {
                            log::error!("Could not find Hearts of Iron IV installation path");
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn scan_directory(string_manager: &crate::utility::util::StringResourceManager, path: &Path, exclude_criteria: &[String]) -> Result<Directory, Box<dyn std::error::Error>> {
        log::info!("Scanning directory: {}", path.display());
        let mut layer = Directory::new();        
        for entry in WalkDir::new(path)
        .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus::get()))
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(path)?;
            // Check if the entry path should be excluded
            if exclude_criteria.iter().any(|excluded| {
                if entry_path.is_file() {
                    // For files, check for an exact match
                    relative_path == Path::new(excluded)
                } else {
                    // For directories, check if the path starts with the excluded directory
                    relative_path.starts_with(excluded)
                }
            }) {
                // log::info!("Excluding: {}", entry_path.display());
                continue;
            }
            if entry.file_type().is_dir() {
                layer.add_directory(relative_path);
            } else {
                let file_category = FileCategory::categorize_file_extension(&entry_path);
                let file_name = relative_path.to_string_lossy();
                match file_category {
                    // Match parseable text files
                    FileCategory::Text | FileCategory::Gui | FileCategory::Gfx |
                    FileCategory::Asset | FileCategory::Csv | FileCategory::Mod => {
                        let mut file = File::open(&entry_path)?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        
                        //layer.add_file(file_name.as_ref(), contents.clone(), crate::parser::sharedparsers::AllResult::default());
                        // Parse the content using the `all` parser
                        //
                        let parsecontents = str::from_utf8(&contents).unwrap_or_default();
                        match parser::sharedparsers::all(parsecontents, string_manager) {
                            Ok((remaining, parsed_result)) => {
                                layer.add_file(file_name.as_ref(), contents.clone(), parsed_result);
                                if !remaining.is_empty() {
                                    log::warn!("Unparsed input remains: {:?}", remaining);
                                }
                            }
                            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                                layer.add_file(file_name.as_ref(), contents.clone(), crate::parser::sharedparsers::AllResult::default());
                                log::error!("Parsing error: {:?}, kind: {:?}", e.input, e.code);
                            }
                            Err(nom::Err::Incomplete(_)) => {
                                layer.add_file(file_name.as_ref(), contents.clone(), crate::parser::sharedparsers::AllResult::default());
                                log::error!("Parsing incomplete. More data needed.");              
                            }
                        }
                        
                    }
                    FileCategory::Yaml | FileCategory::Lua |
                    FileCategory::Shader   => {
                        // don't parse these files yet, but add them to the layer
                        let mut file = File::open(&entry_path)?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        
                        layer.add_file(file_name.as_ref(), contents.clone(), crate::parser::sharedparsers::AllResult::default());
                    }
                    // Match files that we acknowledge but don't parse
                    FileCategory::Sfx | FileCategory::Map | FileCategory::Image |
                    FileCategory::Mesh | FileCategory::Font | FileCategory::Sound |
                    FileCategory::Other => {
                        // Add file with dummy data
                        layer.add_file(file_name.as_ref(), vec![0], crate::parser::sharedparsers::AllResult::default());
                    }
                    FileCategory::Dir => {
                        // This case should not be possible
                        log::error!("Found directory when scanning directory: {}", entry_path.display());
                        log::error!("What the hell happened here?!");
                    },    
                }
            }
        }

        //log::info!("{} {}", layer.path_mappings.len(), " files found");

        Ok(layer)
    }