use std::{env, str::FromStr};

use grimstabot::hakan;
use serenity::all::GatewayIntents;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    SqlitePool,
};
use tracing::info;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let options = SqliteConnectOptions::from_str(&db_url)
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true);

    info!(db_url, "connecting to database");

    let db = SqlitePool::connect_with(options)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("failed to run database migrations");

    let s3 = s3::Bucket::new(
        "mod-platform",
        s3::Region::DoFra1,
        s3::creds::Credentials::from_env().expect("failed to create s3 credentials"),
    )
    .expect("failed to connect to s3 bucket");

    let http = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0",
        )
        .build()
        .expect("failed to build reqwest");

    let state = grimstabot::AppState::new(db, *s3, http);

    //let report = hakan::create_report(&state).await.unwrap();
    //hakan::save_report(&report, &state).await.unwrap();
    //hakan::plot::create(&state).await.unwrap();

    let bot = grimstabot::Bot::new(state);

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let intents = GatewayIntents::non_privileged();

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(bot)
        .await
        .expect("error while starting client");

    client.start().await.unwrap()
}
