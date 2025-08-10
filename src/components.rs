use serenity::{builder::CreateComponents, model::prelude::component::ButtonStyle};

pub fn auction_buttons<'a>(
    components: &'a mut CreateComponents,
    current_page: u32,
    search: Option<&str>,
    sort: Option<&str>,
) -> &'a mut CreateComponents {
    let search_encoded = search
        .map(|s| s.replace(" ", "%20"))
        .unwrap_or_else(|| "".to_string());
    let sort_str = sort.unwrap_or("");

    components.create_action_row(|row| {
        row.create_button(|button| {
            button
                .style(ButtonStyle::Secondary)
                .label("⬅")
                .custom_id(format!(
                    "auction_prev_{}_{}_{}",
                    current_page, search_encoded, sort_str
                ))
                .disabled(current_page <= 1)
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Primary)
                .label(format!("Page {}", current_page))
                .custom_id("auction_current")
                .disabled(true)
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Secondary)
                .label("➡")
                .custom_id(format!(
                    "auction_next_{}_{}_{}",
                    current_page, search_encoded, sort_str
                ))
        })
    })
}

pub fn txn_buttons<'a>(
    components: &'a mut CreateComponents,
    current_page: u32,
    search: Option<&str>,
    sort: Option<&str>,
) -> &'a mut CreateComponents {
    let search_encoded = search
        .map(|s| s.replace(" ", "%20"))
        .unwrap_or_else(|| "".to_string());
    let sort_str = sort.unwrap_or("");

    components.create_action_row(|row| {
        row.create_button(|button| {
            button
                .style(ButtonStyle::Secondary)
                .label("⬅")
                .custom_id(format!(
                    "transaction_prev_{}_{}_{}",
                    current_page, search_encoded, sort_str
                ))
                .disabled(current_page <= 1)
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Primary)
                .label(format!("Page {}", current_page))
                .custom_id("transaction_current")
                .disabled(true)
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Secondary)
                .label("➡")
                .custom_id(format!(
                    "transaction_next_{}_{}_{}",
                    current_page, search_encoded, sort_str
                ))
        })
    })
}

pub fn lb_buttons<'a>(
    components: &'a mut CreateComponents,
    current_page: u32,
    lb_type: &str,
) -> &'a mut CreateComponents {
    components.create_action_row(|row| {
        row.create_button(|button| {
            button
                .style(ButtonStyle::Secondary)
                .label("⬅")
                .custom_id(format!("leaderboard_prev_{}_{}", current_page, lb_type))
                .disabled(current_page <= 1)
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Primary)
                .label(format!("Page {}", current_page))
                .custom_id("leaderboard_current")
                .disabled(true)
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Secondary)
                .label("➡")
                .custom_id(format!("leaderboard_next_{}_{}", current_page, lb_type))
        })
    })
}
