use lemmy_api_common::sensitive::Sensitive;
use lemmy_api_common::person;
use lemmy_api_common::site;
use reqwest::Client;
use reqwest::Response;
use reqwest::Error;


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
}