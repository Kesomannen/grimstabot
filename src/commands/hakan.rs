use anyhow::bail;
use serenity::all::{
    colours::roles::DARK_GREEN, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, CreateEmbed, EditInteractionResponse, InstallationContext,
};

use crate::{hakan, AppState};

pub fn register() -> CreateCommand {
    CreateCommand::new("håkankurs")
        .description("Visa håkankursen.")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "total",
            "Visa totalpris över tid.",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "butik",
            "Visa totalpris per butik över tid.",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "ingrediens",
            "Visa ingredienspris över tid.",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "uppdatera",
            "Uppdatera håkankursen med den senaste datan och skicka en rapport.",
        ))
        .add_integration_type(InstallationContext::User)
}

#[tracing::instrument]
pub async fn run(
    interaction: &CommandInteraction,
    ctx: &Context,
    state: &AppState,
) -> anyhow::Result<()> {
    interaction.defer(&ctx.http).await?;

    bail!("aaaaaaaaaaaaaaaa");

    let command = &interaction.data.options[0].name;

    //let report = hakan::create_report(state).await?;

    let (title, plot_url) = match command.as_str() {
        "total" => ("Håkankursen", hakan::plot::create_total(state).await?),
        "butik" => (
            "Håkankurs per butik",
            hakan::plot::create_by_store(state).await?,
        ),
        "ingrediens" => (
            "Håkankurs per ingrediens",
            hakan::plot::create_by_ingredient(state).await?,
        ),
        "uppdatera" => {
            let _ = hakan::update::send(&ctx.http, state).await?;
            let response = EditInteractionResponse::new().content(format!(
                "☀️ Uppdatering klar! Se <#{}>.",
                hakan::update::UPDATE_CHANNEL
            ));
            interaction.edit_response(&ctx.http, response).await?;

            return Ok(());
        }
        _ => bail!("unknown subcommand"),
    };

    let embed = CreateEmbed::new()
        .color(DARK_GREEN)
        .title(title)
        .image(plot_url);

    let response = EditInteractionResponse::new().add_embed(embed);

    interaction.edit_response(&ctx.http, response).await?;

    Ok(())
}
