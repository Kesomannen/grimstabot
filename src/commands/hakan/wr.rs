use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

use crate::AppState;

pub fn register() -> CreateCommand {
    CreateCommand::new("håkanrekord")
        .description("Visa det nuvarande världsrekordet för håkan-speedrunning.")
}

const WR_TIME: &str = "6:25";
const WR_VIDEO: &str = "https://www.youtube.com/watch?v=VrIUSd15P6E";
const WR_AUTHOR: &str = "Bo, Bertil, Sixten och Olle Rodin";
const WR_DATE: u64 = 1721426400;

#[tracing::instrument]
pub async fn run(
    interaction: &CommandInteraction,
    ctx: &Context,
    state: &AppState,
) -> anyhow::Result<()> {
    let embed = CreateEmbed::new()
        .title("🍰 Nuvarande håkanrekordet 💨")
        .description(format!("# `{WR_TIME}`\n<t:{WR_DATE}>\nav {WR_AUTHOR}"))
        .color(Color::DARK_GREEN);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(WR_VIDEO)
                    .embed(embed),
            ),
        )
        .await?;

    Ok(())
}
