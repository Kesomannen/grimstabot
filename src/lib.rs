use std::sync::Arc;

use chrono::{Local, Timelike};
use hakan::Product;
use serenity::{
    all::{ChannelId, Command, Context, CreateMessage, EventHandler, Http, Interaction, Ready},
    async_trait,
};
use tokio::time::Instant;
use tracing::{error, info};

mod commands;
mod hakan;

pub struct Bot {
    db: sqlx::SqlitePool,
}

impl Bot {
    pub fn new(db: sqlx::SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Err(err) =
            Command::create_global_command(&ctx.http, commands::hakan::register()).await
        {
            error!("failed to register command: {err}");
        }

        let channel_id = ChannelId::new(1359621010726326432);
        let http = ctx.http.clone();
        let db = self.db.clone();
        tokio::spawn(loop_task(channel_id, http, db));

        info!("logged in as {}", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if let Err(err) = commands::hakan::run(&ctx, command).await {
                error!("failed to handle command: {err}");
            }
        }
    }
}

async fn loop_task(channel: ChannelId, http: Arc<Http>, db: sqlx::SqlitePool) {
    loop {
        let now = Local::now();
        let next_9am = if now.hour() < 9 {
            now.date_naive().and_hms_opt(9, 0, 0).unwrap()
        } else {
            (now + chrono::Duration::days(1))
                .date_naive()
                .and_hms_opt(9, 0, 0)
                .unwrap()
        };

        let duration_until = (next_9am - now.naive_local()).to_std().unwrap();
        tokio::time::sleep_until(Instant::now() + duration_until).await;

        if let Err(err) = send_hakan_update(channel, &http, &db).await {
            error!("failed to send hakan update: {err:#}");
        }
    }
}

#[tracing::instrument]
async fn send_hakan_update(
    channel: ChannelId,
    http: &Http,
    db: &sqlx::SqlitePool,
) -> anyhow::Result<()> {
    let products = hakan::get_products().await?;

    let mut tx = db.begin().await?;

    let report = sqlx::query!("INSERT INTO reports DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;

    for product in &products {
        let Product {
            name,
            manufacturer_name,
            price,
            comparative_price,
            comparative_price_text,
            url,
            ..
        } = product;

        sqlx::query!(
            "INSERT INTO product_reports 
            (report_id, name, manufacturer_name, price, comparative_price, comparative_price_text, url, store)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            report.id,
            name,
            manufacturer_name,
            price,
            comparative_price,
            comparative_price_text,
            url,
            "coop"
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let embed = hakan::create_embed(&products);

    channel
        .send_message(http, CreateMessage::new().add_embed(embed))
        .await?;

    Ok(())
}
