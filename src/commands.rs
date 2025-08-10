use reqwest::Client;
use serenity::{
    builder::CreateApplicationCommands,
    model::{
        application::command::CommandOptionType,
        application::interaction::InteractionResponseType,
        prelude::{
            interaction::application_command::ApplicationCommandInteraction,
            interaction::message_component::MessageComponentInteraction,
        },
    },
    prelude::*,
};

use crate::api::{auction_embed, send_api, send_leaderboard, send_stats};
use crate::components::{auction_buttons, lb_buttons, txn_buttons};
use crate::team::{self, country_flag, Rank, TeamMember};

pub fn register_all_commands(
    commands: &mut CreateApplicationCommands,
) -> &mut CreateApplicationCommands {
    commands
        .create_application_command(|c| {
            c.name("lookup")
                .description("Get player info from DonutSMP")
                .create_option(|o| {
                    o.name("user")
                        .description("Username or UUID")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_application_command(|c| {
            c.name("stats")
                .description("Get detailed stats/profile from DonutSMP")
                .create_option(|o| {
                    o.name("user")
                        .description("Username or UUID")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_application_command(|c| {
            c.name("leaderboard")
                .description("Show DonutSMP leaderboards")
                .create_option(|o| {
                    o.name("type")
                        .description("Leaderboard type")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .add_string_choice("💰 Money", "money")
                        .add_string_choice("⚔️ Kills", "kills")
                        .add_string_choice("💀 Deaths", "deaths")
                        .add_string_choice("⛏️ Broken Blocks", "brokenblocks")
                        .add_string_choice("🧱 Placed Blocks", "placedblocks")
                        .add_string_choice("👹 Mobs Killed", "mobskilled")
                        .add_string_choice("⏰ Playtime", "playtime")
                        .add_string_choice("💰 Sell", "sell")
                        .add_string_choice("💎 Shards", "shards")
                        .add_string_choice("🛒 Shop", "shop")
                })
                .create_option(|o| {
                    o.name("page")
                        .description("Page number (default 1)")
                        .kind(CommandOptionType::Integer)
                        .required(false)
                })
        })
        .create_application_command(|c| {
            c.name("auction")
                .description("Show auction house entries")
                .create_option(|o| {
                    o.name("page")
                        .description("Page number (default 1)")
                        .kind(CommandOptionType::Integer)
                        .required(false)
                })
                .create_option(|o| {
                    o.name("search")
                        .description("Search for specific items (e.g. diamond, sword)")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
                .create_option(|o| {
                    o.name("sort")
                        .description("Sort order")
                        .kind(CommandOptionType::String)
                        .required(false)
                        .add_string_choice("💰 Lowest Price", "lowest_price")
                        .add_string_choice("💸 Highest Price", "highest_price")
                        .add_string_choice("🕒 Recently Listed", "recently_listed")
                        .add_string_choice("📅 Last Listed", "last_listed")
                })
        })
        .create_application_command(|c| {
            c.name("auction-transactions")
                .description("Show auction house transaction history")
                .create_option(|o| {
                    o.name("page")
                        .description("Page number (default 1)")
                        .kind(CommandOptionType::Integer)
                        .required(false)
                })
                .create_option(|o| {
                    o.name("search")
                        .description("Search for specific items")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
                .create_option(|o| {
                    o.name("sort")
                        .description("Sort order")
                        .kind(CommandOptionType::String)
                        .required(false)
                        .add_string_choice("💰 Lowest Price", "lowest_price")
                        .add_string_choice("💸 Highest Price", "highest_price")
                        .add_string_choice("🕒 Recently Listed", "recently_listed")
                        .add_string_choice("📅 Last Listed", "last_listed")
                })
        })
        .create_application_command(|c| {
            c.name("help")
                .description("Show all available commands with descriptions")
        })
        .create_application_command(|c| {
            c.name("team-name")
                .description("Set or view the team name")
                .create_option(|o| {
                    o.name("name")
                        .description("New team name (omit to view current)")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
        })
        .create_application_command(|c| {
            c.name("team-add")
                .description("Add or update a team member")
                .create_option(|o| {
                    o.name("ign")
                        .description("In-game name")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_option(|o| {
                    o.name("country")
                        .description("Country")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_option(|o| {
                    o.name("skill")
                        .description("Skill")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_option(|o| {
                    o.name("rank")
                        .description("Rank")
                        .kind(CommandOptionType::String)
                        .required(false)
                        .add_string_choice("Owner", "owner")
                        .add_string_choice("Admin", "admin")
                        .add_string_choice("Member", "member")
                })
                .create_option(|o| {
                    o.name("about")
                        .description("About")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
                .create_option(|o| {
                    o.name("discord")
                        .description("Discord tag (e.g. Name#1234)")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
        })
        .create_application_command(|c| {
            c.name("team-remove")
                .description("Remove a team member by IGN")
                .create_option(|o| {
                    o.name("ign")
                        .description("In-game name")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_application_command(|c| {
            c.name("team-list").description("List the team and members")
        })
        .create_application_command(|c| {
            c.name("online")
                .description("Check which team members are online")
        })
        .create_application_command(|c| {
            c.name("team-help")
                .description("Show team commands and usage")
        })
}

pub async fn handle_command(
    client: &Client,
    donut_key: &str,
    ctx: &Context,
    cmd: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = cmd.data.name.as_str();
    match name {
        "lookup" => {
            let user = cmd
                .data
                .options
                .get(0)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_str())
                .unwrap();
            let path = format!("/v1/lookup/{}", user.replace(" ", "%20"));
            send_api(
                cmd,
                ctx,
                client,
                donut_key,
                &path,
                &format!("🔍 Player Lookup: {}", user),
            )
            .await?;
        }
        "stats" => {
            let user = cmd
                .data
                .options
                .get(0)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_str())
                .unwrap();
            let path = format!("/v1/stats/{}", user.replace(" ", "%20"));
            send_stats(cmd, ctx, client, donut_key, &path, user).await?;
        }
        "leaderboard" => {
            let lb_type = cmd
                .data
                .options
                .get(0)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_str())
                .unwrap();
            let page = cmd
                .data
                .options
                .get(1)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as u32;
            let path = format!("/v1/leaderboards/{}/{}", lb_type, page);
            send_leaderboard(cmd, ctx, client, donut_key, &path, lb_type, page).await?;
        }
        "auction" => {
            let page = cmd
                .data
                .options
                .get(0)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as u32;
            let search = cmd
                .data
                .options
                .iter()
                .find(|opt| opt.name == "search")
                .and_then(|opt| opt.value.as_ref())
                .and_then(|v| v.as_str());
            let sort = cmd
                .data
                .options
                .iter()
                .find(|opt| opt.name == "sort")
                .and_then(|opt| opt.value.as_ref())
                .and_then(|v| v.as_str());

            let path = format!("/v1/auction/list/{}", page);

            let mut title_parts = vec![format!("🏪 Auction House (Page {})", page)];
            if let Some(search_term) = search {
                title_parts.push(format!("🔍 '{}'", search_term));
            }
            if let Some(sort_type) = sort {
                let sort_emoji = match sort_type {
                    "lowest_price" => "💰",
                    "highest_price" => "💸",
                    "recently_listed" => "🕒",
                    "last_listed" => "📅",
                    _ => "📊",
                };
                title_parts.push(format!("{} {}", sort_emoji, sort_type.replace("_", " ")));
            }
            let title = title_parts.join(" | ");

            let (embed, _) =
                auction_embed(client, donut_key, &path, &title, search, sort, page).await?;

            cmd.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.add_embed(embed)
                            .components(|c| auction_buttons(c, page, search, sort))
                    })
            }).await?;
        }
        "auction-transactions" => {
            let page = cmd
                .data
                .options
                .get(0)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as u32;
            let search = cmd
                .data
                .options
                .get(1)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_str());
            let sort = cmd
                .data
                .options
                .get(2)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_str());

            let path = format!("/v1/auction/transactions/{}", page);
            let (embed, _items) = auction_embed(
                client,
                donut_key,
                &path,
                "📜 Auction Transactions",
                search,
                sort,
                page,
            )
            .await?;

            cmd.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.add_embed(embed)
                            .components(|c| txn_buttons(c, page, search, sort))
                    })
            }).await?;
        }
        "help" => {
            cmd.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
            data.embed(|embed| {
                            embed
                                .title("🤖 DonutSMP Bot Commands")
                                .description("Here are all available commands:")
                .color(crate::constants::EMBED_COLOR_ACCENT)
                                .field("**👤 Player Commands**", 
                                    "`/lookup <user>` - Get player info\n\
                                     `/stats <user>` - Show player statistics", 
                                    false)
                                .field("**🏆 Leaderboard Commands**", 
                                    "`/leaderboard <type> [page]` - Show various leaderboards", 
                                    false)
                                .field("**🏪 Auction Commands**", 
                                    "`/auction [page] [search] [sort]` - Browse auction house\n\
                                     `/auction-transactions [page] [search] [sort]` - View transaction history", 
                                    false)
                                .field("**👥 Team Commands**", 
                                    "`/team-help` - Show team commands and usage", 
                                    false)
                                .field("**ℹ️ Other Commands**", 
                                    "`/help` - Show this help message", 
                                    false)
                                .footer(|f| f.text("💡 [brackets] for optional parameters, <brackets> for required parameters"))
                        })
                    })
            }).await?;
        }
        "team-help" => {
            cmd.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
            data.embed(|embed| {
                            embed
                                .title("👥 Team Commands")
                                .description("Manage your team stored by the bot:")
                .color(crate::constants::EMBED_COLOR_ACCENT)
                                .field(
                                    "Commands",
                                    "`/team-name [name]` - View or set the team name\n\
                                     `/team-add <ign> <country> <skill> [rank] [about] [discord]` - Add or update a member\n\
                                     `/team-remove <ign>` - Remove a member by IGN\n\
                                     `/team-list` - Show members grouped by rank\n\
                                     `/online` - Check who is online in your team\n\
                                     `/team-help` - Show this team help",
                                    false,
                                )
                                 .footer(|f| f.text("💡 [brackets] for optional parameters, <brackets> for required parameters"))
                        })
                    })
            }).await?;
        }
        "online" => {
            let team = team::load();
            if team.members.is_empty() {
                cmd.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.title("👥 Team Online Status")
                                    .description("No members yet. Use /team-add to add someone.")
                                    .color(crate::constants::EMBED_COLOR_ACCENT)
                            })
                        })
                })
                .await?;
                return Ok(());
            }

            let mut lines: Vec<String> = Vec::new();
            for m in &team.members {
                let path = format!("/v1/lookup/{}", m.ign.replace(" ", "%20"));
                let url = format!("https://api.donutsmp.net{}", path);
                let res = client
                    .get(&url)
                    .bearer_auth(donut_key)
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await;

                let prefix = match res {
                    Ok(resp) if resp.status().is_success() => "🟢",
                    _ => "🔴",
                };
                lines.push(format!("{} {}", prefix, m.ign));
            }

            let description = if lines.is_empty() {
                "None".to_string()
            } else {
                lines.join("\n")
            };

            cmd.create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            e.title(format!("👥 {} — Online Status", team.name))
                                .description(description)
                                .color(crate::constants::EMBED_COLOR_ACCENT)
                        })
                    })
            })
            .await?;
        }
        "team-name" => {
            let maybe_name = cmd
                .data
                .options
                .get(0)
                .and_then(|v| v.value.as_ref())
                .and_then(|v| v.as_str());
            if let Some(new_name) = maybe_name {
                match team::set_name(new_name) {
                    Ok(updated) => {
                        cmd.create_interaction_response(&ctx.http, |r| {
                            r.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|d| {
                                    d.embed(|e| {
                                        e.title("👥 Team Name Updated")
                                            .description(format!(
                                                "Team name set to: **{}**",
                                                updated.name
                                            ))
                                            .color(crate::constants::EMBED_COLOR_ACCENT)
                                    })
                                })
                        })
                        .await?;
                    }
                    Err(e) => {
                        cmd.create_interaction_response(&ctx.http, |r| {
                            r.interaction_response_data(|d| {
                                d.content(format!("❌ Failed to save: {}", e))
                                    .ephemeral(true)
                            })
                        })
                        .await?;
                    }
                }
            } else {
                let current = team::load();
                cmd.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.title("👥 Team Name")
                                    .description(format!("Current team name: **{}**", current.name))
                                    .color(crate::constants::EMBED_COLOR_ACCENT)
                            })
                        })
                })
                .await?;
            }
        }
        "team-add" => {
            let get = |name: &str| {
                cmd.data
                    .options
                    .iter()
                    .find(|o| o.name == name)
                    .and_then(|o| o.value.as_ref())
                    .and_then(|v| v.as_str())
            };
            let ign = get("ign").unwrap();
            let country = get("country").unwrap();
            let skill = get("skill").unwrap();
            let about = get("about").unwrap_or("");
            let rank = get("rank").map(Rank::from_str).unwrap_or_default();
            let discord_tag = get("discord").unwrap_or("");

            let member = TeamMember {
                ign: ign.into(),
                country: country.into(),
                skill: skill.into(),
                about: about.into(),
                discord_tag: discord_tag.into(),
                rank,
            };
            match team::upsert_member(member) {
                Ok((t, updated)) => {
                    let action = if updated { "updated" } else { "added" };
                    cmd.create_interaction_response(&ctx.http, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                d.embed(|e| {
                                    e.title("✅ Team Member Saved")
                                        .description({
                                            let flag = country_flag(country);
                                            let flag_space = if flag.is_empty() {
                                                "".to_string()
                                            } else {
                                                format!("{} ", flag)
                                            };
                                            let rank_text =
                                                Rank::from_str(get("rank").unwrap_or("member"))
                                                    .as_str();
                                            format!(
                                                "{}{} has been {} to **{}** as {}.",
                                                flag_space, ign, action, t.name, rank_text
                                            )
                                        })
                                        .color(crate::constants::EMBED_COLOR_ACCENT)
                                })
                            })
                    })
                    .await?;
                }
                Err(e) => {
                    cmd.create_interaction_response(&ctx.http, |r| {
                        r.interaction_response_data(|d| {
                            d.content(format!("❌ Failed to save member: {}", e))
                                .ephemeral(true)
                        })
                    })
                    .await?;
                }
            }
        }
        "team-remove" => {
            let ign = cmd
                .data
                .options
                .get(0)
                .and_then(|o| o.value.as_ref())
                .and_then(|v| v.as_str())
                .unwrap();
            match team::remove_member(ign) {
                Ok((t, removed)) => {
                    let msg = if removed {
                        format!("Removed {} from {}", ign, t.name)
                    } else {
                        format!("{} not found in {}", ign, t.name)
                    };
                    cmd.create_interaction_response(&ctx.http, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                d.embed(|e| {
                                    e.title("🗑️ Team Member Removal")
                                        .description(msg)
                                        .color(crate::constants::EMBED_COLOR_ACCENT)
                                })
                            })
                    })
                    .await?;
                }
                Err(e) => {
                    cmd.create_interaction_response(&ctx.http, |r| {
                        r.interaction_response_data(|d| {
                            d.content(format!("❌ Failed to remove member: {}", e))
                                .ephemeral(true)
                        })
                    })
                    .await?;
                }
            }
        }
        "team-list" => {
            let mut team = team::load();
            // Sort by rank (Owner > Admin > Member), then by IGN alphabetically
            team.members
                .sort_by(|a, b| match a.rank.sort_key().cmp(&b.rank.sort_key()) {
                    std::cmp::Ordering::Equal => {
                        a.ign.to_ascii_lowercase().cmp(&b.ign.to_ascii_lowercase())
                    }
                    other => other,
                });
            cmd.create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            let mut e = e
                                .title(format!("👥 {}", team.name))
                                .color(crate::constants::EMBED_COLOR_ACCENT);
                            if team.members.is_empty() {
                                e = e.description("No members yet. Use /team-add to add someone.");
                            } else {
                                let mut owners: Vec<(String, String)> = Vec::new();
                                let mut admins: Vec<(String, String)> = Vec::new();
                                let mut members: Vec<(String, String)> = Vec::new();

                                for m in &team.members {
                                    let flag = country_flag(&m.country);
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
                                    let entry = (m.ign.clone(), value);
                                    match m.rank {
                                        Rank::Owner => owners.push(entry),
                                        Rank::Admin => admins.push(entry),
                                        Rank::Member => members.push(entry),
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
                                if !owners.is_empty() && (!admins.is_empty() || !members.is_empty())
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
                            e
                        })
                    })
            })
            .await?;
        }
        _ => {
            cmd.create_interaction_response(&ctx.http, |r| {
                r.interaction_response_data(|d| d.content("❌ Unknown command").ephemeral(true))
            })
            .await?;
        }
    }
    Ok(())
}

pub async fn handle_component(
    http_client: &reqwest::Client,
    donut_api_key: &str,
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = &component.data.custom_id;

    if custom_id.starts_with("auction_") {
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() < 3 {
            return Err("Invalid component ID".into());
        }

        let (action, current_page) = (parts[1], parts[2].parse::<u32>()?);
        let new_page = match action {
            "prev" => {
                if current_page > 1 {
                    current_page - 1
                } else {
                    1
                }
            }
            "next" => current_page + 1,
            _ => return Err("Unknown action".into()),
        };

        let search_param = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].replace("%20", " "))
        } else {
            None
        };
        let sort_param = if parts.len() > 4 && !parts[4].is_empty() {
            Some(parts[4].to_string())
        } else {
            None
        };

        let mut title_parts = vec![format!("🏪 Auction House (Page {})", new_page)];
        if let Some(search_term) = &search_param {
            title_parts.push(format!("🔍 '{}'", search_term));
        }
        if let Some(sort_type) = &sort_param {
            let sort_emoji = match sort_type.as_str() {
                "lowest_price" => "💰",
                "highest_price" => "💸",
                "recently_listed" => "🕒",
                "last_listed" => "📅",
                _ => "📊",
            };
            title_parts.push(format!("{} {}", sort_emoji, sort_type.replace("_", " ")));
        }
        let title = title_parts.join(" | ");

        let path = format!("/v1/auction/list/{}", new_page);
        let (embed, _) = auction_embed(
            http_client,
            donut_api_key,
            &path,
            &title,
            search_param.as_deref(),
            sort_param.as_deref(),
            new_page,
        )
        .await?;

        component
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.add_embed(embed).components(|c| {
                            auction_buttons(
                                c,
                                new_page,
                                search_param.as_deref(),
                                sort_param.as_deref(),
                            )
                        })
                    })
            })
            .await?;
    } else if custom_id.starts_with("transaction_") {
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() < 3 {
            return Err("Invalid component ID".into());
        }

        let (action, current_page) = (parts[1], parts[2].parse::<u32>()?);
        let new_page = match action {
            "prev" => {
                if current_page > 1 {
                    current_page - 1
                } else {
                    1
                }
            }
            "next" => current_page + 1,
            _ => return Err("Unknown action".into()),
        };

        // Extract search and sort parameters from the message content if available
        let search_param = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].replace("%20", " "))
        } else {
            None
        };
        let sort_param = if parts.len() > 4 && !parts[4].is_empty() {
            Some(parts[4].to_string())
        } else {
            None
        };

        let mut title_parts = vec![format!("📜 Auction Transactions (Page {})", new_page)];
        if let Some(search_term) = &search_param {
            title_parts.push(format!("🔍 '{}'", search_term));
        }
        if let Some(sort_type) = &sort_param {
            let sort_emoji = match sort_type.as_str() {
                "lowest_price" => "💰",
                "highest_price" => "💸",
                "recently_listed" => "🕒",
                "last_listed" => "📅",
                _ => "📊",
            };
            title_parts.push(format!("{} {}", sort_emoji, sort_type.replace("_", " ")));
        }
        let title = title_parts.join(" | ");

        let path = format!("/v1/auction/transactions/{}", new_page);
        let (embed, _) = auction_embed(
            http_client,
            donut_api_key,
            &path,
            &title,
            search_param.as_deref(),
            sort_param.as_deref(),
            new_page,
        )
        .await?;

        component
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.add_embed(embed).components(|c| {
                            txn_buttons(c, new_page, search_param.as_deref(), sort_param.as_deref())
                        })
                    })
            })
            .await?;
    } else if custom_id.starts_with("leaderboard_") {
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() < 4 {
            return Err("Invalid component ID".into());
        }

        let (action, current_page, lb_type) = (parts[1], parts[2].parse::<u32>()?, parts[3]);
        let new_page = match action {
            "prev" => {
                if current_page > 1 {
                    current_page - 1
                } else {
                    1
                }
            }
            "next" => current_page + 1,
            _ => return Err("Unknown action".into()),
        };

        let path = format!("/v1/leaderboards/{}/{}", lb_type, new_page);
        let url = format!("https://api.donutsmp.net{}", path);
        let res = http_client
            .get(&url)
            .bearer_auth(donut_api_key)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err("API Error".into());
        }

        let response_text = res.text().await?;
        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        component
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.embed(|embed| {
                            use crate::formatters::format_leaderboard_response;
                            format_leaderboard_response(&json, embed, lb_type, new_page);
                            embed
                        })
                        .components(|c| lb_buttons(c, new_page, lb_type))
                    })
            })
            .await?;
    }

    Ok(())
}
