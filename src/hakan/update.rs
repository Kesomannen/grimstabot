use std::path::Path;

use anyhow::Result;
use serenity::all::{ChannelId, CreateMessage, Http, RoleId};

use crate::AppState;

use super::{Product, Report};

const UPDATE_CHANNEL: ChannelId = ChannelId::new(1359621010726326432);
const UPDATE_PING_ROLE: RoleId = RoleId::new(1359807749780930570);

#[tracing::instrument]
pub async fn send(http: &Http, state: &AppState) -> Result<()> {
    let report = super::create_report(state).await?;

    insert_report(&report, state).await?;

    let plot_path = super::plot::create(state).await?;
    let url = super::plot::upload(&plot_path, state).await?;

    let embed = super::create_embed(&products).image(url);

    UPDATE_CHANNEL
        .send_message(
            http,
            CreateMessage::new()
                .content(format!("<@&{UPDATE_PING_ROLE}>"))
                .add_embed(embed),
        )
        .await?;

    tokio::fs::remove_file(&plot_path).await.ok();

    Ok(())
}

async fn insert_report(report: &Report, state: &AppState) -> Result<()> {
    let mut tx = state.db.begin().await?;

    let report = sqlx::query!("INSERT INTO reports DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;

    for product in products {
        let Product {
            name,
            manufacturer_name,
            comparative_price,
            comparative_price_text,
            url,
            ingredient,
            ..
        } = product;

        sqlx::query!(
            "INSERT INTO products 
            (report_id, ingredient_id, name, manufacturer_name, comparative_price, comparative_price_text, url, store)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            report.id,
            ingredient.id,
            name,
            manufacturer_name,
            comparative_price,
            comparative_price_text,
            url,
            "coop"
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
