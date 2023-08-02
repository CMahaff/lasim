use lemmy_api_common::lemmy_db_schema::newtypes;
use lemmy_api_common::lemmy_db_schema;

pub struct ToAPI {}

impl ToAPI {
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
            _ => lemmy_db_schema::SortType::TopDay,
        }
    }

    pub fn cast_listing_type(original_type: &str) -> lemmy_db_schema::ListingType {
        match original_type {
            "All" =>lemmy_db_schema::ListingType::All,
            "Local" => lemmy_db_schema::ListingType::Local,
            "Subscribed" => lemmy_db_schema::ListingType::Subscribed,
            _ => lemmy_db_schema::ListingType::Subscribed,
        }
    }
}

pub struct FromAPI {}

impl FromAPI {
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
        }
    }

    pub fn cast_listing_type(original_type: lemmy_db_schema::ListingType) -> &'static str {
        match original_type {
            lemmy_db_schema::ListingType::All => "All",
            lemmy_db_schema::ListingType::Local => "Local",
            lemmy_db_schema::ListingType::Subscribed => "Subscribed",
        }
    }
}