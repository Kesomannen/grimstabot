use std::env;

use serenity::all::GatewayIntents;
use sqlx::PgPool;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!(db_url, "connecting to database");

    let db = PgPool::connect(&db_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("failed to run database migrations");

    let http = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0",
        )
        .build()
        .expect("failed to build reqwest");

    let api_key = env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set");

    let storage = grimstabot::storage::Client::new(
        "grimstabot".into(),
        api_key.into(),
        "https://fmqmtfvzpddscjxkntgn.supabase.co/storage/v1".into(),
        http.clone(),
    );

    let state = grimstabot::AppState::new(db, storage, http);

    //let report = hakan::create_report(&state).await.unwrap();
    //hakan::save_report(&report, &state).await.unwrap();
    //hakan::plot::create_by_store(&state).await.unwrap();

    let bot = grimstabot::Bot::new(state);

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let intents = GatewayIntents::non_privileged();

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(bot)
        .await
        .expect("error while starting client");

    client.start().await.unwrap()
}
