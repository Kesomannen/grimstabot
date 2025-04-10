use std::sync::Arc;

use chrono::{Local, Timelike};
use hakan::Product;
use serenity::{
    all::{
        ChannelId, Command, Context, CreateMessage, EventHandler, Http, Interaction, Ready, RoleId,
    },
    async_trait,
};
use tokio::time::Instant;
use tracing::{error, info};

mod commands;
pub mod hakan;

type Db = sqlx::SqlitePool;

pub struct Bot {
    db: Db,
}

impl Bot {
    pub fn new(db: Db) -> Self {
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

        let http = ctx.http.clone();
        let db = self.db.clone();
        send_hakan_update(&http, &db).await.unwrap();

        tokio::spawn(loop_task(http, db));

        info!(username = ready.user.name, "ready");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if let Err(err) = commands::hakan::run(command, &ctx, &self.db).await {
                error!("failed to handle command: {err}");
            }
        }
    }
}

const UPDATE_CHANNEL: ChannelId = ChannelId::new(1359621010726326432);
const UPDATE_PING_ROLE: RoleId = RoleId::new(1359807749780930570);

async fn loop_task(http: Arc<Http>, db: sqlx::SqlitePool) {
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

        if let Err(err) = send_hakan_update(&http, &db).await {
            error!("failed to send hakan update: {err:#}");
        }
    }
}

#[tracing::instrument]
async fn send_hakan_update(http: &Http, db: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let products = hakan::get_products(db).await?;

    let mut tx = db.begin().await?;

    let report = sqlx::query!("INSERT INTO reports DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;

    for product in &products {
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

    let embed = hakan::create_embed(&products);

    UPDATE_CHANNEL
        .send_message(
            http,
            CreateMessage::new()
                .content(format!("<@&{UPDATE_PING_ROLE}>"))
                .add_embed(embed),
        )
        .await?;

    Ok(())
}
