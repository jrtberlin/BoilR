use std::error::Error;
use std::{
    fmt,
    path::Path,
};
#[cfg(target_os = "windows")]
use std::env::{self};

use steam_shortcuts_util::{parse_shortcuts, shortcut::ShortcutOwned};

pub fn get_shortcuts_for_user(user: &SteamUsersInfo) -> ShortcutInfo {
    let mut shortcuts = vec![];
    let mut new_path = user.shortcut_path.clone();
    if let Some(shortcut_path) = &user.shortcut_path {
        let content = std::fs::read(shortcut_path).unwrap();
        shortcuts = parse_shortcuts(content.as_slice())
            .unwrap()
            .iter()
            .map(|s| s.to_owned())
            .collect();
        println!(
            "Found {} shortcuts , for user: {}",
            shortcuts.len(),
            user.steam_user_data_folder
        );
    } else {
        println!(
            "Did not find a shortcut file for user {}, createing a new",
            user.steam_user_data_folder
        );
        std::fs::create_dir_all(format!("{}/{}", user.steam_user_data_folder, "config")).unwrap();
        new_path = Some(format!(
            "{}/{}",
            user.steam_user_data_folder, "config/shortcuts.vdf"
        ));
    }
    ShortcutInfo {
        shortcuts,
        path: new_path.unwrap(),
    }
}

pub struct ShortcutInfo {
    pub path: String,
    pub shortcuts: Vec<ShortcutOwned>,
}

pub struct SteamUsersInfo {
    pub steam_user_data_folder: String,
    pub shortcut_path: Option<String>,
}

/// Get the paths to the steam users shortcuts (one for each user)
pub fn get_shortcuts_paths() -> Result<Vec<SteamUsersInfo>, Box<dyn Error>> {
    #[cfg(target_os = "windows")]
    let path_string = {
        let key = "PROGRAMFILES(X86)";
        let program_files = env::var(key)?;
        format!(
            "{program_files}//Steam//userdata//",
            program_files = program_files
        )
    };
    #[cfg(target_os = "linux")]
    let path_string = {
        let home = std::env::var("HOME")?;
        format!("{}/.steam/steam/userdata/", home)
    };

    let user_data_path = Path::new(path_string.as_str());
    if !user_data_path.exists() {
        return Result::Err(Box::new(SteamFolderNotFound {
            location_tried: path_string,
        }));
    }
    let user_folders = std::fs::read_dir(&user_data_path)?;
    let users_info = user_folders
        .filter_map(|f| f.ok())
        .map(|folder| {
            let folder_path = folder.path();
            let folder_str = folder_path
                .to_str()
                .expect("We just checked that this was there");
            let path = format!("{}//config//shortcuts.vdf", folder_str);
            let shortcuts_path = Path::new(path.as_str());
            let mut shortcuts_path_op = None;
            if shortcuts_path.exists() {
                shortcuts_path_op = Some(shortcuts_path.to_str().unwrap().to_string());
            }
            SteamUsersInfo {
                steam_user_data_folder: folder_str.to_string(),
                shortcut_path: shortcuts_path_op,
            }
        })
        .collect();
    Ok(users_info)
}

#[derive(Debug)]
struct SteamFolderNotFound {
    location_tried: String,
}

impl fmt::Display for SteamFolderNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not find steam user data at location: {}  Please specify it in the configuration",
            self.location_tried
        )
    }
}

impl Error for SteamFolderNotFound {
    fn description(&self) -> &str {
        self.location_tried.as_str()
    }
}

#[derive(Debug)]
struct SteamUsersDataEmpty {
    location_tried: String,
}

impl fmt::Display for SteamUsersDataEmpty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Steam users data folder is empty: {}  Please specify it in the configuration",
            self.location_tried
        )
    }
}

impl Error for SteamUsersDataEmpty {
    fn description(&self) -> &str {
        self.location_tried.as_str()
    }
}
pub fn get_users_images(user: &SteamUsersInfo) -> Result<Vec<String>, Box<dyn Error>> {
    let grid_folder = Path::new(user.steam_user_data_folder.as_str()).join("config/grid");
    std::fs::create_dir_all(&grid_folder)?;
    let user_folders = std::fs::read_dir(&grid_folder)?;
    let file_names = user_folders
        .filter_map(|image| image.ok())
        .map(|image| image.file_name().into_string().unwrap())
        .collect();
    Ok(file_names)
}