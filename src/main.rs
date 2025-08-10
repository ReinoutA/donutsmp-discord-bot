mod api;
mod commands;
mod components;
mod constants;
mod formatters;
mod response_formatters;
mod team;

use dotenv::dotenv;
use reqwest::Client;
use serenity::{
    async_trait,
    client::ClientBuilder,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        prelude::{ChannelId, MessageId},
    },
    prelude::*,
};
use std::env;
use tracing::{error, info};

use chrono::Utc;
use commands::{handle_command, handle_component, register_all_commands};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct Handler {
    http_client: Client,
    donut_api_key: String,
    online_channel_id: Option<u64>,
    online_interval_minutes: u64,
    last_online_message_id: Arc<Mutex<Option<MessageId>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        if let Ok(guild_id_str) = env::var("GUILD_ID") {
            let guild_id: u64 = guild_id_str.parse().expect("GUILD_ID must be a number");
            let guild_id = serenity::model::id::GuildId(guild_id);
            let _commands = guild_id
                .set_application_commands(&ctx.http, |commands| register_all_commands(commands))
                .await;
            info!("Registered commands for guild {}", guild_id.0);
        } else {
            info!("For global commands, please register them manually or set GUILD_ID");
        }

        // Spawn background task to periodically post online status
        if let Some(channel_id) = self.online_channel_id {
            let ctx_clone = ctx.clone();
            let http_client = self.http_client.clone();
            let api_key = self.donut_api_key.clone();
            let last_msg = Arc::clone(&self.last_online_message_id);
            let interval = self.online_interval_minutes;
            tokio::spawn(async move {
                loop {
                    let mut team = crate::team::load();
                    let mut online_set: HashSet<String> = HashSet::new();
                    let mut location_map: HashMap<String, String> = HashMap::new();
                    for m in &team.members {
                        let path = format!("/v1/lookup/{}", m.ign.replace(" ", "%20"));
                        let url = format!("https://api.donutsmp.net{}", path);
                        let res = http_client
                            .get(&url)
                            .bearer_auth(&api_key)
                            .timeout(Duration::from_secs(10))
                            .send()
                            .await;
                        if let Ok(resp) = res {
                            if resp.status().is_success() {
                                let text = resp.text().await.unwrap_or_default();
                                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                    let loc = json
                                        .get("result")
                                        .and_then(|r| r.get("location"))
                                        .and_then(|v| v.as_str())
                                        .or_else(|| json.get("location").and_then(|v| v.as_str()));
                                    if let Some(location) = loc {
                                        location_map.insert(m.ign.clone(), location.to_string());
                                    }
                                }
                                online_set.insert(m.ign.clone());
                            }
                        }
                    }

                    team.members
                        .sort_by(|a, b| match a.rank.sort_key().cmp(&b.rank.sort_key()) {
                            std::cmp::Ordering::Equal => {
                                a.ign.to_ascii_lowercase().cmp(&b.ign.to_ascii_lowercase())
                            }
                            other => other,
                        });
                    let chan = ChannelId(channel_id);

                    let prev_id_opt = {
                        let guard = last_msg.lock().await;
                        (*guard).clone()
                    };
                    if let Some(prev_id) = prev_id_opt {
                        let _ = chan.delete_message(&ctx_clone.http, prev_id).await;
                    }

                    match chan
                        .send_message(&ctx_clone.http, |m| {
                            m.embed(|e| {
                                let mut e = e
                                    .title(format!("👥 {}", team.name))
                                    .color(crate::constants::EMBED_COLOR_ACCENT);
                                if team.members.is_empty() {
                                    e = e.description(
                                        "No members yet. Use /team-add to add someone.",
                                    );
                                } else {
                                    let mut owners: Vec<(String, String)> = Vec::new();
                                    let mut admins: Vec<(String, String)> = Vec::new();
                                    let mut members: Vec<(String, String)> = Vec::new();

                                    for m in &team.members {
                                        let flag = crate::team::country_flag(&m.country);
                                        let country_display = if flag.is_empty() {
                                            m.country.clone()
                                        } else {
                                            format!("{} ({})", m.country, flag)
                                        };
                                        let discord = if m.discord_tag.is_empty() {
                                            "-".to_string()
                                        } else {
                                            m.discord_tag.clone()
                                        };
                                        let mut value = format!(
                                            "Country: {}\nSkill: {}\nDiscord: {}",
                                            country_display, m.skill, discord
                                        );
                                        if !m.about.is_empty() {
                                            value.push_str(&format!("\nAbout: {}", m.about));
                                        }
                                        let online = online_set.contains(&m.ign);
                                        let name = if online {
                                            if let Some(loc) = location_map.get(&m.ign) {
                                                format!("{}    [ 🟢 - {}]", m.ign, loc)
                                            } else {
                                                format!("{}    [ 🟢 ]", m.ign)
                                            }
                                        } else {
                                            format!("{}    [ 🔴 ]", m.ign)
                                        };
                                        let entry = (name, value);
                                        match m.rank {
                                            crate::team::Rank::Owner => owners.push(entry),
                                            crate::team::Rank::Admin => admins.push(entry),
                                            crate::team::Rank::Member => members.push(entry),
                                        }
                                    }

                                    if !owners.is_empty() {
                                        let count = owners.len();
                                        e = e.field(
                                            format!("👑 Owner ({})", count),
                                            crate::constants::ZWSP,
                                            false,
                                        );
                                        for (name, val) in owners.iter() {
                                            e = e.field(name.clone(), val.clone(), false);
                                        }
                                    }
                                    if !owners.is_empty()
                                        && (!admins.is_empty() || !members.is_empty())
                                    {
                                        e = e.field(
                                            crate::constants::ZWSP,
                                            crate::constants::ZWSP,
                                            false,
                                        );
                                    }
                                    if !admins.is_empty() {
                                        let count = admins.len();
                                        e = e.field(
                                            format!("🛡️ Admin ({})", count),
                                            crate::constants::ZWSP,
                                            false,
                                        );
                                        for (name, val) in admins.iter() {
                                            e = e.field(name.clone(), val.clone(), false);
                                        }
                                    }
                                    if !admins.is_empty() && !members.is_empty() {
                                        e = e.field(
                                            crate::constants::ZWSP,
                                            crate::constants::ZWSP,
                                            false,
                                        );
                                    }
                                    if !members.is_empty() {
                                        let count = members.len();
                                        e = e.field(
                                            format!("👤 Member ({})", count),
                                            crate::constants::ZWSP,
                                            false,
                                        );
                                        for (name, val) in members.iter() {
                                            e = e.field(name.clone(), val.clone(), false);
                                        }
                                    }
                                }
                                e = e.footer(|f| {
                                    let ts = Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
                                    f.text(format!("Last updated: {} (UTC)", ts))
                                });
                                e
                            })
                        })
                        .await
                    {
                        Ok(msg) => {
                            let mut guard = last_msg.lock().await;
                            *guard = Some(msg.id);
                        }
                        Err(e) => {
                            error!("Failed to post online status: {:?}", e);
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(interval.saturating_mul(60))).await;
                }
            });
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => {
                if let Err(e) =
                    handle_command(&self.http_client, &self.donut_api_key, &ctx, &cmd).await
                {
                    error!("Command handling error: {:?}", e);
                    if let Err(e2) = cmd
                        .create_interaction_response(&ctx.http, |r| {
                            r.interaction_response_data(|d| {
                                d.content(format!("Error: {}", e)).ephemeral(true)
                            })
                        })
                        .await
                    {
                        error!("Failed to send error response: {:?}", e2);
                    }
                }
            }
            Interaction::MessageComponent(component) => {
                if let Err(e) =
                    handle_component(&self.http_client, &self.donut_api_key, &ctx, &component).await
                {
                    error!("Component handling error: {:?}", e);
                    if let Err(e2) = component
                        .create_interaction_response(&ctx.http, |r| {
                            r.interaction_response_data(|d| {
                                d.content(format!("Error: {}", e)).ephemeral(true)
                            })
                        })
                        .await
                    {
                        error!("Failed to send error response: {:?}", e2);
                    }
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let donut_api_key = env::var("DONUTSMP_API_KEY").expect("DONUTSMP_API_KEY must be set");
    let online_channel_id = env::var("ONLINE_CHANNEL_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok());
    let online_interval_minutes = env::var("ONLINE_INTERVAL_MINUTES")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10);

    let http_client = Client::builder()
        .user_agent("donutsmp-rs-bot/0.1")
        .build()
        .expect("failed to build http client");

    let handler = Handler {
        http_client,
        donut_api_key: donut_api_key.clone(),
        online_channel_id,
        online_interval_minutes,
        last_online_message_id: Arc::new(Mutex::new(None)),
    };

    let intents = GatewayIntents::GUILDS;

    let mut client = ClientBuilder::new(&token, intents)
        .event_handler(handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
