use crate::profile::ProfileSettings;
use crate::profile::ProfileConfiguration;

use lemmy_api_common::lemmy_db_schema::newtypes;
use lemmy_api_common::lemmy_db_schema;
use lemmy_api_common::person;
use lemmy_api_common::site;
use lemmy_api_common::sensitive::Sensitive;

pub struct ToAPI {}

impl ToAPI {
    pub fn construct_settings(profile_settings: &ProfileSettings) -> person::SaveUserSettings {
        return person::SaveUserSettings {
            show_nsfw: Some(profile_settings.show_nsfw),
            blur_nsfw: Some(profile_settings.blur_nsfw),
            auto_expand: Some(profile_settings.auto_expand),
            show_scores: Some(profile_settings.show_scores),
            theme: Some(profile_settings.theme.clone()),
            default_sort_type: Some(Self::cast_sort_type(&profile_settings.default_sort_type)),
            default_listing_type: Some(Self::cast_listing_type(&profile_settings.default_listing_type)),
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
            discussion_languages: Some(Self::cast_language_array(&profile_settings.discussion_languages)),
            generate_totp_2fa: None, // Don't change
            auth: Sensitive::from(""), // This will be inserted before the request is sent
            open_links_in_new_tab: Some(profile_settings.open_links_in_new_tab),
            infinite_scroll_enabled: Some(profile_settings.infinite_scroll_enabled),
        };
    }

    pub fn cast_language_array(original_languages: &Vec<i32>) -> Vec<newtypes::LanguageId> {
        let mut new_languages: Vec<newtypes::LanguageId> = vec![];
        for language in original_languages {
            new_languages.push(newtypes::LanguageId(*language));
        }
    
        return new_languages;
    }

    pub fn cast_sort_type(original_sort: &str) -> lemmy_db_schema::SortType {
        match original_sort {
            "Active" => lemmy_db_schema::SortType::Active,
            "Hot" => lemmy_db_schema::SortType::Hot,
            "New" => lemmy_db_schema::SortType::New,
            "Old" => lemmy_db_schema::SortType::Old,
            "TopDay" => lemmy_db_schema::SortType::TopDay,
            "TopWeek" => lemmy_db_schema::SortType::TopWeek,
            "TopMonth" => lemmy_db_schema::SortType::TopMonth,
            "TopYear" => lemmy_db_schema::SortType::TopYear,
            "TopAll" => lemmy_db_schema::SortType::TopAll,
            "MostComments" => lemmy_db_schema::SortType::MostComments,
            "NewComments" => lemmy_db_schema::SortType::NewComments,
            "TopHour" => lemmy_db_schema::SortType::TopHour,
            "TopSixHour" => lemmy_db_schema::SortType::TopSixHour,
            "TopTwelveHour" => lemmy_db_schema::SortType::TopTwelveHour,
            "TopThreeMonths" => lemmy_db_schema::SortType::TopThreeMonths,
            "TopSixMonths" => lemmy_db_schema::SortType::TopSixMonths,
            "TopNineMonths" => lemmy_db_schema::SortType::TopNineMonths,
            "Controversial" => lemmy_db_schema::SortType::Controversial,
            "Scaled" => lemmy_db_schema::SortType::Scaled,
            _ => lemmy_db_schema::SortType::TopDay,
        }
    }

    pub fn cast_listing_type(original_type: &str) -> lemmy_db_schema::ListingType {
        match original_type {
            "All" =>lemmy_db_schema::ListingType::All,
            "Local" => lemmy_db_schema::ListingType::Local,
            "Subscribed" => lemmy_db_schema::ListingType::Subscribed,
            "ModeratorView" => lemmy_db_schema::ListingType::ModeratorView,
            _ => lemmy_db_schema::ListingType::Subscribed,
        }
    }
}

pub struct FromAPI {}

impl FromAPI {
    fn parse_url(actor_id: String) -> String {
        // Parse Actor IDs such as:
        //     https://lemmy.world/c/fakecommunity
        //     https://kbin.social/m/fakecommunity
        //     https://chirp.social/@fakecommunity
        // Into:
        //     fakecommunity@the.url
        let removed_begin = actor_id.strip_prefix("https://").unwrap_or(&actor_id);
        let split_url: Vec<&str> = removed_begin.split('/').collect();

        let site = split_url.first().unwrap();
        let community_name = split_url.last().unwrap().trim_matches('@');
        return format!("{}@{}", community_name, site);
    }

    fn construct_blocked_users(original_profile: &site::GetSiteResponse) -> Vec<String> {
        let original_blocks = &(original_profile.my_user.as_ref().unwrap().person_blocks);
        let mut new_blocks = vec![];
    
        for orig_block_view in original_blocks {
            new_blocks.push(Self::parse_url(orig_block_view.target.actor_id.to_string()));
        }
    
        return new_blocks;
    }
    
    fn construct_blocked_communities(original_profile: &site::GetSiteResponse) -> Vec<String> {
        let original_blocks = &(original_profile.my_user.as_ref().unwrap().community_blocks);
        let mut new_blocks = vec![];
    
        for orig_block_view in original_blocks {
            new_blocks.push(Self::parse_url(orig_block_view.community.actor_id.to_string()));
        }
    
        return new_blocks;
    }
    
    fn construct_followed_communities(original_profile: &site::GetSiteResponse) -> Vec<String> {
        let original_follows= &(original_profile.my_user.as_ref().unwrap().follows);
        let mut new_follows = vec![];
    
        for orig_follow_view in original_follows {
            new_follows.push(Self::parse_url(orig_follow_view.community.actor_id.to_string()));
        }
    
        return new_follows;
    }

    pub fn construct_profile(original_profile: &site::GetSiteResponse) -> ProfileConfiguration {
        let my_user = &(original_profile.my_user.as_ref().unwrap());
        let local_user_view = &(my_user.local_user_view);
        let local_user = &(local_user_view.local_user);
        let person = &(local_user_view.person);
    
        return ProfileConfiguration {
            blocked_users: Self::construct_blocked_users(original_profile),
            blocked_communities: Self::construct_blocked_communities(original_profile),
            followed_communities: Self::construct_followed_communities(original_profile),
            blocked_instances: vec![], // TODO: Implement
            saved_posts: vec![], // TODO: Implement
            profile_settings: ProfileSettings {
                show_nsfw: local_user.show_nsfw,
                blur_nsfw: local_user.blur_nsfw,
                auto_expand: local_user.auto_expand,
                show_scores: local_user.show_scores,
                theme: local_user.theme.clone(),
                default_sort_type: Self::cast_sort_type(local_user.default_sort_type).to_string(),
                default_listing_type: Self::cast_listing_type(local_user.default_listing_type).to_string(),
                interface_language: local_user.interface_language.clone(),
                show_avatars: local_user.show_avatars,
                send_notifications_to_email: local_user.send_notifications_to_email,
                bot_account: person.bot_account,
                show_bot_accounts: local_user.show_bot_accounts,
                show_read_posts: local_user.show_read_posts,
                show_new_post_notifs: local_user.show_new_post_notifs,
                discussion_languages: Self::cast_language_array(&my_user.discussion_languages),
                open_links_in_new_tab: local_user.open_links_in_new_tab,
                infinite_scroll_enabled: local_user.infinite_scroll_enabled,
            },
        };
    }

    pub fn cast_language_array(original_languages: &Vec<newtypes::LanguageId>) -> Vec<i32> {
        let mut new_languages: Vec<i32> = vec![];
        for language in original_languages {
            new_languages.push(language.0);
        }
    
        return new_languages;
    }

    pub fn cast_sort_type(original_sort: lemmy_db_schema::SortType) -> &'static str {
        match original_sort {
            lemmy_db_schema::SortType::Active => "Active",
            lemmy_db_schema::SortType::Hot => "Hot",
            lemmy_db_schema::SortType::New => "New",
            lemmy_db_schema::SortType::Old => "Old",
            lemmy_db_schema::SortType::TopDay => "TopDay",
            lemmy_db_schema::SortType::TopWeek => "TopWeek",
            lemmy_db_schema::SortType::TopMonth => "TopMonth",
            lemmy_db_schema::SortType::TopYear => "TopYear",
            lemmy_db_schema::SortType::TopAll => "TopAll",
            lemmy_db_schema::SortType::MostComments => "MostComments",
            lemmy_db_schema::SortType::NewComments => "NewComments",
            lemmy_db_schema::SortType::TopHour => "TopHour",
            lemmy_db_schema::SortType::TopSixHour => "TopSixHour",
            lemmy_db_schema::SortType::TopTwelveHour => "TopTwelveHour",
            lemmy_db_schema::SortType::TopThreeMonths => "TopThreeMonths",
            lemmy_db_schema::SortType::TopSixMonths => "TopSixMonths",
            lemmy_db_schema::SortType::TopNineMonths => "TopNineMonths",
            lemmy_db_schema::SortType::Controversial => "Controversial",
            lemmy_db_schema::SortType::Scaled => "Scaled",
        }
    }

    pub fn cast_listing_type(original_type: lemmy_db_schema::ListingType) -> &'static str {
        match original_type {
            lemmy_db_schema::ListingType::All => "All",
            lemmy_db_schema::ListingType::Local => "Local",
            lemmy_db_schema::ListingType::Subscribed => "Subscribed",
            lemmy_db_schema::ListingType::ModeratorView => "ModeratorView",
        }
    }
}