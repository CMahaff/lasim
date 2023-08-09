#![windows_subsystem = "windows"]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

mod lemmy;
mod profile;
mod migrations;

use lemmy::typecast::FromAPI;
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

const CONFIG_FILENAME: &str = ".lasim_config.json";
const PANIC_LOG: &str = "error.log";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct GlobalSettings {
    pub upload_profile_settings: bool,
    pub upload_community_subs: bool,
    pub upload_community_blocks: bool,
    pub upload_user_blocks: bool,
    pub upload_user_saved_posts: bool,
    pub sync_removals: bool,
    pub confirm_uploads: bool,
    pub write_api_profiles: bool,
}

#[derive(Debug)]
struct ProcessingInstruction {
    instruction_type: SharedString,
    instance: SharedString,
    username: SharedString,
    password: SharedString,
    two_factor_token: SharedString,
    global_settings: GlobalSettings,
}

fn read_global_settings() -> Result<GlobalSettings, String> {
    let home_directory = match home::home_dir() {
        Some(home_dir) => home_dir,
        None => return Err("Cannot identify home directory.".to_string()),
    };
    let config_path = home_directory.join(CONFIG_FILENAME);
    let config_json_result = std::fs::read_to_string(config_path);
    let config_json = match config_json_result {
        Ok(file) => file,
        Err(_) => return Err("Cannot read config file.".to_string()),
    };

    let config_result: Result<GlobalSettings, serde_json::Error> = serde_json::from_slice(config_json.as_bytes());
    let config = match config_result {
        Ok(config) => config,
        Err(_) => return Err("Cannot convert JSON to object".to_string()),
    };

    return Ok(config);
}

fn apply_global_settings(app: Weak<App>) {
    let global_settings = match read_global_settings() {
        Ok(config) => config,
        Err(_) => {
            GlobalSettings {
                upload_profile_settings: true,
                upload_community_subs: true,
                upload_community_blocks: true,
                upload_user_blocks: true,
                upload_user_saved_posts: false,
                sync_removals: false,
                confirm_uploads: true,
                write_api_profiles: false,
            }
        },
    };

    app.unwrap().set_upload_profile_settings(global_settings.upload_profile_settings);
    app.unwrap().set_upload_community_subs(global_settings.upload_community_subs);
    app.unwrap().set_upload_community_blocks(global_settings.upload_community_blocks);
    app.unwrap().set_upload_user_blocks(global_settings.upload_user_blocks);
    app.unwrap().set_upload_user_saved_posts(global_settings.upload_user_saved_posts);
    app.unwrap().set_sync_removals(global_settings.sync_removals);
    app.unwrap().set_confirm_uploads(global_settings.confirm_uploads);
    app.unwrap().set_write_api_profiles(global_settings.write_api_profiles);
}

fn write_global_settings(global_settings: GlobalSettings) {
    let home_directory = match home::home_dir() {
        Some(home_dir) => home_dir,
        None => return,
    };
    let config_path = home_directory.join(CONFIG_FILENAME);
    let mut file = match File::create(config_path) {
        Ok(file) => file,
        Err(_) => {
            return 
        }
    };

    let json_string = serde_json::to_string_pretty(&global_settings);
    file.write_all(json_string.unwrap().as_bytes()).ok();
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

    let api = match lemmy::api::Api::new(instance_url).await {
        Ok(api) => api,
        Err(e) => { 
            logger(format!("ERROR: Invalid Instance URL (or instance is down) - {e}"));
            return
        }
    };

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
    let profile_local = FromAPI::construct_profile(&profile_settings);

    // Write to File
    write_profile(&profile_local, logger);
    
}

fn read_profile() -> Result<profile::ProfileConfiguration, String> {
    return migrations::profile_migrate::read_latest_profile();
}

async fn block_users(api: &lemmy::api::Api,
    jwt_token: &str,
    message_rate_limit: std::time::Duration,
    mut logger: impl FnMut(String),
    user_list: &Vec<String>,
    block: bool) {

    let block_text = if block {
        "block"
    } else {
        "unblock"
    };

    for user in user_list {
        let user_details_result = api.fetch_user_details(jwt_token, user).await;
        thread::sleep(message_rate_limit);

        if user_details_result.is_err() {
            logger(format!("Cannot find user {} to {}, got exception {}",
                            user,
                            block_text,
                            user_details_result.unwrap_err()));
            continue;
        }

        let id = user_details_result.unwrap().person_view.person.id;
        let block_user_result = api.block_user(jwt_token, id, block).await;
        thread::sleep(message_rate_limit);

        match block_user_result {
            Ok(response) => {
                if !response.blocked {
                    format!("Server refused to {} user {}", block_text, user);
                }
            }
            Err(e) => logger(format!("Got exception {}ing user {}: {}", block_text, user, e)),
        }
    }
}

async fn block_communities(api: &lemmy::api::Api,
    jwt_token: &str,
    message_rate_limit: std::time::Duration,
    mut logger: impl FnMut(String),
    community_list: &Vec<String>,
    block: bool) {

    let block_text = if block {
        "block"
    } else {
        "unblock"
    };

    for community in community_list {
        let community_details_result = api.fetch_community_by_name(jwt_token, community).await;
        thread::sleep(message_rate_limit);

        if community_details_result.is_err() {
            logger(format!("Cannot find community {} to {}, got exception {}",
                            community,
                            block_text,
                            community_details_result.unwrap_err()));
            continue;
        }
        
        let id = community_details_result.unwrap().community_view.community.id;
        let block_community_result = api.block_community(jwt_token, id, block).await;
        thread::sleep(message_rate_limit);

        match block_community_result {
            Ok(response) => {
                if !response.blocked {
                    format!("Server refused to {} community {}", block_text, community);
                }
            }
            Err(e) => logger(format!("Got exception {}ing community {}: {}", block_text, community, e)),
        }
    }
}

async fn follow_communities(api: &lemmy::api::Api,
    jwt_token: &str,
    message_rate_limit: std::time::Duration,
    mut logger: impl FnMut(String),
    community_list: &Vec<String>,
    follow: bool) {

    let follow_text = if follow {
        "follow"
    } else {
        "unfollow"
    };

    for community in community_list {
        let community_details_result = api.fetch_community_by_name(jwt_token, community).await;
        thread::sleep(message_rate_limit);

        if community_details_result.is_err() {
            logger(format!("Cannot find community {}, got exception {}",
                           community,
                           community_details_result.unwrap_err()));
            continue;
        }

        let id = community_details_result.unwrap().community_view.community.id;
        let follow_community_result = api.follow_community(jwt_token, id, follow).await;
        thread::sleep(message_rate_limit);

        match follow_community_result {
            Ok(response) => {
                if response.community_view.subscribed == lemmy_api_common::lemmy_db_schema::SubscribedType::NotSubscribed {
                    format!("Server refused to {} community {}", follow_text, community);
                }
            }
            Err(e) => logger(format!("Got exception {}ing community {}: {}", follow_text, community, e)),
        }
    }
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

    let api = match lemmy::api::Api::new(instance_url).await {
        Ok(api) => api,
        Err(e) => { 
            logger(format!("ERROR: Invalid Instance URL (or instance is down) - {e}"));
            return
        }
    };

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
    let new_profile = FromAPI::construct_profile(&new_profile_api);

    // Calculating Differences
    let global_settings = processing_instruction.global_settings;
    let profile_changes = profile::calculate_changes(&original_profile, &new_profile);
    let mut api_calls_needed = 0u32;
    
    if global_settings.upload_profile_settings {
        logger("All profile settings from the original profile will be applied.".to_string());
        api_calls_needed += 1;
    }
    
    if global_settings.upload_user_blocks {
        logger(format!("{} new users will be blocked", profile_changes.users_to_block.len()));
        api_calls_needed += profile_changes.users_to_block.len() as u32 * 2;

        if global_settings.sync_removals {
            logger(format!("{} users will be unblocked", profile_changes.users_to_unblock.len()));
            api_calls_needed += profile_changes.users_to_unblock.len() as u32 * 2;
        }
    }
    
    if global_settings.upload_community_blocks {
        logger(format!("{} new communities will be blocked", profile_changes.communities_to_block.len()));
        api_calls_needed += profile_changes.communities_to_block.len() as u32 * 2;

        if global_settings.sync_removals {
            logger(format!("{} communities will be unblocked", profile_changes.communities_to_unblock.len()));
            api_calls_needed += profile_changes.communities_to_unblock.len() as u32 * 2;
        }
    }
    
    if global_settings.upload_community_subs {
        logger(format!("{} new communities will be followed", profile_changes.communities_to_follow.len()));
        api_calls_needed += profile_changes.communities_to_follow.len() as u32 * 2;

        if global_settings.sync_removals {
            logger(format!("{} communities will be unfollowed", profile_changes.communities_to_unfollow.len()));
            api_calls_needed += profile_changes.communities_to_unfollow.len() as u32 * 2;
        }
    }

    // Call API to actually apply changes to new account

    // Account for Rate Limits - values get mapped as seen here: lemmy/src/api_routes_http.rs
    let mut message_count_per_time_period = new_profile_api.site_view.local_site_rate_limit.message;
    if message_count_per_time_period <= 0 {
        message_count_per_time_period = 1;
    }
    let mut message_time_period_interval_sec = new_profile_api.site_view.local_site_rate_limit.message_per_second;
    if message_time_period_interval_sec <= 0 {
        message_time_period_interval_sec = 1;
    }
    let message_per_second = message_time_period_interval_sec as f64 / message_count_per_time_period as f64;
    let message_rate_limit = std::time::Duration::from_millis((message_per_second * 1000.0).ceil() as u64);
    let estimated_time_sec = (message_rate_limit.as_millis() as f64 * api_calls_needed as f64 / 1000.0) as u32;

    if estimated_time_sec > 60 {
        let minutes = estimated_time_sec / 60;
        let remaining_seconds = estimated_time_sec % 60;
        logger(format!("Estimated Upload Time: {}m {}s", minutes, remaining_seconds));
    } else {
        logger(format!("Estimated Upload Time: {}s", estimated_time_sec));
    }
    

    // Block / Unblock Users
    if global_settings.upload_user_blocks {
        block_users(&api, &jwt_token, message_rate_limit, &mut logger, &profile_changes.users_to_block, true).await;
        if global_settings.sync_removals {
            block_users(&api, &jwt_token, message_rate_limit, &mut logger, &profile_changes.users_to_unblock, false).await;
        }
    }
    
    // Block Communities
    if global_settings.upload_community_blocks {
        block_communities(&api, &jwt_token, message_rate_limit, &mut logger, &profile_changes.communities_to_block, true).await;
        if global_settings.sync_removals {
            block_communities(&api, &jwt_token, message_rate_limit, &mut logger, &profile_changes.communities_to_unblock, false).await;
        }
    }
    
    // Follow Communities
    if global_settings.upload_community_subs {
        follow_communities(&api, &jwt_token, message_rate_limit, &mut logger, &profile_changes.communities_to_follow, true).await;
        if global_settings.sync_removals {
            follow_communities(&api, &jwt_token, message_rate_limit, &mut logger, &profile_changes.communities_to_unfollow, false).await;
        }
    }
    
    // Save profile settings
    if global_settings.upload_profile_settings {
        let save_settings_result = api.save_user_settings(&jwt_token, profile_changes.profile_settings).await;
        if save_settings_result.is_err() {
            logger(format!("Cannot save profile settings, got exception {}", save_settings_result.unwrap_err()));
        }
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
    let app_control_page = app_weak.clone();
    let app_settings_page = app_weak.clone();
    let app_apply_settings = app_weak.clone();

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

    // Bind Control Page clicking action
    app.global::<ControlPageHandler>().on_clicked({
        move |window_type| {      
            let global_settings = GlobalSettings { 
                upload_profile_settings: app_control_page.unwrap().get_upload_profile_settings(),
                upload_community_subs: app_control_page.unwrap().get_upload_community_subs(),
                upload_community_blocks: app_control_page.unwrap().get_upload_community_blocks(),
                upload_user_blocks: app_control_page.unwrap().get_upload_user_blocks(),
                upload_user_saved_posts: app_control_page.unwrap().get_upload_user_saved_posts(),
                sync_removals: app_control_page.unwrap().get_sync_removals(),
                confirm_uploads: app_control_page.unwrap().get_confirm_uploads(),
                write_api_profiles: app_control_page.unwrap().get_write_api_profiles(),
            };

            if window_type == "Download" {
                app_control_page.unwrap().set_download_log_output("".into());
                app_control_page.unwrap().set_download_ui_enabled(false);

                let download_instruction = ProcessingInstruction {
                    instruction_type: window_type,
                    instance: app_control_page.unwrap().get_download_instance_url(),
                    username: app_control_page.unwrap().get_download_username_input(),
                    password: app_control_page.unwrap().get_download_password_input(),
                    two_factor_token: app_control_page.unwrap().get_download_two_factor_input(),
                    global_settings: global_settings,
                };

                instruct_tx.send(download_instruction).unwrap();
            } else {
                app_control_page.unwrap().set_upload_log_output("".into());
                app_control_page.unwrap().set_upload_ui_enabled(false);

                let upload_instruction = ProcessingInstruction {
                    instruction_type: window_type,
                    instance: app_control_page.unwrap().get_upload_instance_url(),
                    username: app_control_page.unwrap().get_upload_username_input(),
                    password: app_control_page.unwrap().get_upload_password_input(),
                    two_factor_token: app_control_page.unwrap().get_upload_two_factor_input(),
                    global_settings: global_settings,
                };

                instruct_tx.send(upload_instruction).unwrap();
            }
        }
    });

    // Bind to toggline of settings
    app.global::<SettingsPageHandler>().on_toggled({
        move || {
            let global_settings = GlobalSettings { 
                upload_profile_settings: app_settings_page.unwrap().get_upload_profile_settings(),
                upload_community_subs: app_settings_page.unwrap().get_upload_community_subs(),
                upload_community_blocks: app_settings_page.unwrap().get_upload_community_blocks(),
                upload_user_blocks: app_settings_page.unwrap().get_upload_user_blocks(),
                upload_user_saved_posts: app_settings_page.unwrap().get_upload_user_saved_posts(),
                sync_removals: app_settings_page.unwrap().get_sync_removals(),
                confirm_uploads: app_settings_page.unwrap().get_confirm_uploads(),
                write_api_profiles: app_settings_page.unwrap().get_write_api_profiles(),
            };
    
            write_global_settings(global_settings);
        }
    });

    // Load Settings
    apply_global_settings(app_apply_settings);

    // Run GUI application
    app.run().unwrap();

    // Cleanup
    instruct_tx_copy.send(ProcessingInstruction {
        instruction_type: "Done".into(),
        instance: "".into(),
        username: "".into(),
        password: "".into(),
        two_factor_token: "".into(),
        global_settings: GlobalSettings { 
            upload_profile_settings: false,
            upload_community_subs: false,
            upload_community_blocks: false,
            upload_user_blocks: false,
            upload_user_saved_posts: false,
            sync_removals: false,
            confirm_uploads: false,
            write_api_profiles: false,
        },
    }).unwrap();
    main_thread.join().unwrap();
}
