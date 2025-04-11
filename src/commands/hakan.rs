use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, InstallationContext,
};

use crate::AppState;

pub fn register() -> CreateCommand {
    CreateCommand::new("håkan")
        .description("Skriv ut den nuvarande håkankursen")
        .add_integration_type(InstallationContext::User)
}

pub async fn run(
    interaction: CommandInteraction,
    ctx: &Context,
    state: &AppState,
) -> anyhow::Result<()> {
    let report = crate::hakan::create_report(state).await?;
    let embed = crate::hakan::create_embed(&report);

    let msg = CreateInteractionResponseMessage::new().embed(embed);
    let response = CreateInteractionResponse::Message(msg);

    interaction.create_response(&ctx.http, response).await?;

    Ok(())
}
