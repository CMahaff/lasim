use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person;
use lemmy_api_common::site;
use lemmy_api_common::community;
use lemmy_api_common::lemmy_db_schema::newtypes;
use reqwest::Client;
use reqwest::Response;
use reqwest::Error;
use url::Url;
use crate::profile;

pub struct API {
    client: Client,
    instance: Url,
}

impl API {
    pub fn new(instance: Url) -> API {
        API {
            client: Client::new(),
            instance,
        }
    }

    pub async fn login(&self, username: &String, password: &String) -> Result<String, Error> {
        let url = self.instance.join("/api/v3/user/login").unwrap();
        let params = person::Login {
            username_or_email: Sensitive::new(username.clone()),
            password: Sensitive::new(password.clone()),
            ..Default::default() // TODO: Add totp_2fa_token for instances with 2-factor
        };
    
        let response: Response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<person::LoginResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json.jwt.unwrap().to_string()),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn fetch_profile_settings(&self, jwt_token: &String) -> Result<site::GetSiteResponse, Error> {
        let url = self.instance.join("/api/v3/site").unwrap();
        let params = site::GetSite {
            auth: Some(Sensitive::new(jwt_token.clone())),
        };
    
        let response: Response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<site::GetSiteResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn fetch_community_by_name(&self, jwt_token: &String, name: &String) -> 
        Result<community::GetCommunityResponse, Error> {

        let url = self.instance.join("/api/v3/community").unwrap();
        let params = community::GetCommunity {
            name: Some(name.clone()),
            auth: Some(Sensitive::new(jwt_token.clone())),
            ..Default::default()
        };
    
        let response: Response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<community::GetCommunityResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn block_community(&self,
        jwt_token: &String,
        community_id: newtypes::CommunityId) -> Result<community::BlockCommunityResponse, Error> {

        let url = self.instance.join("/api/v3/community/block").unwrap();
        let params = community::BlockCommunity {
            community_id,
            block: true,
            auth: Sensitive::new(jwt_token.clone()),
        };
    
        let response: Response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<community::BlockCommunityResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn follow_community(&self,
        jwt_token: &String,
        community_id: newtypes::CommunityId) -> Result<community::CommunityResponse, Error> {

        let url = self.instance.join("/api/v3/community/follow").unwrap();
        let params = community::FollowCommunity {
            community_id,
            follow: true,
            auth: Sensitive::new(jwt_token.clone()),
        };
    
        let response: Response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<community::CommunityResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn fetch_user_details(&self, jwt_token: &String, name: &String) -> 
        Result<person::GetPersonDetailsResponse, Error> {

        let url = self.instance.join("/api/v3/user").unwrap();
        let params = person::GetPersonDetails {
            username: Some(name.clone()),
            auth: Some(Sensitive::new(jwt_token.clone())),
            ..Default::default()
        };
    
        let response: Response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<person::GetPersonDetailsResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn block_user(&self,
        jwt_token: &String,
        person_id: newtypes::PersonId) -> Result<person::BlockPersonResponse, Error> {

        let url = self.instance.join("/api/v3/user/block").unwrap();
        let params = person::BlockPerson {
            person_id,
            block: true,
            auth: Sensitive::new(jwt_token.clone()),
        };
    
        let response: Response = self.client
            .post(url)
            .json(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<person::BlockPersonResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn save_user_settings(&self,
        jwt_token: &String,
        user_settings_local: profile::ProfileSettings) -> Result<person::LoginResponse, Error> {

        let url = self.instance.join("/api/v3/user/save_user_settings").unwrap();
        let mut user_settings_api = profile::construct_settings(&user_settings_local);
        user_settings_api.auth = Sensitive::new(jwt_token.clone());
    
        let response: Response = self.client
            .put(url)
            .json(&user_settings_api)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<person::LoginResponse>().await;
                match json_result {
                    Ok(json) => return Ok(json),
                    Err(e) => return Err(e),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}