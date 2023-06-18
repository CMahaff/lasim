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

    pub async fn login(&self, instance: String, username: String, password: String ) -> Result<String, Error> {
        let params = person::Login {
            username_or_email: Sensitive::new(username),
            password: Sensitive::new(password),
            ..Default::default() // TODO: Add totp_2fa_token for instances with 2-factor
        };
    
        let response: Response = self.client
            .post(instance + "/api/v3/user/login")
            .json(&params)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let json = response.json::<person::LoginResponse>().await.unwrap();
                return Ok(json.jwt.unwrap().to_string());
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}