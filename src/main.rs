mod lemmy;

use slint::Weak;
use futures::executor::block_on;

slint::include_modules!();

#[tokio::main]
async fn process_download(app_ref: App) {
    let instance = app_ref.get_download_instance_url();
    let username = app_ref.get_download_username_input();
    let password = app_ref.get_download_password_input();

    let api = lemmy::API::new();

    let jwt_token_future = api.login(instance.to_string(), username.to_string(), password.to_string());
    let jwt_token_result = block_on(jwt_token_future);

    if jwt_token_result.is_err() {
        let error_string = format!("ERROR: Failed Login - {}", jwt_token_result.unwrap_err());
        app_ref.set_download_log_output(error_string.into());
        return;
    }

    let jwt_token = jwt_token_result.unwrap();
    app_ref.set_download_log_output("Login Success!".into());
}

#[tokio::main]
async fn process_upload(app_ref: App) {
    let instance = app_ref.get_upload_instance_url();
    let username = app_ref.get_upload_username_input();
    let password = app_ref.get_upload_password_input();
    
    app_ref.set_download_log_output("Values read!".into());
}

fn main() {
    let app = App::new().unwrap();

    app.global::<ControlPageHandler>().on_clicked({
        let app_weak: Weak<App> = app.as_weak();

        move |window_type| {
            // TODO: We need some kind of threading so this doesn't lock the UI while this happens
            if window_type == "Download" {
                process_download(app_weak.unwrap());
            } else {
                process_upload(app_weak.unwrap());
            }
        }
    });

    app.run().unwrap();
}
