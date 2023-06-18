mod lemmy;

use slint::Weak;
use slint::SharedString;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use futures::executor::block_on;

slint::include_modules!();

struct ProcessingInstruction {
    instruction_type: SharedString,
    instance: SharedString,
    username: SharedString,
    password: SharedString,
}

#[tokio::main]
async fn process_download(processing_instruction: ProcessingInstruction, mut logger: impl FnMut(String)) {
    let instance = processing_instruction.instance;
    let username = processing_instruction.username;
    let password = processing_instruction.password;
    logger(format!("Logging in as {}", username));

    let api = lemmy::API::new();

    let jwt_token_future = api.login(instance.to_string(), username.to_string(), password.to_string());
    let jwt_token_result = block_on(jwt_token_future);

    if jwt_token_result.is_err() {
        logger(format!("ERROR: Failed Login - {}", jwt_token_result.unwrap_err()));
        return;
    }

    let jwt_token = jwt_token_result.unwrap();
    logger("Login Successful.".to_string());
}

#[tokio::main]
async fn process_upload(processing_instruction: ProcessingInstruction, mut logger: impl FnMut(String)) {

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
