use dotenv;

use ethers::core::k256;
use ethers::signers::Wallet;
use ethers::types::Address;
use ethers::types::TransactionReceipt;
use ethers::types::TransactionRequest;
use ethers::types::U256;
use reqwest;
use tokio::time::sleep;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use serde_json::Value;

use serenity::async_trait;
use serenity::prelude::Context;
use serenity::prelude::*;

pub use ethers::utils;
pub use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Chain,
};
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use serenity::framework::standard::macros::{command, group, help, hook};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
    StandardFramework,
};
use serenity::model::channel::{Channel, Message};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::UserId;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use tokio::sync::Mutex;
pub type ClientSign = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        //println!("{} is connected!", ready.user.name);
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name, shard[0], shard[1],
            );
        }
    }
}

#[group]
#[commands(eth_price, eth_balance, send_faucet)]
struct General;

#[help]
#[individual_command_tip = "Hello! Use `!` as a prefix for commands\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]

async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => {
            println!("Processed command '{}'", command_name);
            sleep(tokio::time::Duration::from_secs(5)).await;
        }
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

use serenity::futures::future::BoxFuture;
use serenity::FutureExt;
fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
    _command_name: &str,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        &format!("Try this again in {} seconds.", info.as_secs()),
                    )
                    .await;
            }
        };
    }
    .boxed()
}

#[tokio::main]
async fn main() {
    let token = dotenv::var("DISCORD_TOKEN").unwrap();
    //let http = Http::new(&token);

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("!")
                .delimiters(vec![", ", " "])
                .with_whitespace(true)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .bucket("emoji", |b| b.delay(5))
        .await
        .bucket("complicated", |b| {
            b.limit(2)
                .time_span(30)
                .delay(5)
                .limit_for(LimitedFor::Channel)
                .await_ratelimits(1)
                .delay_action(delay_action)
        })
        .await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn eth_price(ctx: &Context, msg: &Message) -> CommandResult {
    let etherscan_api_key = dotenv::var("ETHERSCAN_API_KEY").unwrap();
    let client = reqwest::Client::new();
    // Lấy thông tin giá ETH 
    let response = client
        .get(format!(
            "https://api.etherscan.io/api?module=stats&action=ethprice&apikey={}",
            etherscan_api_key
        ))
        .send()
        .await
        .unwrap();
    match response.status() {
        reqwest::StatusCode::OK => {
            let body = response.text().await.unwrap();
            let json: Value = serde_json::from_str(&body).unwrap();
            let price = json["result"]["ethusd"].as_str().unwrap();
            msg.reply(&ctx.http, format!("The current price of ETH is ${}", price))
                .await?;
        }
        _ => {
            msg.reply(&ctx.http, "Something went wrong").await?;
        }
    }
    Ok(())
}

#[command]
async fn eth_balance(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(account) => {
            let settings = if let Some(guild_id) = msg.guild_id {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };

            let etherscan_api_key = dotenv::var("ETHERSCAN_API_KEY").unwrap();
            let client = reqwest::Client::new();
            // call GET method 
            let response = client
                .get(format!(
                    "https://api.etherscan.io/api?module=account\
                                               &action=balance&address={}&tag=latest&apikey={}",
                    account, etherscan_api_key
                ))
                .send()
                .await
                .unwrap();
            // Kiểm tra trạng thái trả về 
            match response.status() {
                reqwest::StatusCode::OK => {
                    let body = response.text().await.unwrap();
                    let json: Value = serde_json::from_str(&body).unwrap();
                    let balance = format!(
                        "{:.2}",
                        (json["result"].as_str().unwrap().parse::<f64>().unwrap()
                            / 1000000000000000000_f64)
                    );
                    let reply = format!("The balance of {} is {} ETH", account, balance);
                    let content = content_safe(&ctx.cache, reply, &settings, &msg.mentions);
                    msg.channel_id.say(&ctx.http, &content).await?;
                    return Ok(());
                }
                _ => {
                    msg.reply(&ctx.http, "Something went wrong").await?;
                    return Ok(());
                }
            }
        }
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.")
                .await?;
            return Ok(());
        }
    };
}

#[command]
async fn send_faucet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single::<String>() {
        Ok(account) => {
            println!("Account:{}", account);
            let settings = if let Some(guild_id) = msg.guild_id {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };
            let provider_endpoint = dotenv::var("PROVIDER_ENDPOINT").unwrap();
            let provider = Provider::<Http>::try_from(provider_endpoint)?;
            let private_key = dotenv::var("PRIVATE_FAUCET_KEY").unwrap();

            // định nghĩa local wallet từ private key 
            let wallet: LocalWallet = private_key
                .parse::<LocalWallet>()?
                .with_chain_id(Chain::Sepolia);

            // định nghĩa client 
            let client = SignerMiddleware::new(provider.clone(), wallet.clone());
            // Faucet address lấy từ private key 
            let faucet_address = wallet.clone().address();

            // Argument từ command `!send_faucet <user address>`
            let to_address = account.parse::<Address>()?;
            // Thực hiện transaction khi user yêu cầu từ bot 
            let tx = send_transaction(&client, faucet_address, to_address).await?;
            if let Some(tx) = tx {
                
                let reply = format!("Transaction hash:{}", tx.transaction_hash);
                let content = content_safe(&ctx.cache, reply, &settings, &msg.mentions);
                msg.channel_id.say(&ctx.http, &content).await?;
            }
            return Ok(());
        }
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.")
                .await?;
            return Ok(());
        }
    }
}

async fn send_transaction(
    client: &ClientSign,
    address_from: Address,
    address_to: Address,
) -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Beginning transfer of 1 native currency from {} to {}.",
        address_from, address_to
    );

    // định nghĩa 1 transacion
    let tx = TransactionRequest::new()
        .to(address_to)
        .value(U256::from(utils::parse_ether("0.0001")?))
        .from(address_from);
    // kí transaction 
    let tx = client.send_transaction(tx, None).await?.await?;
    // In ra kết quả trả về -> transaction có thực hiện thành công hay không 
    println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

    Ok(tx)
}

// Một số vấn đề cần cải tiến để tránh spam cho network
// + Một address chỉ nhận được faucet amount 1 lần trong ngày -> 24h/1 faucet
// + Nếu một ví có nhiều account thì chỉ có 1 account nhận được faucet 
