use std::cmp::Ordering;

use anyhow::Result;
use chrono::Utc;
use itertools::Itertools;
use serenity::all::{ChannelId, Color, CreateEmbed, CreateMessage, Http, Message, RoleId};

use crate::AppState;

pub const CHANNEL: ChannelId = ChannelId::new(1359621010726326432);
pub const PING_ROLE: RoleId = RoleId::new(1359807749780930570);

#[tracing::instrument]
pub async fn send(http: &Http, state: &AppState) -> Result<Message> {
    let report = super::create_report(state).await?;
    let last_report = super::db::last_products(state).await?;

    super::save_report(&report, state).await?;

    let url = super::plot::create_total(state, false).await?;
    let last_total_price: f64 = last_report.iter().map(|(_, product)| product.price).sum();

    let cheapest_products = report.cheapest().collect_vec();
    let total_price: f64 = cheapest_products
        .iter()
        .map(|(ingredient, _, product)| product.comparative_price * ingredient.amount)
        .sum();

    let fields = cheapest_products
        .iter()
        .sorted_by(|(a_ingredient, _, a), (b_ingredient, _, b)| {
            (a.comparative_price * a_ingredient.amount)
                .total_cmp(&(b.comparative_price * b_ingredient.amount))
                .reverse()
        })
        .map(|(ingredient, store, product)| {
            let last_ord = last_report
                .iter()
                .find(|(name, _)| *name == ingredient.name)
                .map(|(_, last_product)| {
                    product
                        .comparative_price
                        .total_cmp(&last_product.comparative_price)
                })
                .unwrap_or(Ordering::Equal);

            (
                format!(
                    "{}{} `{:0.1}kr`",
                    get_emoji(last_ord),
                    ingredient.name,
                    product.comparative_price * ingredient.amount
                ),
                format!(
                    "[{} {}]({}) ({}) ({}{})",
                    product.manufacturer_name,
                    product.name,
                    product.url,
                    store,
                    product.comparative_price,
                    product.comparative_price_text,
                ),
                false,
            )
        });

    let embed = CreateEmbed::new()
        .title(format!("☀️🍰 Håkanbörsen har öppnat för dagen! 🍰☀️"))
        .color(Color::DARK_GREEN)
        .description(format!(
            "<t:{}>\n# {}`{total_price:0.3}kr`",
            Utc::now().timestamp(),
            get_emoji(total_price.total_cmp(&last_total_price))
        ))
        .fields(fields)
        .image(url);

    let msg = CHANNEL
        .send_message(
            http,
            CreateMessage::new()
                .content(format!("<@&{PING_ROLE}>"))
                .add_embed(embed),
        )
        .await?;

    Ok(msg)
}

fn get_emoji(ord: Ordering) -> &'static str {
    match ord {
        Ordering::Less => "📉 ",
        Ordering::Equal => "",
        Ordering::Greater => "📈 ",
    }
}
