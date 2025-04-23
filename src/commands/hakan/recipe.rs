use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, RoleId,
};

use crate::AppState;

pub fn register() -> CreateCommand {
    CreateCommand::new("håkanrecept").description("Visa håkanreceptet.")
}

const RECIPE: &'static str = "Slib slorb";

#[tracing::instrument]
pub async fn run(
    interaction: &CommandInteraction,
    ctx: &Context,
    state: &AppState,
) -> anyhow::Result<()> {
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().add_embed(
                    CreateEmbed::new()
                        .title("🍰 Håkanrecept")
                        .description(RECIPE),
                ),
            ),
        )
        .await?;

    Ok(())
}
