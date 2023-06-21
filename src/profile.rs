use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::site;
use lemmy_api_common::person;
pub struct ProfileConfigurationChanges {
    pub users_to_block: Vec<String>,
    pub communities_to_block: Vec<String>,
    pub communities_to_follow: Vec<String>,
    pub new_settings: person::SaveUserSettings,
}

fn parse_url(actor_id: String) -> String {
    let removed_begin = actor_id.strip_prefix("https://").unwrap_or(&actor_id);
    let split_url: Vec<&str> = removed_begin.split("/").collect();
    return format!("{}@{}", split_url.get(2).unwrap(), split_url.get(0).unwrap());
}

fn calculate_users_to_block(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> Vec<String> {
    let original_blocks = &(original_profile.my_user.as_ref().unwrap().person_blocks);
    let new_blocks = &(new_profile.my_user.as_ref().unwrap().person_blocks);
    let mut new_block_requests: Vec<String> = vec![];

    for orig_block_view in original_blocks {
        let orig_block_user = parse_url(orig_block_view.target.actor_id.to_string());

        let mut already_blocked = false;
        for new_block_view in new_blocks {
            let new_block_user = parse_url(new_block_view.target.actor_id.to_string());
            
            if orig_block_user == new_block_user {
                already_blocked = true;
                break;
            }
        }

        if !already_blocked {
            new_block_requests.push(orig_block_user);
        }
    }

    return new_block_requests;
}

fn calculate_communities_to_block(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> Vec<String> {
    let original_blocks = &(original_profile.my_user.as_ref().unwrap().community_blocks);
    let new_blocks = &(new_profile.my_user.as_ref().unwrap().community_blocks);
    let mut new_block_requests: Vec<String> = vec![];

    for orig_block_view in original_blocks {
        let orig_block_comm = parse_url(orig_block_view.community.actor_id.to_string());

        let mut already_blocked = false;
        for new_block_view in new_blocks {
            let new_block_comm = parse_url(new_block_view.community.actor_id.to_string());
            
            if orig_block_comm == new_block_comm {
                already_blocked = true;
                break;
            }
        }

        if !already_blocked {
            new_block_requests.push(orig_block_comm);
        }
    }

    return new_block_requests;
}

fn calculate_communities_to_follow(original_profile: &site::GetSiteResponse, new_profile: &site::GetSiteResponse) -> Vec<String> {
    let original_follows= &(original_profile.my_user.as_ref().unwrap().follows);
    let new_follows = &(new_profile.my_user.as_ref().unwrap().follows);
    let mut new_follow_requests: Vec<String> = vec![];

    for orig_follow_view in original_follows {
        let orig_follow_comm = parse_url(orig_follow_view.community.actor_id.to_string());

        let mut already_followed = false;
        for new_follow_view in new_follows {
            let new_follow_comm = parse_url(new_follow_view.community.actor_id.to_string());
            
            if orig_follow_comm == new_follow_comm {
                already_followed = true;
                break;
            }
        }

        if !already_followed {
            new_follow_requests.push(orig_follow_comm);
        }
    }

    return new_follow_requests;
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