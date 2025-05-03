use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}, time::Instant};

use flexi_logger::{Logger, FileSpec, Duplicate};

mod vfs;
mod utility;
mod parser;
mod scope;
use crate::vfs::{unionfs::*, scanner};




fn write_data_to_file(data: &[u8], filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(data)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with console output and log file
    Logger::try_with_str("info")?
        .log_to_file(FileSpec::default().directory(PathBuf::from("./logs")))
        .duplicate_to_stderr(Duplicate::Info)  
        .format_for_files(flexi_logger::colored_with_thread)
        .start()?;
    
    let mut vfs = Vfs::new();
    let base_game_path = scanner::get_base_game_path()?;
    let ignore_list = vec!["unit_nudger_markers.txt".to_string(),"buildings_nudger_markers.txt".to_string(),"fakegfx2.txt".to_string(),"fakegfx.txt".to_string(),"fake2.txt".to_string(),"fake.txt".to_string(),"console_history.txt".to_string(),"interface/credits.txt".to_string(), "licenses.txt".to_string(),"changelog.txt".to_string(), "dowser.exe".to_string(), "tbb.dll".to_string(), "tbb_debug.dll".to_string(), "nakama-cpp.dll".to_string(), "pops_api.dll".to_string(), "steam_api64.dll".to_string(), "PDXBrowser_IPC.dll".to_string(), "ThirdPartyLicenses.txt".to_string(), "launcher-settings.json".to_string(), "EmptySteamDepot".to_string(), "pdx_browser".to_string(), "pdx_launcher".to_string(), "tools".to_string(), "wiki".to_string(), "launcher-assets".to_string(), "crash_reporter".to_string(), "_CommonRedist".to_string(), "browser".to_string(), "cef".to_string(), "Documents".to_string(), "dlc_metadata".to_string(), "pdx_online_assets".to_string()];

    let start_time = Instant::now();
    vfs.add_layer(scanner::scan_directory(vfs.get_string_manager(), &base_game_path, &ignore_list).expect("Failed to scan directory for vfs setup"), base_game_path);
    log::info!("Scanned in {:?}", start_time.elapsed());

    match vfs.read_dir(PathBuf::from("common")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    //pyo3::prepare_freethreaded_python();
    // Test reading a file and writing it to a file
    let start_time = Instant::now();
    match vfs.read_parseresult(PathBuf::from("common/units/names_divisions/FIN_names_divisions.txt")) {     
        Ok(parsed_result) => {
            // Convert parsed data to a string or structured output
            let output_string = format!("{:#?}", parsed_result);
            // Write parsed output to a file
            fs::write("output_file_parser.txt", output_string)?;
            log::info!("Parsing successful: check output_file_parser.txt");
            log::info!("Parsed in {:?}", start_time.elapsed());
            }
        Err(err) => log::info!("Error reading file: {:?}", err),
    }
    // Example usage: get file metadata
    let start_time = Instant::now();
    let (size, category) = vfs.file_metadata(PathBuf::from("common/script_enums.txt"))?;
    log::info!("Test Node found in {:?}", start_time.elapsed());
    log::info!("File size: {} bytes", size);
    log::info!("File category: {:?}", category);


    let mod_path = Path::new("c:/users/afrey/documents/github/cg-black-requiem");
    let mod_ignore_list = vec![".vscode".to_string(), ".gitattributes".to_string(), ".git".to_string(), ".gitignore".to_string()];
    let start_time = Instant::now();
    vfs.add_layer(scanner::scan_directory(vfs.get_string_manager(), mod_path, &mod_ignore_list).expect("Failed to scan directory for vfs setup"), mod_path.to_path_buf());
    log::info!("Scanned in {:?}", start_time.elapsed());

    match vfs.read_dir(PathBuf::from("common/decisions")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    match vfs.read_dir(PathBuf::from("common/decisions/categories")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    match vfs.read_dir(PathBuf::from("gfx")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    // Test reading a file and writing it to a file
    let start_time = Instant::now();
    match vfs.read_file(PathBuf::from("common/script_enums.txt")) {     
        Ok(data) => {
            log::info!("Test Node found in {:?}", start_time.elapsed());
            if let Err(err) = write_data_to_file(&data, "output_file2.txt") {
                log::error!("Error writing to file: {:?}", err);
            } else {
                log::info!("Data successfully written to file.");
            }
        },
        Err(err) => log::info!("Error reading file: {:?}", err),
    }

    // Example usage: get file metadata
    let start_time = Instant::now();
    let (size, category) = vfs.file_metadata(PathBuf::from("common/national_focus/area.txt"))?;
    log::info!("Test Node found in {:?}", start_time.elapsed());
    log::info!("File size: {} bytes", size);
    log::info!("File category: {:?}", category);

    match vfs.read_dir(PathBuf::from("")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    Ok(())
}