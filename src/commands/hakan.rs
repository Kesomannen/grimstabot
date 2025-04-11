use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
    InstallationContext,
};

use crate::{hakan, AppState};

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
    interaction.defer(&ctx.http).await?;

    //let report = hakan::create_report(state).await?;

    let plot_path = hakan::plot::create_by_store(state).await?;
    let plot_url = hakan::plot::upload(&plot_path, state).await?;

    let embed = CreateEmbed::new().image(plot_url);

    let response = EditInteractionResponse::new().add_embed(embed);

    interaction.edit_response(&ctx.http, response).await?;

    Ok(())
}
