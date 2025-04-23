use std::sync::Arc;

use anyhow::{anyhow, Result};
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

const GUILD: GuildId = GuildId::new(916599635001368616);

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let commands = [
            commands::hakan::stock::register(),
            commands::hakan::role::register(),
            commands::hakan::recipe::register(),
            commands::hakan::wr::register(),
        ];

        for command in commands {
            if let Err(err) = Command::create_global_command(&ctx.http, command).await {
                error!("failed to register command: {err}");
            }
        }

        /*
        guild_id
            .set_commands(&ctx.http, vec![commands::hakan::stock::register()])
            .await
            .unwrap();
        */

        if let Err(err) = setup_hakan_chron_job(ctx.http.clone(), self.state.clone()).await {
            error!("failed to start hakan chron job: {err:#}");
        }

        //hakan::update::send(&ctx.http, &self.state).await.unwrap();

        info!(username = ready.user.name, "ready");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!(name = command.data.name, "received command interaction");

            let res = match command.data.name.as_str() {
                "håkankurs" => commands::hakan::stock::run(&command, &ctx, &self.state).await,
                "håkanroll" => commands::hakan::role::run(&command, &ctx, &self.state).await,
                "håkanrecept" => commands::hakan::recipe::run(&command, &ctx, &self.state).await,
                "håkanrekord" => commands::hakan::wr::run(&command, &ctx, &self.state).await,
                _ => Err(anyhow!("unknown command name")),
            };

            if let Err(err) = res {
                error!("failed to handle command: {err:#}");

                let response = CreateEmbed::new()
                    .color(Color::RED)
                    .title("An error occured!")
                    .description(format!("`{err:#}`"));

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
