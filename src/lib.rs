use std::sync::Arc;

use anyhow::Result;
use serenity::{
    all::{
        Color, Command, Context, CreateEmbed, EditInteractionResponse, EventHandler, GuildId, Http,
        Interaction, Ready,
    },
    async_trait,
};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

mod commands;
pub mod hakan;

#[derive(Debug, Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    s3: s3::Bucket,
    http: reqwest::Client,
}

impl AppState {
    pub fn new(db: sqlx::PgPool, s3: s3::Bucket, http: reqwest::Client) -> Self {
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

        let guild_id = GuildId::new(916599635001368616);

        guild_id
            .set_commands(&ctx.http, vec![commands::hakan::register()])
            .await
            .unwrap();

        if let Err(err) = setup_hakan_chron_job(ctx.http.clone(), self.state.clone()).await {
            error!("failed to start hakan chron job: {err:#}");
        }

        //hakan::update::send(&ctx.http, &self.state).await.unwrap();

        info!(username = ready.user.name, "ready");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!(name = command.data.name, "received command interaction");

            if let Err(err) = commands::hakan::run(&command, &ctx, &self.state).await {
                error!("failed to handle command: {err:#}");

                let response = CreateEmbed::new()
                    .color(Color::RED)
                    .title("An error occured!")
                    .description(format!("{err:#?}"));

                command
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new().add_embed(response),
                    )
                    .await
                    .ok();
            }
        }
    }
}

async fn setup_hakan_chron_job(http: Arc<Http>, state: AppState) -> Result<()> {
    let scheduler = JobScheduler::new().await?;

    let job = Job::new_async("0 0 7 * * *", move |_uuid, _l| {
        let http = http.clone();
        let state = state.clone();

        Box::pin(async move {
            info!("running daily hakan update");

            if let Err(err) = hakan::update::send(&http, &state).await {
                error!("failed to send hakan update: {err:#}");
            }
        })
    })
    .unwrap();

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(())
}
