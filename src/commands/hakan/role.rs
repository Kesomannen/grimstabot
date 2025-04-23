use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, RoleId,
};

use crate::AppState;

pub fn register() -> CreateCommand {
    CreateCommand::new("älskahåkan")
        .description("Få håkanälskarrollen eller ta bort den från dig själv.")
}

const ROLE_ID: RoleId = RoleId::new(1359807749780930570);

#[tracing::instrument]
pub async fn run(
    interaction: &CommandInteraction,
    ctx: &Context,
    state: &AppState,
) -> anyhow::Result<()> {
    let member = crate::GUILD_ID
        .member(&ctx.http, interaction.user.id)
        .await?;

    let has_role = interaction
        .user
        .has_role(&ctx.http, crate::GUILD_ID, ROLE_ID)
        .await?;

    let response = if has_role {
        member.remove_role(&ctx.http, ROLE_ID).await?;
        "💔💔 NEEEEEJ!"
    } else {
        member.add_role(&ctx.http, ROLE_ID).await?;
        "☀️😊 Välkommen!"
    };

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(response),
            ),
        )
        .await?;

    Ok(())
}
