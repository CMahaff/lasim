#![windows_subsystem = "windows"]

mod lemmy;
mod profile;
mod migrations;

use slint::Weak;
use slint::SharedString;
use url::Url;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use futures::executor::block_on;

slint::include_modules!();

const PANIC_LOG: &str = "error.log";

struct ProcessingInstruction {
    instruction_type: SharedString,
    instance: SharedString,
    username: SharedString,
    password: SharedString,
    two_factor_token: SharedString,
}

fn write_panic_info(info: &String) {
    let path = Path::new(PANIC_LOG);
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(_) => return
    };

    file.write_all(info.as_bytes()).ok();
}

fn evaluate_two_factor_token(token: &String) -> Result<Option<String>, &str> {
    if token.is_empty() {
        return Ok(None);
    }

    if token.chars().count() != 6 {
        return Err("2FA Token should be 6 characters")
    }

    match token.parse::<u32>() {
        Ok(_) => Ok(Some(token.clone())),
        Err(_) => Err("2FA Token should be a number"),
    }
}

fn write_profile(profile_local: &profile::ProfileConfiguration, mut logger: impl FnMut(String)) {
    let profile_filename = migrations::profile_migrate::get_latest_profile_name();
    let path = Path::new(profile_filename.as_str());
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            logger(format!("ERROR: Cannot write file - {}: {}", path.display(), e));
            return
        }
    };

    let json_string = serde_json::to_string_pretty(&profile_local);
    match file.write_all(json_string.unwrap().as_bytes()) {
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
    let mut instance = processing_instruction.instance.to_string();
    let username = processing_instruction.username.to_string();
    let password = processing_instruction.password.to_string();
    let two_factor_token = match evaluate_two_factor_token(&processing_instruction.two_factor_token.to_string()) {
        Ok(token) => token,
        Err(e) => {
            logger(format!("ERROR: Invalid 2FA Token - {}", e));
            return;
        },
    };

    if !instance.starts_with("http") {
        instance.insert_str(0, "https://");
    }
    let instance_url_result = Url::parse(instance.as_str());
    if instance_url_result.is_err() {
        logger("ERROR: Invalid Instance URL".to_string());
        return;
    }
    let instance_url = instance_url_result.unwrap();

    let api = lemmy::api::Api::new(instance_url);

    // Login
    logger(format!("Logging in as {}", username));
    let jwt_token_future = api.login(&username, &password, two_factor_token);
    let jwt_token_result = block_on(jwt_token_future);
    if jwt_token_result.is_err() {
        logger(format!("ERROR: Failed Login - {}", jwt_token_result.unwrap_err()));
        return;
    }

    let jwt_token = jwt_token_result.unwrap();
    logger("Login Successful.".to_string());

    // Fetch Profile
    let profile_settings_future = api.fetch_profile_settings(&jwt_token);
    let profile_settings_result = block_on(profile_settings_future);
    if profile_settings_result.is_err() {
        logger(format!("ERROR: Failed to fetch Profile - {}", profile_settings_result.unwrap_err()));
        return;
    }
    let profile_settings = profile_settings_result.unwrap();
    logger("Profile retrieved!".to_string());

    // Convert Profile
    let profile_local = profile::construct_profile(&profile_settings);

    // Write to File
    write_profile(&profile_local, logger);
    
}

fn read_profile() -> Result<profile::ProfileConfiguration, String> {
    return migrations::profile_migrate::read_latest_profile();
}

#[tokio::main]
async fn process_upload(processing_instruction: ProcessingInstruction, mut logger: impl FnMut(String)) {
    // Read original profile
    let original_profile = match read_profile() {
        Ok(profile) => profile,
        Err(e) => {
            logger(e);
            return;
        },
    };

    // Fetch data from UI
    let mut instance = processing_instruction.instance.to_string();
    let username = processing_instruction.username.to_string();
    let password = processing_instruction.password.to_string();
    let two_factor_token = match evaluate_two_factor_token(&processing_instruction.two_factor_token.to_string()) {
        Ok(token) => token,
        Err(e) => {
            logger(format!("ERROR: Invalid 2FA Token - {}", e));
            return;
        },
    };

    if !instance.starts_with("http") {
        instance.insert_str(0, "https://");
    }
    let instance_url_result = Url::parse(instance.as_str());
    if instance_url_result.is_err() {
        logger("ERROR: Invalid Instance URL".to_string());
        return;
    }
    let instance_url = instance_url_result.unwrap();

    let api = lemmy::api::Api::new(instance_url);

    // Login
    logger(format!("Logging in as {}", username));
    let jwt_token_future = api.login(&username, &password, two_factor_token);
    let jwt_token_result = block_on(jwt_token_future);
    if jwt_token_result.is_err() {
        logger(format!("ERROR: Failed Login - {}", jwt_token_result.unwrap_err()));
        return;
    }

    let jwt_token = jwt_token_result.unwrap();
    logger("Login Successful.".to_string());

    // Fetch New Profile
    let new_profile_future = api.fetch_profile_settings(&jwt_token);
    let new_profile_result = block_on(new_profile_future);
    if new_profile_result.is_err() {
        logger(format!("ERROR: Failed to fetch Porfile - {}", new_profile_result.unwrap_err()));
        return;
    }
    let new_profile_api = new_profile_result.unwrap();
    logger("Existing Settings Downloaded. Calculating delta...".to_string());

    // Convert
    let new_profile = profile::construct_profile(&new_profile_api);

    // Calculating Differences
    let profile_changes = profile::calculate_changes(&original_profile, &new_profile);
    logger("All profile settings from the original profile will be applied.".to_string());
    logger(format!("{} new users will be blocked", profile_changes.blocked_users.len()));
    logger(format!("{} new communities will be blocked", profile_changes.blocked_communities.len()));
    logger(format!("{} new communities will be followed", profile_changes.followed_communities.len()));

    // Call API to actually apply changes to new account

    // Account for Rate Limits - values get mapped as seen here: lemmy/src/api_routes_http.rs
    let message_per_second = new_profile_api.site_view.local_site_rate_limit.message_per_second;
    let message_rate_limit = std::time::Duration::from_millis((1000f64 / message_per_second as f64).ceil() as u64);

    // Block Users
    for blocked_user_string in profile_changes.blocked_users {
        let user_details_result = api.fetch_user_details(&jwt_token, &blocked_user_string).await;
        thread::sleep(message_rate_limit);

        if user_details_result.is_err() {
            logger(format!("Cannot find user {} to block, got exception {}",
                           blocked_user_string,
                           user_details_result.unwrap_err()));
            continue;
        }

        let id_to_block = user_details_result.unwrap().person_view.person.id;
        let block_user_result = api.block_user(&jwt_token, id_to_block).await;
        thread::sleep(message_rate_limit);

        match block_user_result {
            Ok(response) => {
                if !response.blocked {
                    format!("Server refused to block user {}", blocked_user_string);
                }
            }
            Err(e) => logger(format!("Got exception blocking user {}: {}", blocked_user_string, e)),
        }
    }
    
    // Block Communities
    for blocked_community_string in profile_changes.blocked_communities {
        let community_details_result = api.fetch_community_by_name(&jwt_token, &blocked_community_string).await;
        thread::sleep(message_rate_limit);

        if community_details_result.is_err() {
            logger(format!("Cannot find community {} to block, got exception {}",
                           blocked_community_string,
                           community_details_result.unwrap_err()));
            continue;
        }
        
        let id_to_block = community_details_result.unwrap().community_view.community.id;
        let block_community_result = api.block_community(&jwt_token, id_to_block).await;
        thread::sleep(message_rate_limit);

        match block_community_result {
            Ok(response) => {
                if !response.blocked {
                    format!("Server refused to block community {}", blocked_community_string);
                }
            }
            Err(e) => logger(format!("Got exception blocking community {}: {}", blocked_community_string, e)),
        }
    }
    
    // Follow Communities
    for follow_community_string in profile_changes.followed_communities {
        let community_details_result = api.fetch_community_by_name(&jwt_token, &follow_community_string).await;
        thread::sleep(message_rate_limit);

        if community_details_result.is_err() {
            logger(format!("Cannot find community {}, got exception {}",
                           follow_community_string,
                           community_details_result.unwrap_err()));
            continue;
        }

        let id_to_follow = community_details_result.unwrap().community_view.community.id;
        let follow_community_result = api.follow_community(&jwt_token, id_to_follow).await;
        thread::sleep(message_rate_limit);

        match follow_community_result {
            Ok(response) => {
                if response.community_view.subscribed == lemmy_api_common::lemmy_db_schema::SubscribedType::NotSubscribed {
                    format!("Server refused to follow community {}", follow_community_string);
                }
            }
            Err(e) => logger(format!("Got exception following community {}: {}", follow_community_string, e)),
        }
    }
    
    // Save profile settings
    let save_settings_result = api.save_user_settings(&jwt_token, profile_changes.profile_settings).await;
    if save_settings_result.is_err() {
        logger(format!("Cannot save profile settings, got exception {}", save_settings_result.unwrap_err()));
    }

    logger("Finished!".to_string());
}

fn main() {
    // Setup some kind of logging for if we crash
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            write_panic_info(&format!("Unexpected Error Occurred: {s:?}"));
        } else {
            write_panic_info(&"Unknown Error Occurred!".to_string());
        }
        panic_hook(panic_info);
        std::process::exit(1);
    }));

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
                    two_factor_token: app_weak_clone.unwrap().get_download_two_factor_input(),
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
                    two_factor_token: app_weak_clone.unwrap().get_upload_two_factor_input(),
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
        two_factor_token: "".into(),
    }).unwrap();
    main_thread.join().unwrap();
}
