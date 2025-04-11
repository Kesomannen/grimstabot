use anyhow::Result;
use serenity::all::{ChannelId, CreateMessage, Http, RoleId};

use crate::AppState;

const UPDATE_CHANNEL: ChannelId = ChannelId::new(1359621010726326432);
const UPDATE_PING_ROLE: RoleId = RoleId::new(1359807749780930570);

#[tracing::instrument]
pub async fn send(http: &Http, state: &AppState) -> Result<()> {
    let report = super::create_report(state).await?;

    super::save_report(&report, state).await?;

    let plot_path = super::plot::create_total(state).await?;
    let url = super::plot::upload(&plot_path, state).await?;

    let embed = super::create_embed(&report).image(url);

    UPDATE_CHANNEL
        .send_message(
            http,
            CreateMessage::new()
                .content(format!("<@&{UPDATE_PING_ROLE}>"))
                .add_embed(embed),
        )
        .await?;

    //tokio::fs::remove_file(&plot_path).await.ok();

    Ok(())
}
