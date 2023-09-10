#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProfileSettings {
    pub show_nsfw: bool,
    pub blur_nsfw: bool,
    pub auto_expand: bool,
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
    pub blocked_instances: Vec<String>,
    pub saved_posts: Vec<String>,
    pub profile_settings: ProfileSettings,
}

#[derive(Debug, Clone)]
pub struct ProfileChanges {
    pub users_to_block: Vec<String>,
    pub users_to_unblock: Vec<String>,
    pub communities_to_block: Vec<String>,
    pub communities_to_unblock: Vec<String>,
    pub communities_to_follow: Vec<String>,
    pub communities_to_unfollow: Vec<String>,
    pub instances_to_block: Vec<String>,
    pub instances_to_unblock: Vec<String>,
    pub posts_to_save: Vec<String>,
    pub posts_to_unsave: Vec<String>,
    pub profile_settings: ProfileSettings,
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

pub fn calculate_changes(original_profile: &ProfileConfiguration, new_profile: &ProfileConfiguration) -> ProfileChanges {
    return ProfileChanges {
        users_to_block: calculate_users_to_block(original_profile, new_profile),
        users_to_unblock: calculate_users_to_block(new_profile, original_profile),
        communities_to_block: calculate_communities_to_block(original_profile, new_profile),
        communities_to_unblock: calculate_communities_to_block(new_profile, original_profile),
        communities_to_follow: calculate_communities_to_follow(original_profile, new_profile),
        communities_to_unfollow: calculate_communities_to_follow(new_profile, original_profile),
        instances_to_block: vec![], // TODO: Implement
        instances_to_unblock: vec![], // TODO: Implement
        posts_to_save: vec![], // TODO: Implement
        posts_to_unsave: vec![], // TODO: Implement
        profile_settings: original_profile.profile_settings.clone(),
    };
}
