use crate::profile;
use crate::migrations;
use std::path::Path;

const PROFILE_FILENAME_START: &str = "profile_v";
const PROFILE_FILENAME_END: &str = ".json";
const PROFILE_CURRENT_VERSION: u16 = 2;

pub fn read_latest_profile() -> Result<profile::ProfileConfiguration, String> {
    // Identify latest profile version
    let mut latest_profile_version: u16 = 0;
    let directory_items = std::fs::read_dir("./").unwrap();
    for item in directory_items {
        if item.is_ok() {
            let dir_entry = item.unwrap();
            let metadata = std::fs::metadata(dir_entry.path());
            if metadata.is_ok_and(|m| m.is_file()) {
                let filename = String::from(dir_entry.file_name().to_str().unwrap());
                if filename.starts_with(PROFILE_FILENAME_START) && filename.ends_with(PROFILE_FILENAME_END) {
                    let end_index = filename.find(PROFILE_FILENAME_END).unwrap();
                    let profile_version_string = filename.get(PROFILE_FILENAME_START.char_indices().count()..end_index);
                    let profile_version = profile_version_string.unwrap().parse::<u16>().unwrap();

                    if profile_version > latest_profile_version {
                        latest_profile_version = profile_version;
                    }
                }
            }
        }
    }

    // Convert as necessary
    let mut profile_v1: Option<migrations::migrate_v1_to_v2::ProfileConfigurationV1> = None;
    let mut profile_v2: Option<profile::ProfileConfiguration> = None;

    if latest_profile_version == 1 {
        let read_result = migrations::migrate_v1_to_v2::read_profile();
        profile_v1 = match read_result {
            Ok(profile) => Some(profile),
            Err(e) => return Err(e),
        };
        latest_profile_version = 2;
    }

    if latest_profile_version == PROFILE_CURRENT_VERSION {
        if profile_v1.is_some() {
            profile_v2 = Some(migrations::migrate_v1_to_v2::convert_profile(profile_v1.unwrap()));
        } else {
            let filename = format!("profile_v{}.json", PROFILE_CURRENT_VERSION);
            let path = Path::new(filename.as_str());
            let profile_json_result = std::fs::read_to_string(path);
            let profile_json = match profile_json_result {
                Ok(file) => file,
                Err(_) => return Err(format!("ERROR: Failed to open {}", filename)),
            };

            let profile_local_result: Result<profile::ProfileConfiguration, serde_json::Error> = serde_json::from_slice(profile_json.as_bytes());
            let profile_local = match profile_local_result {
                Ok(profile) => profile,
                Err(e) => return Err(format!("ERROR: Failed to parse {} JSON - {}", filename, e)),
            };

            profile_v2 = Some(profile_local);
        }
    }

    if profile_v2.is_some() {
        return Ok(profile_v2.unwrap());
    } else {
        return Err("ERROR: No saved profiles found. Use download option first!".to_string());
    }
}

pub fn get_latest_profile_name() -> String {
    return format!("{}{}{}", PROFILE_FILENAME_START, PROFILE_CURRENT_VERSION, PROFILE_FILENAME_END);
}