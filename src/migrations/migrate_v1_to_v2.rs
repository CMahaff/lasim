use crate::profile::ProfileConfiguration;
use crate::profile::ProfileSettings;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProfileSettingsV1 {
    show_nsfw: bool,
    show_scores: bool,
    theme: String,
    default_sort_type: String,
    default_listing_type: String,
    interface_language: String,
    show_avatars: bool,
    send_notifications_to_email: bool,
    bot_account: bool,
    show_bot_accounts: bool,
    show_read_posts: bool,
    show_new_post_notifs: bool,
    discussion_languages: Vec<i32>,
    open_links_in_new_tab: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProfileConfigurationV1 {
    pub blocked_users: Vec<String>,
    pub blocked_communities: Vec<String>,
    pub followed_communities: Vec<String>,
    pub profile_settings: ProfileSettingsV1,
}

fn read_profile() -> Result<ProfileConfigurationV1, String> {
    let path = Path::new("profile_v1.json");
    let profile_json_result = std::fs::read_to_string(path);
    let profile_json = match profile_json_result {
        Ok(file) => file,
        Err(_) => return Err("ERROR: Failed to open profile settings!".to_string()),
    };

    let profile_local_result: Result<ProfileConfigurationV1, serde_json::Error> = serde_json::from_slice(profile_json.as_bytes());
    let profile_local = match profile_local_result {
        Ok(profile) => profile,
        Err(e) => return Err(format!("ERROR: Failed to parse profile JSON - {}", e)),
    };

    return Ok(profile_local);
}

pub fn convert_profile() -> Result<ProfileConfiguration, String> {
    let old_profile = match read_profile() {
        Ok(profile) => profile,
        Err(e) => return Err(e)
    };

    let new_profile = ProfileConfiguration {
        blocked_users: old_profile.blocked_users,
        blocked_communities: old_profile.blocked_communities,
        followed_communities: old_profile.followed_communities,
        profile_settings: ProfileSettings {
            show_nsfw: old_profile.profile_settings.show_nsfw,
            show_scores: old_profile.profile_settings.show_scores,
            theme: old_profile.profile_settings.theme,
            default_sort_type: old_profile.profile_settings.default_sort_type,
            default_listing_type: old_profile.profile_settings.default_listing_type,
            interface_language: old_profile.profile_settings.interface_language,
            show_avatars: old_profile.profile_settings.show_avatars,
            send_notifications_to_email: old_profile.profile_settings.send_notifications_to_email,
            bot_account: old_profile.profile_settings.bot_account,
            show_bot_accounts: old_profile.profile_settings.show_bot_accounts,
            show_read_posts: old_profile.profile_settings.show_read_posts,
            show_new_post_notifs: old_profile.profile_settings.show_new_post_notifs,
            discussion_languages: old_profile.profile_settings.discussion_languages,
            open_links_in_new_tab: old_profile.profile_settings.open_links_in_new_tab,
            infinite_scroll_enabled: false,
        },
    };

    return Ok(new_profile);
}