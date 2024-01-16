use std::collections::BTreeMap;

use crate::error::{Error, Result};

use reqwest::Client;
use serde::Deserialize;
pub struct Discord {
    pub client: Client,
    pub token: String,
}

// Define login response after requesting login by user
#[derive(Deserialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    id: String,
    // lý do tại vì sao có này là vì thông tin trả về của URI `/users/@me`
    //  có trường dữ liệu `username` nhưng vì mình định nghĩa trường 
    // dữ liệu của struct là `user_name` nên mình cần có `serde(rename...)`
    // để 2 bên matching với nhau 
    #[serde(rename(deserialize = "username"))]
    user_name: String,
}

fn check_status(response: reqwest::Result<reqwest::Response>) -> Result<reqwest::Response> {
    let response = response?;
    if !response.status().is_success() {
        return Err(Error::Status(response.status()));
    }

    Ok(response)
}

const BASE_URL_API: &'static str = "https://discord.com/api/v10/";
impl Discord {
    pub async fn login(email: &str, password: &str) -> Result<Self> {
        let mut map = BTreeMap::new();
        map.insert("email", email);
        map.insert("password", password);

        let client = Client::new();
        let body = serde_json::to_string(&map).map_err(|e| Error::Json(e))?;
        let response = check_status(
            client
                .post(format!("{}/auth/login", BASE_URL_API))
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await,
        )?;

        let res = response.text().await?;
        // Convert response to struct
        let json: LoginResponse = serde_json::from_str(&res)?;

        Ok(Discord {
            client,
            token: json.token,
        })
    }

    // Get my infor

    pub async fn get_current_user(&self) -> Result<()> {

        // Method lấy user cần có authentication -> thêm header Authorization 
        // lấy token sau khi đã login 
        let response = check_status(
            self.client
                .get(format!("{}/users/@me", BASE_URL_API))
                .header("Content-Type", "application/json")
                .header("Authorization", self.token.to_owned())
                .header("Content-Length", 0)
                .send()
                .await,
        )?;

        let res = response.text().await?;
        // Convert response to struct
        let json: UserResponse = serde_json::from_str(&res)?;
        let me = format!("My Id:{}, My Username:{}", json.id, json.user_name);
        println!("{}", me);

        Ok(())
    }

    pub async fn logout(self) -> Result<()> {
        // Method lấy logout cần có authentication -> thêm header Authorization 
        // lấy token sau khi đã login 
        check_status(
            self.client
                .post(format!("{}/auth/logout", BASE_URL_API))
                .header("Authorization", self.token)
                .header("Content-Length", 0)
                .send()
                .await,
        )?;

        Ok(())
    }
}
