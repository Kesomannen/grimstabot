use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, InstallationContext,
};

use crate::Db;

pub fn register() -> CreateCommand {
    CreateCommand::new("håkan")
        .description("Skriv ut den nuvarande håkankursen")
        .add_integration_type(InstallationContext::User)
}

pub async fn run(interaction: CommandInteraction, ctx: &Context, db: &Db) -> anyhow::Result<()> {
    let products = crate::hakan::get_products(db).await?;
    let embed = crate::hakan::create_embed(&products);

    let msg = CreateInteractionResponseMessage::new().embed(embed);
    let response = CreateInteractionResponse::Message(msg);

    interaction.create_response(&ctx.http, response).await?;

    Ok(())
}
