use std::sync::Arc;

use chrono::{Local, Timelike};
use serenity::{
    all::{Command, Context, EventHandler, Http, Interaction, Ready},
    async_trait,
};
use tokio::time::Instant;
use tracing::{error, info};

mod commands;
pub mod hakan;

#[derive(Debug, Clone)]
pub struct AppState {
    db: sqlx::SqlitePool,
    s3: s3::Bucket,
    http: reqwest::Client,
}

impl AppState {
    pub fn new(db: sqlx::SqlitePool, s3: s3::Bucket, http: reqwest::Client) -> Self {
        Self { db, s3, http }
    }
}

pub struct Bot {
    state: AppState,
}

impl Bot {
    pub fn new(state: AppState) -> Self {
        Self { state }
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

        //send_hakan_update(&http, &db, &s3).await.unwrap();
        tokio::spawn(hakan_loop_task(ctx.http.clone(), self.state.clone()));

        info!(username = ready.user.name, "ready");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!(name = command.data.name, "received command interaction");

            if let Err(err) = commands::hakan::run(command, &ctx, &self.state).await {
                error!("failed to handle command: {err}");
            }
        }
    }
}

async fn hakan_loop_task(http: Arc<Http>, state: AppState) {
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

        if let Err(err) = hakan::update::send(&http, &state).await {
            error!("failed to send hakan update: {err:#}");
        }
    }
}
