use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person;
use lemmy_api_common::site;
use lemmy_api_common::community;
use lemmy_api_common::lemmy_db_schema::newtypes;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use reqwest::Error;
use url::Url;
use crate::profile;
use crate::lemmy::typecast::ToAPI;

pub struct Api {
    client: Client,
    instance: Url,
}

impl Api {
    pub fn new(instance: Url) -> Api {
        let mut client_builder = ClientBuilder::new();
        client_builder = client_builder.user_agent("LASIM - https://github.com/CMahaff/lasim");
        let new_client = client_builder.build().unwrap();

        return Api {
            client: new_client,
            instance,
        }
    }

    pub async fn login(&self, username: &str, password: &str, two_factor_token: Option<String>) -> Result<String, Error> {
        let url = self.instance.join("/api/v3/user/login").unwrap();
        let params = person::Login {
            username_or_email: Sensitive::new(username.to_string()),
            password: Sensitive::new(password.to_string()),
            totp_2fa_token: two_factor_token,
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

    pub async fn fetch_profile_settings(&self, jwt_token: &str) -> Result<site::GetSiteResponse, Error> {
        let url = self.instance.join("/api/v3/site").unwrap();
        let params = site::GetSite {
            auth: Some(Sensitive::new(jwt_token.to_string())),
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
                    Err(e) => return Err(e.without_url()),
                }
            },
            Err(e) => {
                return Err(e.without_url());
            }
        }
    }

    pub async fn fetch_community_by_name(&self, jwt_token: &str, name: &str) -> 
        Result<community::GetCommunityResponse, Error> {

        let url = self.instance.join("/api/v3/community").unwrap();
        let params = community::GetCommunity {
            name: Some(name.to_string()),
            auth: Some(Sensitive::new(jwt_token.to_string())),
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
                    Err(e) => return Err(e.without_url()),
                }
            },
            Err(e) => {
                return Err(e.without_url());
            }
        }
    }

    pub async fn block_community(&self,
        jwt_token: &str,
        community_id: newtypes::CommunityId,
        block: bool) -> Result<community::BlockCommunityResponse, Error> {

        let url = self.instance.join("/api/v3/community/block").unwrap();
        let params = community::BlockCommunity {
            community_id,
            block: block,
            auth: Sensitive::new(jwt_token.to_string()),
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
        jwt_token: &str,
        community_id: newtypes::CommunityId,
        follow: bool) -> Result<community::CommunityResponse, Error> {

        let url = self.instance.join("/api/v3/community/follow").unwrap();
        let params = community::FollowCommunity {
            community_id,
            follow: follow,
            auth: Sensitive::new(jwt_token.to_string()),
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

    pub async fn fetch_user_details(&self, jwt_token: &str, name: &str) -> 
        Result<person::GetPersonDetailsResponse, Error> {

        let url = self.instance.join("/api/v3/user").unwrap();
        let params = person::GetPersonDetails {
            username: Some(name.to_string()),
            auth: Some(Sensitive::new(jwt_token.to_string())),
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
                    Err(e) => return Err(e.without_url()),
                }
            },
            Err(e) => {
                return Err(e.without_url());
            }
        }
    }

    pub async fn block_user(&self,
        jwt_token: &str,
        person_id: newtypes::PersonId,
        block: bool) -> Result<person::BlockPersonResponse, Error> {

        let url = self.instance.join("/api/v3/user/block").unwrap();
        let params = person::BlockPerson {
            person_id,
            block: block,
            auth: Sensitive::new(jwt_token.to_string()),
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
        jwt_token: &str,
        user_settings_local: profile::ProfileSettings) -> Result<person::LoginResponse, Error> {

        let url = self.instance.join("/api/v3/user/save_user_settings").unwrap();
        let mut user_settings_api = ToAPI::construct_settings(&user_settings_local);
        user_settings_api.auth = Sensitive::new(jwt_token.to_string());
    
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
