use std::{env, str::FromStr};

use serenity::all::GatewayIntents;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    SqlitePool,
};
use tracing::info;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().compact().init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let options = SqliteConnectOptions::from_str(&db_url)
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true);

    info!(db_url, "connecting to database");

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    let res = sqlx::query!("SELECT * FROM product_reports")
        .fetch_all(&pool)
        .await
        .unwrap();
    dbg!(&res);

    let bot = grimstabot::Bot::new(pool);

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let intents = GatewayIntents::non_privileged();

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(bot)
        .await
        .expect("error while starting client");

    client.start().await.unwrap()
}
