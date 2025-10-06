// initial filescan 
use std::{
    fs::{self, File}, io::Read, path::{Path, PathBuf}, str, time::Instant
    };

use serde::Deserialize;
use jwalk::WalkDir;
use native_dialog::FileDialog;
extern crate winreg;
use winreg::{RegKey, enums::*};

use crate::{parser, utility::error::print_error};
use crate::parser::sharedparsers::truncate_input;

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
    // Accumulate parse durations to compute an average per-scan
    let mut total_parse_duration = std::time::Duration::ZERO;
    let mut parse_count: usize = 0;
    // Store per-file durations for richer statistics and top-N slowest listing
    let mut per_file_durations: Vec<(String, std::time::Duration)> = Vec::new();
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
                        layer.add_file(file_name.as_ref(), contents.clone());
                        // time only the parsing step
                        let start = Instant::now();
                        let errors = layer.parse_file(file_name.as_ref(), string_manager);
                        let elapsed = start.elapsed();
                        total_parse_duration += elapsed;
                        parse_count += 1;
                        per_file_durations.push((file_name.to_string(), elapsed));

                        if !errors.is_empty() {
                            log::error!(
                                "❌ Parsing failed for {} ({} errors):",
                                file_name.as_ref(),
                                errors.len()
                            );
                            let src_str = String::from_utf8_lossy(&contents);
                            for err in &errors {
                                print_error(&src_str, err);
                            }
                        }
                        //match parser::sharedparsers::parse(parsecontents, string_manager) {
                        //    Ok((remaining, parsed_result)) => {
                        //        layer.add_file(file_name.as_ref(), contents.clone(), parsed_result);
                        //        if !remaining.is_empty() {
                        //            log::warn!("Unparsed input remains: {:?}", remaining);
                        //        }
                        //    }
                        //    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                        //        layer.add_file(file_name.as_ref(), contents.clone(), crate::parser::sharedparsers::AllResult::default());
                        //        log::error!("Parsing error in file {:?}: {:?}, kind: {:?}", file_name, truncate_input(e.input, 10), e.code);
                        //    }
                        //    Err(nom::Err::Incomplete(_)) => {
                        //        layer.add_file(file_name.as_ref(), contents.clone(), crate::parser::sharedparsers::AllResult::default());
                        //        log::error!("Parsing incomplete. More data needed.");              
                        //    }
                        //}
                        
                    }
                    FileCategory::Yaml | FileCategory::Lua |
                    FileCategory::Shader   => {
                        // don't parse these files yet, but add them to the layer
                        let mut file = File::open(&entry_path)?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        
                        layer.add_file(file_name.as_ref(), contents.clone());
                    }
                    // Match files that we acknowledge but don't parse
                    FileCategory::Sfx | FileCategory::Map | FileCategory::Image |
                    FileCategory::Mesh | FileCategory::Font | FileCategory::Sound |
                    FileCategory::Other => {
                        // Add file with dummy data
                        layer.add_file(file_name.as_ref(), vec![0]);
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

        // Report statistics if any files were parsed
        if parse_count > 0 {
            let avg_ms = (total_parse_duration.as_secs_f64() * 1000.0) / (parse_count as f64);

            // Prepare a vector of durations in milliseconds for statistics
            let mut durations_ms: Vec<f64> = per_file_durations.iter().map(|(_, d)| d.as_secs_f64() * 1000.0).collect();
            durations_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min = durations_ms.first().copied().unwrap_or(0.0);
            let max = durations_ms.last().copied().unwrap_or(0.0);
            let median = if durations_ms.is_empty() { 0.0 } else {
                let mid = durations_ms.len() / 2;
                if durations_ms.len() % 2 == 0 {
                    (durations_ms[mid - 1] + durations_ms[mid]) / 2.0
                } else {
                    durations_ms[mid]
                }
            };

            // Standard deviation
            let mean = avg_ms;
            let variance = durations_ms.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (durations_ms.len() as f64);
            let stddev = variance.sqrt();

            // Percentiles
            let percentile = |p: f64| -> f64 {
                if durations_ms.is_empty() { return 0.0 }
                let rank = p / 100.0 * ((durations_ms.len() - 1) as f64);
                let lo = rank.floor() as usize;
                let hi = rank.ceil() as usize;
                if lo == hi { durations_ms[lo] } else {
                    let w = rank - (lo as f64);
                    durations_ms[lo] * (1.0 - w) + durations_ms[hi] * w
                }
            };
            let p90 = percentile(90.0);
            let p95 = percentile(95.0);

            log::info!("Parse stats over {} files: avg={:.3} ms, min={:.3} ms, max={:.3} ms, median={:.3} ms, stddev={:.3} ms, p90={:.3} ms, p95={:.3} ms", parse_count, avg_ms, min, max, median, stddev, p90, p95);

            // Top-20 slowest files
            per_file_durations.sort_by(|a, b| b.1.cmp(&a.1));
            let top_n = 20.min(per_file_durations.len());
            log::info!("Top {} slowest parsed files:", top_n);
            for (i, (path, dur)) in per_file_durations.iter().take(top_n).enumerate() {
                let ms = dur.as_secs_f64() * 1000.0;
                let pct = (ms / (total_parse_duration.as_secs_f64() * 1000.0)) * 100.0;
                log::info!("  {:>2}. {:<80} {:>8.3} ms ({:>5.2}%)", i + 1, path, ms, pct);
            }
        } else {
            log::info!("No files were parsed in this scan");
        }

        Ok(layer)
    }