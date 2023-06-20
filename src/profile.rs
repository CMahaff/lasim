use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::site;
use lemmy_api_common::person;
use lemmy_api_common::lemmy_db_schema::newtypes;

pub struct ProfileConfigurationChanges {
    pub users_to_block: Vec<newtypes::PersonId>,
    pub communities_to_block: Vec<newtypes::CommunityId>,
    pub communities_to_follow: Vec<newtypes::CommunityId>,
    pub new_settings: person::SaveUserSettings,
}

fn calculate_users_to_block(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> Vec<newtypes::PersonId> {
    return Vec::new();
}

fn calculate_communities_to_block(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> Vec<newtypes::CommunityId> {
    return Vec::new();
}

fn calculate_communities_to_follow(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> Vec<newtypes::CommunityId> {
    return Vec::new();
}

fn construct_settings(original_profile: &site::GetSiteResponse) -> person::SaveUserSettings {
    let local_user_view = &(original_profile.my_user.as_ref().unwrap().local_user_view);
    let local_user = &(local_user_view.local_user);
    let person = &(local_user_view.person);
    return person::SaveUserSettings {
        show_nsfw: Some(local_user.show_nsfw),
        show_scores: Some(local_user.show_scores),
        theme: Some(local_user.theme.clone()),
        default_sort_type: Some(local_user.default_sort_type),
        default_listing_type: Some(local_user.default_listing_type),
        interface_language: Some(local_user.interface_language.clone()),
        avatar: None, // TODO: Support Avatar migration
        banner: None, // TODO: Support Banner migration
        display_name: None, // Don't Change
        email: None, // Don't Change
        bio: None, // Don't Change
        matrix_user_id: None, // Don't Change
        show_avatars: Some(local_user.show_avatars),
        send_notifications_to_email: Some(local_user.send_notifications_to_email),
        bot_account: Some(person.bot_account),
        show_bot_accounts: Some(local_user.show_bot_accounts),
        show_read_posts: Some(local_user.show_read_posts),
        show_new_post_notifs: Some(local_user.show_new_post_notifs),
        discussion_languages: Some(original_profile.discussion_languages.clone()),
        auth: Sensitive::from(""), // This will be inserted before the request is sent
    };
}

pub fn calculate_changes(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> ProfileConfigurationChanges {
    return ProfileConfigurationChanges {
        users_to_block: calculate_users_to_block(original_profile, new_profile),
        communities_to_block: calculate_communities_to_block(original_profile, new_profile),
        communities_to_follow: calculate_communities_to_follow(original_profile, new_profile),
        new_settings: construct_settings(original_profile),
    };
}