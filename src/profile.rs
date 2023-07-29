use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::site;
use lemmy_api_common::person;

use crate::lemmy::typecast;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProfileSettings {
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
pub struct ProfileConfiguration {
    pub blocked_users: Vec<String>,
    pub blocked_communities: Vec<String>,
    pub followed_communities: Vec<String>,
    pub profile_settings: ProfileSettings,
}

pub fn construct_settings(profile_settings: &ProfileSettings) -> person::SaveUserSettings {
    return person::SaveUserSettings {
        show_nsfw: Some(profile_settings.show_nsfw),
        show_scores: Some(profile_settings.show_scores),
        theme: Some(profile_settings.theme.clone()),
        default_sort_type: Some(typecast::ToAPI::cast_sort_type(&profile_settings.default_sort_type)),
        default_listing_type: Some(typecast::ToAPI::cast_listing_type(&profile_settings.default_listing_type)),
        interface_language: Some(profile_settings.interface_language.clone()),
        avatar: None, // TODO: Support Avatar migration
        banner: None, // TODO: Support Banner migration
        display_name: None, // Don't Change
        email: None, // Don't Change
        bio: None, // Don't Change
        matrix_user_id: None, // Don't Change
        show_avatars: Some(profile_settings.show_avatars),
        send_notifications_to_email: Some(profile_settings.send_notifications_to_email),
        bot_account: Some(profile_settings.bot_account),
        show_bot_accounts: Some(profile_settings.show_bot_accounts),
        show_read_posts: Some(profile_settings.show_read_posts),
        show_new_post_notifs: Some(profile_settings.show_new_post_notifs),
        discussion_languages: Some(typecast::ToAPI::cast_language_array(&profile_settings.discussion_languages)),
        generate_totp_2fa: None, // Don't change
        auth: Sensitive::from(""), // This will be inserted before the request is sent
        open_links_in_new_tab: Some(profile_settings.open_links_in_new_tab),
        infinite_scroll_enabled: Some(profile_settings.infinite_scroll_enabled),
    };
}

fn construct_blocked_users(original_profile: &site::GetSiteResponse) -> Vec<String> {
    let original_blocks = &(original_profile.my_user.as_ref().unwrap().person_blocks);
    let mut new_blocks = vec![];

    for orig_block_view in original_blocks {
        new_blocks.push(parse_url(orig_block_view.target.actor_id.to_string()));
    }

    return new_blocks;
}

fn construct_blocked_communities(original_profile: &site::GetSiteResponse) -> Vec<String> {
    let original_blocks = &(original_profile.my_user.as_ref().unwrap().community_blocks);
    let mut new_blocks = vec![];

    for orig_block_view in original_blocks {
        new_blocks.push(parse_url(orig_block_view.community.actor_id.to_string()));
    }

    return new_blocks;
}

fn construct_followed_communities(original_profile: &site::GetSiteResponse) -> Vec<String> {
    let original_follows= &(original_profile.my_user.as_ref().unwrap().follows);
    let mut new_follows = vec![];

    for orig_follow_view in original_follows {
        new_follows.push(parse_url(orig_follow_view.community.actor_id.to_string()));
    }

    return new_follows;
}

pub fn construct_profile(original_profile: &site::GetSiteResponse) -> ProfileConfiguration {
    let my_user = &(original_profile.my_user.as_ref().unwrap());
    let local_user_view = &(my_user.local_user_view);
    let local_user = &(local_user_view.local_user);
    let person = &(local_user_view.person);

    return ProfileConfiguration {
        blocked_users: construct_blocked_users(original_profile),
        blocked_communities: construct_blocked_communities(original_profile),
        followed_communities: construct_followed_communities(original_profile),
        profile_settings: ProfileSettings {
            show_nsfw: local_user.show_nsfw,
            show_scores: local_user.show_scores,
            theme: local_user.theme.clone(),
            default_sort_type: typecast::FromAPI::cast_sort_type(local_user.default_sort_type).to_string(),
            default_listing_type: typecast::FromAPI::cast_listing_type(local_user.default_listing_type).to_string(),
            interface_language: local_user.interface_language.clone(),
            show_avatars: local_user.show_avatars,
            send_notifications_to_email: local_user.send_notifications_to_email,
            bot_account: person.bot_account,
            show_bot_accounts: local_user.show_bot_accounts,
            show_read_posts: local_user.show_read_posts,
            show_new_post_notifs: local_user.show_new_post_notifs,
            discussion_languages: typecast::FromAPI::cast_language_array(&my_user.discussion_languages),
            open_links_in_new_tab: local_user.open_links_in_new_tab,
            infinite_scroll_enabled: local_user.infinite_scroll_enabled,
        },
    };
}

fn parse_url(actor_id: String) -> String {
    let removed_begin = actor_id.strip_prefix("https://").unwrap_or(&actor_id);
    let split_url: Vec<&str> = removed_begin.split("/").collect();
    return format!("{}@{}", split_url.get(2).unwrap(), split_url.get(0).unwrap());
}

fn calculate_users_to_block(original_profile: &ProfileConfiguration, new_profile: &ProfileConfiguration) -> Vec<String> {
    let original_blocks = &(original_profile.blocked_users);
    let new_blocks = &(new_profile.blocked_users);
    let mut new_block_requests: Vec<String> = vec![];

    for orig_block_user in original_blocks {
        let mut already_blocked = false;
        for new_block_user in new_blocks {
            if orig_block_user == new_block_user {
                already_blocked = true;
                break;
            }
        }

        if !already_blocked {
            new_block_requests.push(orig_block_user.clone());
        }
    }

    return new_block_requests;
}

fn calculate_communities_to_block(original_profile: &ProfileConfiguration, new_profile: &ProfileConfiguration) -> Vec<String> {
    let original_blocks = &(original_profile.blocked_communities);
    let new_blocks = &(new_profile.blocked_communities);
    let mut new_block_requests: Vec<String> = vec![];

    for orig_block_comm in original_blocks {
        let mut already_blocked = false;
        for new_block_comm in new_blocks {      
            if orig_block_comm == new_block_comm {
                already_blocked = true;
                break;
            }
        }

        if !already_blocked {
            new_block_requests.push(orig_block_comm.clone());
        }
    }

    return new_block_requests;
}

fn calculate_communities_to_follow(original_profile: &ProfileConfiguration, new_profile: &ProfileConfiguration) -> Vec<String> {
    let original_follows= &(original_profile.followed_communities);
    let new_follows = &(new_profile.followed_communities);
    let mut new_follow_requests: Vec<String> = vec![];

    for orig_follow_comm in original_follows {
        let mut already_followed = false;
        for new_follow_comm in new_follows {
            if orig_follow_comm == new_follow_comm {
                already_followed = true;
                break;
            }
        }

        if !already_followed {
            new_follow_requests.push(orig_follow_comm.clone());
        }
    }

    return new_follow_requests;
}

pub fn calculate_changes(original_profile: &ProfileConfiguration, new_profile: &ProfileConfiguration) -> ProfileConfiguration {
    return ProfileConfiguration {
        blocked_users: calculate_users_to_block(original_profile, new_profile),
        blocked_communities: calculate_communities_to_block(original_profile, new_profile),
        followed_communities: calculate_communities_to_follow(original_profile, new_profile),
        profile_settings: original_profile.profile_settings.clone(),
    };
}