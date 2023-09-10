use crate::profile::ProfileConfiguration;
use crate::profile::ProfileSettings;
use std::path::Path;

// V2: Added "infinite_scroll_enabled"
// To convert to current (V3):
// - Add "blur_nsfw" and "auto_expand"
// - Support new "SortType" and "ListingType"
// - Support instance blocking and saved posts

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProfileSettingsV2 {
    pub show_nsfw: bool,
    pub show_scores: bool,
    pub theme: String,
    pub default_sort_type: String,
    pub default_listing_type: String,
    pub interface_language: String,
    pub show_avatars: bool,
    pub send_notifications_to_email: bool,
    pub bot_account: bool,
    pub show_bot_accounts: bool,
    pub show_read_posts: bool,
    pub show_new_post_notifs: bool,
    pub discussion_languages: Vec<i32>,
    pub open_links_in_new_tab: bool,
    pub infinite_scroll_enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProfileConfigurationV2 {
    pub blocked_users: Vec<String>,
    pub blocked_communities: Vec<String>,
    pub followed_communities: Vec<String>,
    pub profile_settings: ProfileSettingsV2,
}

const OLD_PROFILE_FILENAME: &str = "profile_v2.json";

pub fn read_profile() -> Result<ProfileConfigurationV2, String> {
    let path = Path::new(OLD_PROFILE_FILENAME);
    let profile_json_result = std::fs::read_to_string(path);
    let profile_json = match profile_json_result {
        Ok(file) => file,
        Err(_) => return Err(format!("ERROR: Failed to open {}", OLD_PROFILE_FILENAME)),
    };

    let profile_local_result: Result<ProfileConfigurationV2, serde_json::Error> = serde_json::from_slice(profile_json.as_bytes());
    let profile_local = match profile_local_result {
        Ok(profile) => profile,
        Err(e) => return Err(format!("ERROR: Failed to parse {} JSON - {}", OLD_PROFILE_FILENAME, e)),
    };

    return Ok(profile_local);
}

pub fn convert_profile(old_profile: ProfileConfigurationV2) -> ProfileConfiguration {
    let new_profile = ProfileConfiguration {
        blocked_users: old_profile.blocked_users,
        blocked_communities: old_profile.blocked_communities,
        followed_communities: old_profile.followed_communities,
        blocked_instances: vec![],
        saved_posts: vec![],
        profile_settings: ProfileSettings {
            show_nsfw: old_profile.profile_settings.show_nsfw,
            blur_nsfw: true,
            auto_expand: false,
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

    return new_profile;
}