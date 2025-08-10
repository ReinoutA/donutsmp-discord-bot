use serde_json::Value;

pub fn format_number(number: i64) -> String {
    let mut result = String::new();
    let num_str = number.to_string();
    let chars: Vec<char> = num_str.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(*c);
    }

    result
}

pub fn format_playtime(value: &Value) -> String {
    if let Some(minutes) = value.as_i64() {
        let hours = minutes / 60;
        let days = hours / 24;
        if days > 0 {
            format!("{}d {}h", days, hours % 24)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes % 60)
        } else {
            format!("{}m", minutes)
        }
    } else {
        value.to_string()
    }
}

pub fn format_auction_response_with_page(
    result: &[serde_json::Value],
    embed: &mut serenity::builder::CreateEmbed,
    current_page: u32,
) {
    if result.is_empty() {
        embed.description("🏪 No auction entries found on this page.");
        return;
    }

    let start_index = (current_page - 1) * 10;
    let mut description = String::new();

    for (i, auction) in result.iter().take(10).enumerate() {
        let item_number = start_index + (i as u32) + 1;

        let item_name = auction
            .get("item")
            .and_then(|item| item.get("display_name"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                auction
                    .get("item")
                    .and_then(|item| item.get("id"))
                    .and_then(|v| v.as_str())
            })
            .map(|name| {
                if name.starts_with("minecraft:") {
                    name.replace("minecraft:", "").replace("_", " ")
                } else {
                    name.to_string()
                }
            })
            .unwrap_or_else(|| "Unknown Item".to_string());

        let item_count = auction
            .get("item")
            .and_then(|item| item.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(1);

        let price = auction.get("price").and_then(|v| v.as_i64()).unwrap_or(0);

        let seller_name = auction
            .get("seller")
            .and_then(|seller| seller.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let mut enchant_info = String::new();
        if let Some(enchants) = auction
            .get("item")
            .and_then(|item| item.get("enchants"))
            .and_then(|enchants| enchants.get("enchantments"))
            .and_then(|enchantments| enchantments.get("levels"))
            .and_then(|levels| levels.as_object())
        {
            if !enchants.is_empty() {
                let mut enchant_list = Vec::new();
                for (enchant_name, level) in enchants.iter() {
                    let level_num = level.as_i64().unwrap_or(1);
                    let display_name = enchant_name.replace("minecraft:", "").replace("_", " ");
                    if level_num > 1 {
                        enchant_list.push(format!("{} {}", display_name, level_num));
                    } else {
                        enchant_list.push(display_name);
                    }
                }
                if !enchant_list.is_empty() {
                    enchant_info = format!(" ({})", enchant_list.join(", "));
                }
            }
        }

        let count_str = if item_count > 1 {
            format!("{}x ", item_count)
        } else {
            String::new()
        };

        description.push_str(&format!(
            "**{}**. {}{}{} - **${}**\n└ *Seller: {}*\n\n",
            item_number,
            count_str,
            item_name,
            enchant_info,
            format_number(price),
            seller_name
        ));
    }

    description = description.trim_end().to_string();

    if description.len() > 4000 {
        description.truncate(3950);
        description.push_str("\n\n*... and more entries*");
    }

    embed.description(description);
}

pub fn format_stats_response(
    result: &Value,
    embed: &mut serenity::builder::CreateEmbed,
    player_name: &str,
) {
    if let Some(stats) = result.get("result") {
        embed.title(format!("📊 Player Stats: {}", player_name));
        embed.color(crate::constants::EMBED_COLOR_ACCENT);

        let mut description = String::new();

        if let Some(money) = stats.get("money").and_then(|v| v.as_str()) {
            if let Ok(money_value) = money.parse::<i64>() {
                description.push_str(&format!("💰 **Money:** ${}\n", format_number(money_value)));
            }
        }

        if let Some(shards) = stats.get("shards").and_then(|v| v.as_str()) {
            if let Ok(shards_value) = shards.parse::<i64>() {
                description.push_str(&format!("💎 **Shards:** {}\n", format_number(shards_value)));
            }
        }

        description.push('\n');

        if let Some(money_made) = stats.get("money_made_from_sell").and_then(|v| v.as_str()) {
            if let Ok(money_made_value) = money_made.parse::<i64>() {
                description.push_str(&format!(
                    "📈 **Money made:** ${}\n",
                    format_number(money_made_value)
                ));
            }
        }

        if let Some(money_spent) = stats.get("money_spent_on_shop").and_then(|v| v.as_str()) {
            if let Ok(money_spent_value) = money_spent.parse::<i64>() {
                description.push_str(&format!(
                    "🛒 **Money spent:** ${}\n",
                    format_number(money_spent_value)
                ));
            }
        }

        description.push('\n');

        if let Some(playtime) = stats.get("playtime").and_then(|v| v.as_str()) {
            if let Ok(playtime_ms) = playtime.parse::<i64>() {
                let seconds = playtime_ms / 1000;
                let minutes = seconds / 60;
                let hours = minutes / 60;
                let days = hours / 24;
                let remaining_hours = hours % 24;
                let remaining_minutes = minutes % 60;

                if days > 0 {
                    description.push_str(&format!(
                        "🕒 **Playtime:** {}d {}h {}m\n",
                        days, remaining_hours, remaining_minutes
                    ));
                } else if hours > 0 {
                    description.push_str(&format!(
                        "🕒 **Playtime:** {}h {}m\n",
                        hours, remaining_minutes
                    ));
                } else if minutes > 0 {
                    description.push_str(&format!("🕒 **Playtime:** {}m\n", minutes));
                } else {
                    description.push_str(&format!("🕒 **Playtime:** {}s\n", seconds));
                }
            }
        }

        description.push('\n');

        if let Some(kills) = stats.get("kills").and_then(|v| v.as_str()) {
            if let Ok(kills_value) = kills.parse::<i64>() {
                description.push_str(&format!("⚔️ **Kills:** {}\n", format_number(kills_value)));
            }
        }

        if let Some(deaths) = stats.get("deaths").and_then(|v| v.as_str()) {
            if let Ok(deaths_value) = deaths.parse::<i64>() {
                description.push_str(&format!("💀 **Deaths:** {}\n", format_number(deaths_value)));
            }
        }

        if let Some(mobs_killed) = stats.get("mobs_killed").and_then(|v| v.as_str()) {
            if let Ok(mobs_value) = mobs_killed.parse::<i64>() {
                description.push_str(&format!(
                    "🐗 **Mobs killed:** {}\n",
                    format_number(mobs_value)
                ));
            }
        }

        description.push('\n');

        if let Some(placed) = stats.get("placed_blocks").and_then(|v| v.as_str()) {
            if let Ok(placed_value) = placed.parse::<i64>() {
                description.push_str(&format!(
                    "🧱 **Blocks placed:** {}\n",
                    format_number(placed_value)
                ));
            }
        }

        if let Some(broken) = stats.get("broken_blocks").and_then(|v| v.as_str()) {
            if let Ok(broken_value) = broken.parse::<i64>() {
                description.push_str(&format!(
                    "⛏️ **Blocks broken:** {}\n",
                    format_number(broken_value)
                ));
            }
        }

        embed.description(description.trim());
    } else {
        embed.title(format!("❌ Stats not found for {}", player_name));
        embed.description("Could not find stats for this player.");
        embed.color(crate::constants::EMBED_COLOR_ERROR_ALT);
    }
}

pub fn format_leaderboard_response(
    result: &Value,
    embed: &mut serenity::builder::CreateEmbed,
    lb_type: &str,
    page: u32,
) {
    if let Some(entries) = result.get("result").and_then(|r| r.as_array()) {
        let emoji = match lb_type {
            "money" => "💰",
            "kills" => "⚔️",
            "deaths" => "💀",
            "brokenblocks" => "⛏️",
            "placedblocks" => "🧱",
            "mobskilled" => "👹",
            "playtime" => "⏰",
            "sell" => "💰",
            "shards" => "💎",
            "shop" => "🛒",
            _ => "🏆",
        };

        let display_name = match lb_type {
            "money" => "Money",
            "kills" => "Kills",
            "deaths" => "Deaths",
            "brokenblocks" => "Blocks Broken",
            "placedblocks" => "Blocks Placed",
            "mobskilled" => "Mobs Killed",
            "playtime" => "Playtime",
            "sell" => "Money from Selling",
            "shards" => "Shards",
            "shop" => "Money Spent",
            _ => "Unknown",
        };

        embed.title(format!(
            "{} {} Leaderboard (Page {})",
            emoji, display_name, page
        ));
        embed.color(crate::constants::EMBED_COLOR_ACCENT);

        if entries.is_empty() {
            embed.description("No entries found on this page.");
            return;
        }

        let mut description = String::new();
        let start_position = ((page - 1) * 20) + 1;

        for (i, entry) in entries.iter().take(20).enumerate() {
            let position = start_position + i as u32;
            let username = entry
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let value_str = entry.get("value").and_then(|v| v.as_str()).unwrap_or("0");

            let formatted_value = if let Ok(value) = value_str.parse::<i64>() {
                match lb_type {
                    "money" | "sell" | "shop" => format!("${}", format_number(value)),
                    "playtime" => {
                        let seconds = value / 1000;
                        let minutes = seconds / 60;
                        let hours = minutes / 60;
                        let days = hours / 24;
                        let remaining_hours = hours % 24;
                        let remaining_minutes = minutes % 60;

                        if days > 0 {
                            format!("{}d {}h {}m", days, remaining_hours, remaining_minutes)
                        } else if hours > 0 {
                            format!("{}h {}m", hours, remaining_minutes)
                        } else if minutes > 0 {
                            format!("{}m", minutes)
                        } else {
                            format!("{}s", seconds)
                        }
                    }
                    _ => format_number(value),
                }
            } else {
                value_str.to_string()
            };

            let medal = match position {
                1 => "🥇",
                2 => "🥈",
                3 => "🥉",
                _ => "  ",
            };

            description.push_str(&format!(
                "{} **#{}** {} - {}\n",
                medal, position, username, formatted_value
            ));
        }

        embed.description(description.trim());

        let displayed_count = std::cmp::min(entries.len(), 20);
        embed.footer(|f| {
            f.text(format!(
                "Page {} • Showing {} entries",
                page, displayed_count
            ))
        });
    } else {
        embed.title(format!("❌ {} Leaderboard not found", lb_type));
        embed.description("Could not load leaderboard data.");
        embed.color(crate::constants::EMBED_COLOR_ERROR);
    }
}
