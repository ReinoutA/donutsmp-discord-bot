use reqwest::Client;
use serde_json::Value;
use serenity::{
    model::prelude::interaction::application_command::ApplicationCommandInteraction, prelude::*,
};
use std::time::Duration;

use crate::formatters::{
    format_auction_response_with_page, format_leaderboard_response, format_stats_response,
};
use crate::response_formatters::format_api_response;

pub async fn send_api(
    cmd: &ApplicationCommandInteraction,
    ctx: &Context,
    client: &Client,
    donut_key: &str,
    path: &str,
    friendly_title: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Immediately ack with a deferred response (allows more than 3s processing)
    cmd.create_interaction_response(&ctx.http, |resp| resp.kind(serenity::model::prelude::interaction::InteractionResponseType::DeferredChannelMessageWithSource)).await?;

    let url = format!("https://api.donutsmp.net{}", path);
    let res = client
        .get(&url)
        .bearer_auth(donut_key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        // Special case: the lookup endpoint returns 500 when the player is offline; treat it as info, not an error
        if status.as_u16() == 500 && path.starts_with("/v1/lookup/") {
            let mut embed = serenity::builder::CreateEmbed::default();
            embed
                .title(friendly_title)
                .description("🔴 Player is offline")
                .color(crate::constants::EMBED_COLOR_ACCENT);

            cmd.create_followup_message(&ctx.http, |m| m.add_embed(embed))
                .await?;
            return Ok(());
        }

        let error_msg = match status.as_u16() {
            401 => "❌ **Authentication Error**: Invalid API key".to_string(),
            403 => "❌ **Access Forbidden**: You don't have permission to access this endpoint"
                .to_string(),
            404 => "❌ **Not Found**: The requested resource doesn't exist".to_string(),
            429 => "❌ **Rate Limited**: Too many requests, please try again later".to_string(),
            500..=599 => "❌ **Server Error**: The server encountered an error".to_string(),
            _ => format!("❌ **Error**: API returned HTTP {}", status),
        };

        let mut embed = serenity::builder::CreateEmbed::default();
        embed
            .title("DonutSMP API Error")
            .description(error_msg)
            .color(crate::constants::EMBED_COLOR_ERROR)
            .footer(|f| f.text(format!("Status: {} | Path: {}", status, path)));

        cmd.create_followup_message(&ctx.http, |m| m.add_embed(embed))
            .await?;
        return Ok(());
    }

    let json: Value = res.json().await?;

    let mut embed = serenity::builder::CreateEmbed::default();
    embed
        .title(friendly_title)
        .color(crate::constants::EMBED_COLOR_ACCENT);

    let formatted = format_api_response(&json, path, &mut embed);

    if formatted {
        cmd.create_followup_message(&ctx.http, |m| m.add_embed(embed))
            .await?;
    } else {
        let pretty = serde_json::to_string_pretty(&json)?;
        if pretty.len() <= 4000 {
            embed.description(format!("```json\n{}\n```", pretty));
            cmd.create_followup_message(&ctx.http, |m| m.add_embed(embed))
                .await?;
        } else {
            let filename = format!("{}.json", path.trim_start_matches("/").replace("/", "_"));
            cmd.create_followup_message(&ctx.http, |m| {
                m.add_file((pretty.as_bytes(), filename.as_str()))
                    .content(format!(
                        "{} — Response attached ({} bytes)",
                        friendly_title,
                        pretty.len()
                    ))
            })
            .await?;
        }
    }

    Ok(())
}

pub async fn auction_embed(
    client: &reqwest::Client,
    donut_key: &str,
    path: &str,
    title: &str,
    search: Option<&str>,
    sort: Option<&str>,
    current_page: u32,
) -> Result<
    (serenity::builder::CreateEmbed, Vec<serde_json::Value>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let url = format!("https://api.donutsmp.net{}", path);

    // Build request based on whether we have search/sort parameters
    let response = if search.is_some() || sort.is_some() {
        // Use POST with JSON body for search/sort
        let mut body = serde_json::json!({});

        if let Some(search_term) = search {
            body["search"] = serde_json::Value::String(search_term.to_string());
        }

        if let Some(sort_type) = sort {
            body["sort"] = serde_json::Value::String(sort_type.to_string());
        }

        client
            .post(&url)
            .header("Authorization", format!("Bearer {}", donut_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
    } else {
        // Use GET for basic requests without search/sort
        client
            .get(&url)
            .header("Authorization", format!("Bearer {}", donut_key))
            .send()
            .await?
    };

    let response_text = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&response_text)?;

    let mut embed = serenity::builder::CreateEmbed::default();
    embed
        .title(title)
        .color(crate::constants::EMBED_COLOR_ACCENT);

    if let Some(result) = json.get("result") {
        if let Some(items) = result.as_array() {
            format_auction_response_with_page(items, &mut embed, current_page);
        } else {
            embed.description("❌ No items found or invalid response format");
        }
    } else if let Some(status) = json.get("status") {
        embed.description(format!(
            "❌ API Error: {}",
            status.as_str().unwrap_or("Unknown error")
        ));
    } else {
        embed.description("❌ Unexpected response format");
    }

    let mut footer_parts = Vec::new();
    if let Some(search_term) = search {
        footer_parts.push(format!("🔍 Search: '{}'", search_term));
    }
    if let Some(sort_type) = sort {
        let sort_display = match sort_type {
            "lowest_price" => "💰 Lowest Price",
            "highest_price" => "💸 Highest Price",
            "recently_listed" => "🕒 Recently Listed",
            "last_listed" => "📅 Last Listed",
            _ => sort_type,
        };
        footer_parts.push(format!("📊 Sort: {}", sort_display));
    }

    if !footer_parts.is_empty() {
        embed.footer(|f| f.text(footer_parts.join(" | ")));
    }

    let items = json
        .get("result")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    Ok((embed, items))
}

pub async fn send_stats(
    cmd: &ApplicationCommandInteraction,
    ctx: &Context,
    client: &Client,
    donut_key: &str,
    path: &str,
    player_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Immediately ack with a deferred response (allows more than 3s processing)
    cmd.create_interaction_response(&ctx.http, |resp| resp.kind(serenity::model::prelude::interaction::InteractionResponseType::DeferredChannelMessageWithSource)).await?;

    let url = format!("https://api.donutsmp.net{}", path);
    let res = client
        .get(&url)
        .bearer_auth(donut_key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?;

    if !res.status().is_success() {
        let error_msg = format!("❌ API Error: {}", res.status());
        cmd.edit_original_interaction_response(&ctx.http, |response| {
            response.embed(|e| {
                e.title("Error")
                    .description(error_msg)
                    .color(crate::constants::EMBED_COLOR_ERROR)
            })
        })
        .await?;
        return Ok(());
    }

    let response_text = res.text().await?;
    let json: Value = match serde_json::from_str(&response_text) {
        Ok(j) => j,
        Err(e) => {
            let error_msg = format!("❌ Failed to parse response: {}", e);
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.embed(|e| {
                    e.title("Error")
                        .description(error_msg)
                        .color(crate::constants::EMBED_COLOR_ERROR)
                })
            })
            .await?;
            return Ok(());
        }
    };

    cmd.edit_original_interaction_response(&ctx.http, |response| {
        response.embed(|embed| {
            format_stats_response(&json, embed, player_name);
            embed
        })
    })
    .await?;

    Ok(())
}

pub async fn send_leaderboard(
    cmd: &ApplicationCommandInteraction,
    ctx: &Context,
    client: &Client,
    donut_key: &str,
    path: &str,
    lb_type: &str,
    page: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Immediately ack with a deferred response (allows more than 3s processing)
    cmd.create_interaction_response(&ctx.http, |resp| resp.kind(serenity::model::prelude::interaction::InteractionResponseType::DeferredChannelMessageWithSource)).await?;

    let url = format!("https://api.donutsmp.net{}", path);
    let res = client
        .get(&url)
        .bearer_auth(donut_key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?;

    if !res.status().is_success() {
        let error_msg = format!("❌ API Error: {}", res.status());
        cmd.edit_original_interaction_response(&ctx.http, |response| {
            response.embed(|e| {
                e.title("Error")
                    .description(error_msg)
                    .color(crate::constants::EMBED_COLOR_ERROR)
            })
        })
        .await?;
        return Ok(());
    }

    let response_text = res.text().await?;
    let json: Value = match serde_json::from_str(&response_text) {
        Ok(j) => j,
        Err(e) => {
            let error_msg = format!("❌ Failed to parse response: {}", e);
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.embed(|e| {
                    e.title("Error")
                        .description(error_msg)
                        .color(crate::constants::EMBED_COLOR_ERROR)
                })
            })
            .await?;
            return Ok(());
        }
    };

    cmd.edit_original_interaction_response(&ctx.http, |response| {
        response
            .embed(|embed| {
                format_leaderboard_response(&json, embed, lb_type, page);
                embed
            })
            .components(|c| {
                use crate::components::lb_buttons;
                lb_buttons(c, page, lb_type)
            })
    })
    .await?;

    Ok(())
}
