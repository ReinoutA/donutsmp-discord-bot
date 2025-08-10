use crate::formatters::{format_number, format_playtime};
use serde_json::Value;

pub fn format_api_response(
    json: &Value,
    path: &str,
    embed: &mut serenity::builder::CreateEmbed,
) -> bool {
    match path {
        p if p.contains("/lookup/") => format_lookup_response(json, embed),
        p if p.contains("/stats/") => format_stats_response(json, embed),
        p if p.contains("/leaderboards/") => format_leaderboard_response(json, embed),
        p if p.contains("/auction/list/") => format_auction_response(json, embed),
        p if p.contains("/auction/transactions/") => {
            format_auction_transactions_response(json, embed)
        }
        p if p.contains("/online") => format_online_response(json, embed),
        p if p.contains("/server") => format_server_response(json, embed),
        _ => false,
    }
}

pub fn format_lookup_response(json: &Value, embed: &mut serenity::builder::CreateEmbed) -> bool {
    if let Some(result) = json.get("result") {
        if let Some(username) = result.get("username").and_then(|v| v.as_str()) {
            embed.field("Username", username, true);
        }
        if let Some(location) = result.get("location").and_then(|v| v.as_str()) {
            embed.field("Location", location, true);
        }
        if let Some(rank) = result.get("rank").and_then(|v| v.as_str()) {
            embed.field("Rank", rank, true);
        }

        if result.as_object().map_or(false, |obj| !obj.is_empty()) {
            return true;
        }
    }

    if let Some(status) = json.get("status") {
        embed.field("📊 Status", status.to_string(), true);
    }
    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }

    true
}

pub fn format_stats_response(json: &Value, embed: &mut serenity::builder::CreateEmbed) -> bool {
    let data = json.get("result").unwrap_or(json);

    if let Some(username) = data.get("username").and_then(|v| v.as_str()) {
        embed.field("👤 Player", username, true);
    }
    if let Some(money) = data.get("money") {
        let money_val = money.as_i64().unwrap_or(0);
        embed.field("💰 Money", format!("${}", format_number(money_val)), true);
    }
    if let Some(kills) = data.get("kills") {
        embed.field("⚔️ Kills", kills.to_string(), true);
    }
    if let Some(deaths) = data.get("deaths") {
        embed.field("💀 Deaths", deaths.to_string(), true);
    }
    if let Some(playtime) = data.get("playtime") {
        embed.field("⏰ Playtime", format_playtime(playtime), true);
    }
    if let Some(rank) = data.get("rank").and_then(|v| v.as_str()) {
        embed.field("🎭 Rank", rank, true);
    }
    if let Some(location) = data.get("location").and_then(|v| v.as_str()) {
        embed.field("📍 Location", location, true);
    }
    if let Some(status) = json.get("status") {
        if status != &serde_json::Value::Number(serde_json::Number::from(0)) {
            embed.field("📊 Status", status.to_string(), true);
        }
    }
    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }

    true
}

pub fn format_leaderboard_response(
    json: &Value,
    embed: &mut serenity::builder::CreateEmbed,
) -> bool {
    let data = json.get("result").unwrap_or(json);

    if let Some(leaderboard) = data.get("leaderboard").and_then(|v| v.as_array()) {
        let mut description = String::new();
        for (i, entry) in leaderboard.iter().take(10).enumerate() {
            if let (Some(name), Some(value)) = (
                entry.get("username").and_then(|v| v.as_str()),
                entry.get("value"),
            ) {
                let rank_emoji = match i + 1 {
                    1 => "🥇",
                    2 => "🥈",
                    3 => "🥉",
                    _ => "▫️",
                };
                description.push_str(&format!(
                    "{} **{}**. {} - {}\n",
                    rank_emoji,
                    i + 1,
                    name,
                    value
                ));
            }
        }
        if !description.is_empty() {
            embed.description(description);
            return true;
        }
    }

    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }
    if let Some(status) = json.get("status") {
        embed.field("📊 Status", status.to_string(), true);
    }

    true
}

pub fn format_auction_response(json: &Value, embed: &mut serenity::builder::CreateEmbed) -> bool {
    if let Some(result) = json.get("result").and_then(|v| v.as_array()) {
        if result.is_empty() {
            embed.description("🏪 No auction entries found on this page.");
            return true;
        }

        crate::formatters::format_auction_response_with_page(result, embed, 1);

        return true;
    }

    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }
    if let Some(status) = json.get("status") {
        embed.field("📊 Status", status.to_string(), true);
    }

    true
}

pub fn format_auction_transactions_response(
    json: &Value,
    embed: &mut serenity::builder::CreateEmbed,
) -> bool {
    if let Some(result) = json.get("result").and_then(|v| v.as_array()) {
        if result.is_empty() {
            embed.description("📜 No auction transactions found on this page.");
            return true;
        }

        let mut description = String::new();
        for (i, transaction) in result.iter().take(10).enumerate() {
            let item_name = transaction
                .get("item")
                .and_then(|item| item.get("display_name"))
                .and_then(|v| v.as_str())
                .or_else(|| {
                    transaction
                        .get("item")
                        .and_then(|item| item.get("id"))
                        .and_then(|v| v.as_str())
                })
                .or_else(|| transaction.get("item_name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown Item");

            let price = transaction
                .get("price")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let buyer = transaction
                .get("buyer")
                .and_then(|buyer| buyer.get("name"))
                .and_then(|v| v.as_str())
                .or_else(|| transaction.get("buyer_name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown");

            let seller = transaction
                .get("seller")
                .and_then(|seller| seller.get("name"))
                .and_then(|v| v.as_str())
                .or_else(|| transaction.get("seller_name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown");

            let timestamp = transaction
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .or_else(|| transaction.get("time").and_then(|v| v.as_i64()));

            let time_str = if let Some(ts) = timestamp {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let hours_ago = (current_time - ts) / 3600;
                if hours_ago > 24 {
                    format!("{}d ago", hours_ago / 24)
                } else if hours_ago > 0 {
                    format!("{}h ago", hours_ago)
                } else {
                    "Recently".to_string()
                }
            } else {
                "Unknown time".to_string()
            };

            description.push_str(&format!(
                "**{}**. **{}** - **${}**\n└ *{} → {} | {}*\n\n",
                i + 1,
                item_name,
                format_number(price),
                seller,
                buyer,
                time_str
            ));
        }

        description = description.trim_end().to_string();

        if description.len() > 4000 {
            description.truncate(3950);
            description.push_str("\n\n*... and more transactions*");
        }

        embed.description(description);
        return true;
    }

    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }
    if let Some(status) = json.get("status") {
        embed.field("📊 Status", status.to_string(), true);
    }

    true
}

pub fn format_online_response(json: &Value, embed: &mut serenity::builder::CreateEmbed) -> bool {
    let data = json.get("result").unwrap_or(json);

    if let Some(count) = data.get("online") {
        embed.field("👥 Players Online", count.to_string(), true);
    }
    if let Some(max) = data.get("max") {
        embed.field("🏠 Max Players", max.to_string(), true);
    }
    if let Some(players) = data.get("players").and_then(|v| v.as_array()) {
        let player_list = players
            .iter()
            .filter_map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if !player_list.is_empty() {
            embed.field("📋 Player List", player_list, false);
        }
    }

    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }
    if let Some(status) = json.get("status") {
        embed.field("📊 Status", status.to_string(), true);
    }

    true
}

pub fn format_server_response(json: &Value, embed: &mut serenity::builder::CreateEmbed) -> bool {
    let data = json.get("result").unwrap_or(json);

    if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
        embed.field("🌐 Server", name, true);
    }
    if let Some(version) = data.get("version").and_then(|v| v.as_str()) {
        embed.field("📦 Version", version, true);
    }
    if let Some(motd) = data.get("motd").and_then(|v| v.as_str()) {
        embed.field("📜 MOTD", motd, false);
    }
    if let Some(players_online) = data.get("online") {
        embed.field("👥 Online", players_online.to_string(), true);
    }
    if let Some(max_players) = data.get("max") {
        embed.field("🏠 Max Players", max_players.to_string(), true);
    }

    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        embed.field("💬 Message", message, false);
    }
    if let Some(status) = json.get("status") {
        embed.field("📊 Status", status.to_string(), true);
    }

    true
}
