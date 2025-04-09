use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, InstallationContext,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("håkan")
        .description("Skriv ut den nuvarande håkankursen")
        .add_integration_type(InstallationContext::User)
}

pub async fn run(ctx: &Context, interaction: CommandInteraction) -> anyhow::Result<()> {
    let products = crate::hakan::get_products().await?;
    let embed = crate::hakan::create_embed(&products);

    let msg = CreateInteractionResponseMessage::new().embed(embed);
    let response = CreateInteractionResponse::Message(msg);

    interaction.create_response(&ctx.http, response).await?;

    Ok(())
}
