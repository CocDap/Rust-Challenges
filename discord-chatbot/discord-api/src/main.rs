pub mod config;
pub mod discord;
pub mod error;
use crate::config::get_config_from_env;
use crate::discord::Discord;
use crate::error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Get env file 
    let discord_config = get_config_from_env();
    // Login 
    let discord = Discord::login(&discord_config.user_name, &discord_config.password).await?;
    println!("Login succesfully");

    // Get my information 
    discord.get_current_user().await?;

    // Logout 
    discord.logout().await?;
    println!("Logout succesfully");
    Ok(())
}
