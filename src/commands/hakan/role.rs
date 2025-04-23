use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, RoleId,
};

use crate::{hakan, AppState};

pub fn register() -> CreateCommand {
    CreateCommand::new("håkanroll")
        .description("Få rollen för håkanälskare eller ta bort den från dig själv.")
}

#[tracing::instrument]
pub async fn run(
    interaction: &CommandInteraction,
    ctx: &Context,
    state: &AppState,
) -> anyhow::Result<()> {
    let member = crate::GUILD.member(&ctx.http, interaction.user.id).await?;

    let has_role = interaction
        .user
        .has_role(&ctx.http, crate::GUILD, hakan::update::PING_ROLE)
        .await?;

    let (title, description) = if has_role {
        member
            .remove_role(&ctx.http, hakan::update::PING_ROLE)
            .await?;
        ("💔 NEEEEJ!! 😭", "Tog bort din håkanälskarroll.")
    } else {
        member.add_role(&ctx.http, hakan::update::PING_ROLE).await?;
        ("☀️ Välkommen! 😊", "La till håkanälskarrollen.")
    };

    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Color::DARK_GREEN);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await?;

    Ok(())
}
