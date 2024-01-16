pub struct DiscordConfig {
    pub user_name: String,
    pub password: String,
}

use std::env;

pub fn get_config_from_env() -> DiscordConfig {
    let discord_username =
        env::var("USER_NAME").expect("Set the environment variable USER_NAME to use for this bot");
    let discord_password =
        env::var("PASSWORD").expect("Set the environment variable PASSWORD to use for this bot");

    DiscordConfig {
        user_name: discord_username,
        password: discord_password,
    }
}