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
    let ignore_list = vec!["EmptySteamDepot", "tbb.dll", "pdx_browser", "pdx_launcher", "tools", "wiki", "launcher-assets", "crash_reporter", "_CommonRedist", "browser", "cef", "Documents", "PDXBrowser_IPC.dll", "ThirdPartyLicenses.txt"];
    let start_time = Instant::now();
    vfs.add_layer(Box::new(scanner::scan_directory(&base_game_path, &ignore_list).expect("Failed to scan directory for VFS setup")), base_game_path);
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
    let mod_ignore_list = vec![".vscode", ".gitattributes", ".git", ".gitignore", "tbb.dll", "pdx_browser", "pdx_launcher", "tools", "wiki", "launcher-assets", "crash_reporter", "_CommonRedist", "browser", "cef", "Documents", "PDXBrowser_IPC.dll", "ThirdPartyLicenses.txt"];
    let start_time = Instant::now();
    vfs.add_layer(Box::new(scanner::scan_directory(mod_path, &mod_ignore_list).expect("Failed to scan directory for VFS setup")), mod_path);
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

    Ok(())
}


/* Let the Cycle be Discontinued
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMX;     ...';cox0XWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMX;             .',:oxOXWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMK;                    .;ld0NMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMK,                        .'cxKWMMMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMWNNXXk'                            .;o0WMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMO;'....:ll,                           .;dKWMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMWXKNMMMMMMMMMMx.    '0WWO.                             .cONMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMWXkl;..:KMMMMMMMMMx.    '0MMK,            .:l;.              .;kNMWMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMNk:.      'OWMMMMMMWd.    '0MMN:           'kWMWKkl'              ,xNMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMM0;          .dK0kdl:;.     '0MMWKkxoc;'.. .cKMMMMWWMNc               ;kWMMMMMMMMMMM
MMMMMMMMMMMMMMMMMXl.           ..           '0MMMMMMMMWNK0kONMMMMMMMWk.                .cKWMMMMMMMMM
MMMMMMMMMMMMMMMMMMWx.                       '0MMMMMMMMMMMMMMMMMMMMMMO'                   'kWMMMMMMMM
MMMMMMMMMMMMMMMMMMMWk.               ...',;..lkO0XNNWMMMMMMMMMMMMMMK;                     .lXMMMMMMM
MMMMMMMWNNWMMMMMMMNO:.          .;ldO0XNWWWd.   ...';cdkXWWMMMMMMMMNx,                      ;KMMMMMM
MMMMMMNd''cx0NMMNk;          ,lOXWMMMMMMMMWd.           .;dKWMMMMMMMMXx'       .',.          ,0MMMMM
MMMMMK:     .,ld:         .:kNMMMMMMMMMMMMMx.              .:OWMMMMMMMMXx,.':lx0NWO,          ,0MMMM
MMMMK;                   ;OWMMMMMMMMMMMMMMMx.                .cKMMMMMMMMMNKNMMMMMMMK:          ,KMMM
MMMX;                   lXWMMMMMMMMMMMMMMMMx.                  ,OWMMMMMMMMMMMMMMMMMWXc          :XMM
MMMk.                  lNMMMMMMMMMMMMMMMMMMx.                   '0MMMMMMMMMMMMMMMMMNO:.          oWM
MMMNOo;.              ;XMMMMMMMMMMMMMMMMMMMx.                    :XMMMMMMMMMMMMWXkl,.            .OM
MMMMMMWKx,            dMMMMMMMMMMMMMMMMMMMMx.                    .kMMMMMMMMMMMWx.                 :N
MMMMMMMMNc           .kMMMMMMMMMMMMMMMMMMMMx.                     dMMMMMMMMMMMMx.                 .k
MMMMMMMMk.           .OMMMMMMMMMMMMMMMMMMMMx.                     oMMMMMMMMMMMMN:                  c
MMMMMMMNc            .kMMMMMMMMMMMMMMMMMMMMx.                    .xMMMMMMMMMMMMMO.                 '
0Okxxddc.             oWMMMMMMMMMMMMMMMMMMMx.                    .OMMMMMMMMMMMMMWo.                .
                      ;KMMMMMMMMMMMMMMMMMMMx.                    :NMMMMMMMMMMMMMMN0kkkkkko.         
                      .xWMMMMMMMMMMMMMMMMMMx.                   .xMMMMMMMMMMMMMMMMMMMWMMM0,         
                       ;KMMMMMMMMMMMMMMMMMMx.                   :NMMMMMMMMMMMMMMMMMMMMMMMK,         
                        dMMMMMMMMMMMMMMMMMMx.                  .kMMMMMMMMMMMMMMMMMMMMMMMMK,         
.                       lWMMMWWMMMMMMMMMMMMx.                  '0MMMMMMMMMMMMMMMMNKOOOOOOd.         
0OOkkxxl.              ;KMMWN0O0XNWMMMMMMMMx.          ...'.    :XMMMMMMMMMMMMMMWd.                .
MMMMMMMN:             .OMMWk'. ..';cok0NMMMx.   .';:ldk0KXNXo.   oWMMMMMMMMMMMMMO.                 '
MMMMMMMMk.            ,KMMNc          .,kWMx. .l0NWWMMMMMWMM0,   :NMMMMMMMMMMMMX:                  c
MMMMWMMMNc            .OWMM0;        .,oKWMk.  'oOKNMMMMWMWK:    oWMMMMMMMMMMMWd.                 .k
MMMMMMWXk:             cXMMMNkc;;:cox0NMMW0xl.   ..'codxxdc.    ,KMMMMMMMMMMMMWd.                 :X
MMMNOo;.                lNMMMMMMMMMMMMMWWx.cN0,                'OWMMMMMMMMMMMMMWKd:.             .kM
MMMk.                   .cXMMMMMMMMMMMMM0, :0Wx.              ,0WMMMMMMMMMMMMMMMMMWKx,           lNM
MMMX:                     'dOkxodKMMMMMM0lxo':c.       ,c:,',oXMMMMMMMMMMMMMMMMMMMMMX:          :XMM
MMMMK;                          '0MMMMMMWWMx.          lWMWNWWMMMMMMMMMMMNKNWMMMMMMK:          ,0MMM
MMMMMXc.    .;dx;               :NMMMMMMMMWx.          ,KMMMWMMMMMMMMMWXd,.'cdONMWO,          '0MMMM
MMMMMMWk;'ckXWMMNk;             oWXKWMWXNMXd'  .    .. .OMMMMMMMMMMWWKl.      .':c.          ,0MMMMM
MMMMMMMMNNWMMMMMMMNk;          .xK:'OMKcxMxll..o;   lK:.xMMMMMMMMMMXl.                      ;KWMMMMM
MMMMMMMMMMMMMMMMMMMNd.          .'  ';'..;..cxOXKxxkXWKOXMMMMMMMMMMXc                     .lXMMMMMMM
MMMMMMMMMMMMMMMMMMNl.                       :NMMMMMMMMWMMMMMMMMMMMMMXl                   'kNMMMMMMMM
MMMMMMMMMMMMMMMMMX:            .            :NMMMMMMMWWX0OxONMMMMMMMMNl.               .cKMMMMMMMMMM
MMMMMMMMMMMMMMMMM0;          'kKOxoc;,.     :NMMW0dol:'..  .lXMMMMMMMNk'              ;OWMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMNOl,.     ;0WMMMMMMWd.    :NMMN:           ;KMMWXkl,.             ,xNMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMWN0d:..:KMMMMMMMMMx.    :NMMN:            'oo:.               ,xNMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMXKNMMMMMMMMMMx.    :NMMN:                             .:kNMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM0:'...'looc.                          .;dKWMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMWNXXO,                            .;oONMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMX;                        .'cdKWMWMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMX;                    .,cd0NMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMX;             ..,:lxOXWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMX;      ..',:lxOXWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
*/