use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, RoleId,
};

use crate::AppState;

pub fn register() -> CreateCommand {
    CreateCommand::new("håkanrecept").description("Visa håkanreceptet.")
}

const RECIPE: &'static str = "
Ca 48 bitar.

God som en liten smakbit efter maten.
Gärna med en klick vispgrädde till.

8 ägg
2 ½ dl kakao
10 dl (850 g) strösocker
6 dl (500 g) vetemjöl
1 tsk salt
400 g smält smör

1. Sätt ugnen på 175°.
2. Vispa äggen och sockret pösigt.
3. Rör ner de övriga ingredienserna.
4. Smörj en långpanna och häll upp smeten.
5. Grädda kakan på nedersta falsen i ugnen i ca 35 minuter. Känn efter med en stick när kakan är torr.
6. Skär upp kakan i 5 x 5 cm stora rutor.

Njut 😋";

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
                        .color(Color::DARK_GREEN)
                        .description(RECIPE),
                ),
            ),
        )
        .await?;

    Ok(())
}
