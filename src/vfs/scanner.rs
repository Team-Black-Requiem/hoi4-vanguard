// initial filescan 
use std::{
    path::{Path, PathBuf},
    io::Read,
    fs::File,
    };

use jwalk::WalkDir;
use native_dialog::FileDialog;
extern crate winreg;
use winreg::{RegKey, enums::*};

use super::in_memory::Directory;
use super::fileorg::FileCategory;

const SIMULATE_REGISTRY_FAILURE: bool = false;  // For Debugging: Set this variable to true to simulate a failure to find the registry key

    #[cfg(target_os = "windows")]                                    //acquire steam install from windows registry. ( ͡° ͜ʖ ͡°)
    fn find_hoi4_installation_path() -> Option<PathBuf> {            //perfomance is basically free.

    
        if SIMULATE_REGISTRY_FAILURE {                               //assumes that most people will have a valid hoi4 directory in the same location as steam
            return None;
        }
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

    //pub(crate) fn scan_directory(path: &Path) -> Result<Directory, Box<dyn std::error::Error>> {
    //    log::info!("Scanning directory: {}", path.display());
    //    let mut layer = Directory::new();
    //    let path = path.to_string_lossy().replace('\\', "/");
    //    for entry in WalkDir::new(path) {
    //        let entry = entry?;
    //        let entry_path = entry.path().to_string_lossy().replace('\\', "/");
    //        let entry_path = PathBuf::from(entry_path);
    //        if entry_path.is_dir() {
    //            //log::info!("Entering directory: {}", entry_path.to_str().unwrap());
    //            layer.add_directory(entry_path.to_str().unwrap());
    //        } else {
    //            let file_category = super::FileCategory::categorize_file_extension(&entry_path);
    //            match file_category {
    //                FileCategory::Text | FileCategory::GUI | FileCategory::GFX |
    //                FileCategory::Asset | FileCategory::YAML | FileCategory::CSV |
    //                FileCategory::Shader | FileCategory::Lua => {
    //                    let file_name = entry_path.to_str().unwrap();
    //                    let mut file = File::open(&entry_path)?;
    //                    let mut contents = Vec::new();
    //                    file.read_to_end(&mut contents)?;
    //                    layer.add_file(file_name, contents);
    //                }
    //                _ => {}
    //            }
    //            //log::info!("Found file: {}", entry_path.display());
    //        }
    //    }
    //    Ok(layer)
    //}

    pub(crate) fn scan_directory(path: &Path, exclude_criteria: &[&str]) -> Result<Directory, Box<dyn std::error::Error>> {
        log::info!("Scanning directory: {}", path.display());
        let mut layer = Directory::new();
        let path_string = path.to_string_lossy().replace('\\', "/");
        for entry in WalkDir::new(path_string) {
            let entry = entry?;
            let entry_path = entry.path().to_string_lossy().replace('\\', "/");
            let entry_path = PathBuf::from(entry_path);
            // Check if the entry path should be excluded
            if exclude_criteria.iter().any(|&excluded| entry_path.starts_with(excluded)) {
                log::info!("Excluding: {}", entry_path.display());
                continue;
            }
            let relative_path = entry_path.strip_prefix(path)?;
            if entry_path.is_dir() {
                //log::info!("Entering directory: {}", entry_path.to_str().unwrap());
                layer.add_directory(relative_path);
            } else {
                let file_category = FileCategory::categorize_file_extension(&entry_path);
                match file_category {
                    FileCategory::Text | FileCategory::GUI | FileCategory::GFX |
                    FileCategory::Asset | FileCategory::YAML | FileCategory::CSV |
                    FileCategory::Shader | FileCategory::Lua => {
                        let file_name = relative_path.to_str().unwrap();
                        let mut file = File::open(&entry_path)?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        layer.add_file(file_name, contents);
                    }
                    _ => {}
                }
                //log::info!("Found file: {}", entry_path.display());
            }
        }
        Ok(layer)
    }