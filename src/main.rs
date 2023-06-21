mod lemmy;
mod profile;

use lemmy_api_common::site;
use slint::Weak;
use slint::SharedString;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use futures::executor::block_on;

slint::include_modules!();

const PROFILE_FILENAME: &str = "profile_settings.json";

struct ProcessingInstruction {
    instruction_type: SharedString,
    instance: SharedString,
    username: SharedString,
    password: SharedString,
}

fn write_profile(profile_settings: &site::GetSiteResponse, mut logger: impl FnMut(String)) {
    let path = Path::new(PROFILE_FILENAME);
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            logger(format!("ERROR: Cannot write file - {}: {}", path.display(), e));
            return
        }
    };

    let json_string = serde_json::to_string_pretty(&profile_settings);
    match file.write_all(format!("{}", json_string.unwrap()).as_bytes()) {
        Ok(_) => {
            logger(format!("Wrote Profile to: {}", path.to_str().unwrap()))
        },
        Err(e) => {
            logger(format!("ERROR: Cannot write file - {}: {}", path.display(), e));
        }
    }
}

#[tokio::main]
async fn process_download(processing_instruction: ProcessingInstruction, mut logger: impl FnMut(String)) {
    // Fetch data from UI
    let instance = processing_instruction.instance.to_string();
    let username = processing_instruction.username.to_string();
    let password = processing_instruction.password.to_string();
    logger(format!("Logging in as {}", username));

    let api = lemmy::API::new();

    // Login
    let jwt_token_future = api.login(&instance, &username, &password);
    let jwt_token_result = block_on(jwt_token_future);
    if jwt_token_result.is_err() {
        logger(format!("ERROR: Failed Login - {}", jwt_token_result.unwrap_err()));
        return;
    }

    let jwt_token = jwt_token_result.unwrap();
    logger("Login Successful.".to_string());

    // Fetch Profile
    let profile_settings_future = api.fetch_profile_settings(&instance, &jwt_token);
    let profile_settings_result = block_on(profile_settings_future);
    if profile_settings_result.is_err() {
        logger(format!("ERROR: Failed to fetch Profile - {}", profile_settings_result.unwrap_err()));
        return;
    }
    let profile_settings = profile_settings_result.unwrap();
    logger("Profile retrieved!".to_string());

    // Write to File
    write_profile(&profile_settings, logger);
    
}

fn read_profile() -> Result<site::GetSiteResponse, String> {
    let path = Path::new(PROFILE_FILENAME);
    let profile_json_result = std::fs::read_to_string(path);
    let profile_json = match profile_json_result {
        Ok(file) => file,
        Err(_) => return Err("ERROR: Failed to open profile settings!".to_string()),
    };

    let profile_settings_result: Result<site::GetSiteResponse, serde_json::Error> = serde_json::from_slice(profile_json.as_bytes());
    let profile_settings = match profile_settings_result {
        Ok(profile) => profile,
        Err(e) => return Err(format!("ERROR: Failed to parse profile JSON - {}", e)),
    };

    return Ok(profile_settings);
}

#[tokio::main]
async fn process_upload(processing_instruction: ProcessingInstruction, mut logger: impl FnMut(String)) {
    // Read original profile
    let original_profile = match read_profile() {
        Ok(profile) => profile,
        Err(e) => {
            logger(format!("{}", e).to_string());
            return;
        },
    };

    // Fetch data from UI
    let instance = processing_instruction.instance.to_string();
    let username = processing_instruction.username.to_string();
    let password = processing_instruction.password.to_string();
    logger(format!("Logging in as {}", username));

    let api = lemmy::API::new();

    // Login
    let jwt_token_future = api.login(&instance, &username, &password);
    let jwt_token_result = block_on(jwt_token_future);
    if jwt_token_result.is_err() {
        logger(format!("ERROR: Failed Login - {}", jwt_token_result.unwrap_err()));
        return;
    }

    let jwt_token = jwt_token_result.unwrap();
    logger("Login Successful.".to_string());

    // Fetch New Profile
    let new_profile_future = api.fetch_profile_settings(&instance, &jwt_token);
    let new_profile_result = block_on(new_profile_future);
    if new_profile_result.is_err() {
        logger(format!("ERROR: Failed to fetch Porfile - {}", new_profile_result.unwrap_err()));
        return;
    }
    let new_profile = new_profile_result.unwrap();
    logger("Existing Settings Downloaded. Calculating delta...".to_string());

    // Calculating Differences
    let profile_changes = profile::calculate_changes(&original_profile, &new_profile);
    logger("All profile settings from the original profile will be applied.".to_string());
    logger(format!("{} new users will be blocked", profile_changes.users_to_block.len()));
    logger(format!("{} new communities will be blocked", profile_changes.communities_to_block.len()));
    logger(format!("{} new communities will be followed", profile_changes.communities_to_follow.len()));

    // Call API to actually apply changes to new account

    for blocked_user_string in profile_changes.users_to_block {
        // TODO: Call API to fetch user (to get ID)
        // TODO: Call API to block user
    }
    
    for blocked_community_string in profile_changes.communities_to_block {
        // TODO: Call API to fetch community (to get ID)
        // TODO: Call API to block communities
    }
    
    for follow_community_string in profile_changes.communities_to_follow {
        // TODO: Call API to fetch community (to get ID)
        // TODO: Call API to follow communities
    }
    
    // TODO: Call API to apply settings
}

fn main() {
    // Setup processing thread communication
    let (instruct_tx, instruct_rx): (Sender<ProcessingInstruction>, Receiver<ProcessingInstruction>) = mpsc::channel();
    let instruct_tx_copy = instruct_tx.clone();

    // Construct Slint App
    let app = App::new().unwrap();
    let app_weak: Weak<App> = app.as_weak();
    let app_weak_clone = app_weak.clone();

    // Main instruction processing thread
    let main_thread = thread::spawn(move || {
        loop {
            let processing_instruction = instruct_rx.recv().unwrap();
            
            if processing_instruction.instruction_type == "Done" {
                break;
            } else if processing_instruction.instruction_type == "Download" {
                // Closure madness: create a logger closure for updating the UI
                let app_copy = app_weak.clone();
                let logger = |text: String| {
                    let app_internal_copy = app_copy.clone();
                    slint::invoke_from_event_loop(move || {
                        let original_text = app_internal_copy.unwrap().get_download_log_output();
                        let new_text = format!("{}{}\n", original_text, text);
                        app_internal_copy.unwrap().set_download_log_output(new_text.into())
                    }).unwrap();
                };

                process_download(processing_instruction, logger);

                slint::invoke_from_event_loop(move || {
                    app_copy.unwrap().set_download_ui_enabled(true);
                }).unwrap();
            } else {
                // Closure madness: same thing but for uploading
                let app_copy = app_weak.clone();
                let logger = |text: String| {
                    let app_internal_copy = app_copy.clone();
                    slint::invoke_from_event_loop(move || {
                        let original_text = app_internal_copy.unwrap().get_upload_log_output();
                        let new_text = format!("{}{}\n", original_text, text);
                        app_internal_copy.unwrap().set_upload_log_output(new_text.into())
                    }).unwrap();
                };

                process_upload(processing_instruction, logger);

                slint::invoke_from_event_loop(move || {
                    app_copy.unwrap().set_upload_ui_enabled(true);
                }).unwrap();
            }
        }
    });

    // Bind thread action to Click Event
    app.global::<ControlPageHandler>().on_clicked({
        move |window_type| {            
            if window_type == "Download" {
                app_weak_clone.unwrap().set_download_log_output("".into());
                app_weak_clone.unwrap().set_download_ui_enabled(false);

                let download_instruction = ProcessingInstruction {
                    instruction_type: window_type,
                    instance: app_weak_clone.unwrap().get_download_instance_url(),
                    username: app_weak_clone.unwrap().get_download_username_input(),
                    password: app_weak_clone.unwrap().get_download_password_input(),
                };

                instruct_tx.send(download_instruction).unwrap();
            } else {
                app_weak_clone.unwrap().set_upload_log_output("".into());
                app_weak_clone.unwrap().set_upload_ui_enabled(false);

                let upload_instruction = ProcessingInstruction {
                    instruction_type: window_type,
                    instance: app_weak_clone.unwrap().get_upload_instance_url(),
                    username: app_weak_clone.unwrap().get_upload_username_input(),
                    password: app_weak_clone.unwrap().get_upload_password_input(),
                };

                instruct_tx.send(upload_instruction).unwrap();
            }
        }
    });

    // Run GUI application
    app.run().unwrap();

    // Cleanup
    instruct_tx_copy.send(ProcessingInstruction {
        instruction_type: "Done".into(),
        instance: "".into(),
        username: "".into(),
        password: "".into(),
    }).unwrap();
    main_thread.join().unwrap();
}
