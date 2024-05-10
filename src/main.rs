use std::{fs::File, io::Write, path::{Path, PathBuf}, time::Instant};

use flexi_logger::{Logger, FileSpec, Duplicate};

mod vfs;
use crate::vfs::{filesystem::*, unionfs::*, scanner};


fn write_data_to_file(data: &[u8], filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(data)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with console output and log file
    Logger::try_with_str("info")?
        .log_to_file(FileSpec::default().directory(PathBuf::from(".")))
        .duplicate_to_stderr(Duplicate::Info)  
        .format_for_files(flexi_logger::colored_with_thread)
        .start()?;
    
    let mut vfs = UnionFileSystem::new();
    let base_game_path = scanner::get_base_game_path()?;
    let ignore_list = vec!["dowser.exe", "tbb.dll", "tbb_debug.dll", "nakama-cpp.dll", "pops_api.dll", "steam_api64.dll", "PDXBrowser_IPC.dll", "ThirdPartyLicenses.txt", "launcher-settings.json", "EmptySteamDepot", "pdx_browser", "pdx_launcher", "tools", "wiki", "launcher-assets", "crash_reporter", "_CommonRedist", "browser", "cef", "Documents"];
    
    let start_time = Instant::now();
    vfs.add_layer(scanner::scan_directory(&base_game_path, &ignore_list).expect("Failed to scan directory for VFS setup"), base_game_path);
    log::info!("Scanned in {:?}", start_time.elapsed());

    match vfs.read_dir(Path::new("common")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }
    // Test reading a file and writing it to a file
    let start_time = Instant::now();
    match vfs.read_file(Path::new("common/script_enums.txt")) {     
        Ok(data) => {
            log::info!("Test Node found in {:?}", start_time.elapsed());
            if let Err(err) = write_data_to_file(&data, "output_file.txt") {
                log::error!("Error writing to file: {:?}", err);
            } else {
                log::info!("Data successfully written to file.");
            }
        },
        Err(err) => log::info!("Error reading file: {:?}", err),
    }
    // Example usage: get file metadata
    let start_time = Instant::now();
    let (size, category) = vfs.file_metadata(Path::new("common/script_enums.txt"))?;
    log::info!("Test Node found in {:?}", start_time.elapsed());
    log::info!("File size: {} bytes", size);
    log::info!("File category: {:?}", category);


    let mod_path = Path::new("c:/users/afrey/documents/github/cg-black-requiem");
    let mod_ignore_list = vec![".vscode", ".gitattributes", ".git", ".gitignore"];
    let start_time = Instant::now();
    vfs.add_layer(scanner::scan_directory(mod_path, &mod_ignore_list).expect("Failed to scan directory for VFS setup"), mod_path);
    log::info!("Scanned in {:?}", start_time.elapsed());

    match vfs.read_dir(Path::new("common/decisions")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    match vfs.read_dir(Path::new("common/decisions/categories")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    match vfs.read_dir(Path::new("gfx")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    // Test reading a file and writing it to a file
    let start_time = Instant::now();
    match vfs.read_file(Path::new("common/script_enums.txt")) {     
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
    let (size, category) = vfs.file_metadata(Path::new("common/national_focus/area.txt"))?;
    log::info!("Test Node found in {:?}", start_time.elapsed());
    log::info!("File size: {} bytes", size);
    log::info!("File category: {:?}", category);

    match vfs.read_dir(Path::new("")) {
        Ok(contents) => log::info!("Directory contents: {:?}", contents),
        Err(err) => log::info!("Error listing directory: {:?}", err),
    }

    Ok(())
}