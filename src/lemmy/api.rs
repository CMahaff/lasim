use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person;
use lemmy_api_common::site;
use lemmy_api_common::community;
use lemmy_api_common::lemmy_db_schema::newtypes;
use reqwest::Client;
use reqwest::Response;
use reqwest::Error;
use crate::profile;

pub struct API {
    client: Client,
}

impl API {
    pub fn new() -> API {
        API {
            client: Client::new(),
        }
    }

    pub async fn login(&self, instance: &String, username: &String, password: &String) -> Result<String, Error> {
        let params = person::Login {
            username_or_email: Sensitive::new(username.clone()),
            password: Sensitive::new(password.clone()),
            ..Default::default() // TODO: Add totp_2fa_token for instances with 2-factor
        };
    
        let response: Response = self.client
            .post(instance.clone() + "/api/v3/user/login")
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

    pub async fn fetch_profile_settings(&self, instance: &String, jwt_token: &String) -> Result<site::GetSiteResponse, Error> {
        let params = site::GetSite {
            auth: Some(Sensitive::new(jwt_token.clone())),
        };
    
        let response: Response = self.client
            .get(instance.clone() + "/api/v3/site")
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

    pub async fn fetch_community_by_name(&self, instance: &String, jwt_token: &String, name: &String) -> 
        Result<community::GetCommunityResponse, Error> {

        let params = community::GetCommunity {
            name: Some(name.clone()),
            auth: Some(Sensitive::new(jwt_token.clone())),
            ..Default::default()
        };
    
        let response: Response = self.client
            .get(instance.clone() + "/api/v3/community")
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
        instance: &String,
        jwt_token: &String,
        community_id: newtypes::CommunityId) -> Result<community::BlockCommunityResponse, Error> {

        let params = community::BlockCommunity {
            community_id,
            block: true,
            auth: Sensitive::new(jwt_token.clone()),
        };
    
        let response: Response = self.client
            .post(instance.clone() + "/api/v3/community/block")
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
        instance: &String,
        jwt_token: &String,
        community_id: newtypes::CommunityId) -> Result<community::CommunityResponse, Error> {

        let params = community::FollowCommunity {
            community_id,
            follow: true,
            auth: Sensitive::new(jwt_token.clone()),
        };
    
        let response: Response = self.client
            .post(instance.clone() + "/api/v3/community/follow")
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

    pub async fn fetch_user_details(&self, instance: &String, jwt_token: &String, name: &String) -> 
        Result<person::GetPersonDetailsResponse, Error> {

        let params = person::GetPersonDetails {
            username: Some(name.clone()),
            auth: Some(Sensitive::new(jwt_token.clone())),
            ..Default::default()
        };
    
        let response: Response = self.client
            .get(instance.clone() + "/api/v3/user")
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
        instance: &String,
        jwt_token: &String,
        person_id: newtypes::PersonId) -> Result<person::BlockPersonResponse, Error> {

        let params = person::BlockPerson {
            person_id,
            block: true,
            auth: Sensitive::new(jwt_token.clone()),
        };
    
        let response: Response = self.client
            .post(instance.clone() + "/api/v3/user/block")
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
        instance: &String,
        jwt_token: &String,
        user_settings_local: profile::ProfileSettings) -> Result<person::LoginResponse, Error> {

        let mut user_settings_api = profile::construct_settings(&user_settings_local);
        user_settings_api.auth = Sensitive::new(jwt_token.clone());
    
        let response: Response = self.client
            .post(instance.clone() + "/api/v3/user/save_user_settings")
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