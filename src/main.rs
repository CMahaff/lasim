use slint::Weak;

slint::include_modules!();

fn process_download(app_ref: App) {
    let instance = app_ref.get_download_instance_url();
    let username = app_ref.get_download_username_input();
    let password = app_ref.get_download_password_input();

    app_ref.set_download_log_output("Values read!".into());
}

fn process_upload(app_ref: App) {
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
            if window_type == "Download" {
                process_download(app_weak.unwrap());
            } else {
                process_upload(app_weak.unwrap());
            }
        }
    });

    app.run().unwrap();
}
