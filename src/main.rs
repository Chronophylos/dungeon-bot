#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use dungeon_bot::{
    bot::{Args, Bot},
    db::Player,
    Config,
};
use lazy_static::lazy_static;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use smol::future::FutureExt as _;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use std::time::Instant;
use twitchchat::PrivmsgExt as _;

const PREFIX: char = '>';
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_REPO: &str = env!("CARGO_PKG_REPOSITORY");

lazy_static! {
    static ref BOOT_TIME: Instant = Instant::now();
}

fn register(args: Args, pool: &PgPool) -> Result<()> {
    let uid = args.user_id()?;
    let player = Player::new(pool, uid);

    if smol::block_on(player.exists())? {
        args.writer
            .reply(args.raw, "You are already on my list 📝")?;
        return Ok(());
    }

    smol::block_on(player.insert())?;

    args.writer.reply(
        args.raw,
        "I added you to my records. Your character has been created",
    )?;

    Ok(())
}

fn unregister(args: Args, pool: &PgPool) -> Result<()> {
    let uid = args.user_id()?;
    let player = Player::new(pool, uid);

    if !smol::block_on(player.exists())? {
        args.writer.reply(
            args.raw,
            "I cannot delete what doesn't exist: You are not in my records",
        )?;

        return Ok(());
    }

    match args.msg.arguments.get(0) {
        Some(&"confirm") => {
            smol::block_on(player.delete())?;
            args.writer
                .reply(args.raw, "I removed you from my records 🔥🗒")?
        }
        Some(_) => args.writer.reply(
                args.raw,
                &format!(
                    "Type `{} unregister confirm` if you want to unregister",
                    PREFIX
                ),
            )?,
        _ => args.writer.reply(
                args.raw,
                &format!(
                    "❗ This will delete your character and all of your progress. Are you sure about that? Type `{} unregister confirm` if you want to unregister",
                    PREFIX
                ),
            )?,
    }

    Ok(())
}

fn enter(args: Args, pool: &PgPool) -> Result<()> {
    let uid = args.user_id()?;
    let player = Player::new(pool, uid);

    if !smol::block_on(player.exists())? {
        args.writer.reply(
            args.raw,
            &format!("You're not registered. Register with `{} register`", PREFIX),
        )?;

        return Ok(());
    }

    if let Some(cooldown) = smol::block_on(player.can_enter())? {
        args.writer.reply(
            args.raw,
            &format!("You cannot enter the dungeon. Please wait for {}", cooldown),
        )?;

        return Ok(());
    }

    let stats = smol::block_on(player.get_stats())?;

    //dbg!(stats.dps());
    //dbg!(stats.max_health());

    Ok(())
}

fn ping(args: Args) -> Result<()> {
    let latency = args
        .raw
        .tmi_sent_ts()
        .map(|ts| NaiveDateTime::from_timestamp(ts as i64, 0))
        .map(|time| time.signed_duration_since(Utc::now().naive_utc()))
        .map(|duration| duration.to_string())
        .unwrap_or(String::from("unknown"));

    args.writer.reply(
        args.raw,
        &format!(
            "| Uptime: {:?} Message Latency: {}",
            Instant::now().duration_since(*BOOT_TIME),
            latency
        ),
    )?;

    Ok(())
}

fn main() -> Result<()> {
    // evaluate boot time
    let _ = *BOOT_TIME;

    SimpleLogger::new().with_level(LevelFilter::Debug).init()?;

    let config = Config::load("config.ron")?;

    let pool = smol::block_on(
        async {
            PgPoolOptions::new()
                .max_connections(10)
                .connect(config.database_url())
                .await
                .context("Could not connect to database")
        }
        .or(async {
            smol::Timer::after(Duration::from_secs(10)).await;
            Err(anyhow!("Could not connect to database: Timeout"))
        }),
    )?;

    smol::block_on(sqlx::migrate!("db/migrations").run(&pool))?;

    let mut bot = Bot::new('>')
        .with_bot_command(|args: Args| {
            args.writer.reply(
                args.raw,
                &format!(
                    "| {} {} made by Chronophylos in Rust. Prefix: {}. Try `{} help` for more",
                    APP_NAME, APP_VERSION, PREFIX, PREFIX
                ),
            )?;

            Ok(())
        })
        .with_command("register", Vec::new(), {
            let pool = pool.clone();
            move |args: Args| register(args, &pool)
        })
        .with_command("unregister", Vec::new(), {
            let pool = pool.clone();
            move |args: Args| unregister(args, &pool)
        })
        .with_command("enter", vec!["e"], {
            let pool = pool.clone();
            move |args: Args| enter(args, &pool)
        })
        .with_command("help", vec!["commands"], |args: Args| {
            args.writer
                .reply(args.raw, "this command is not yet implemented")?;
            Ok(())
        })
        .with_command("ping", Vec::new(), ping)
        .with_command("repo", vec!["source"], |args: Args| {
            args.writer.reply(
                args.raw,
                &format!("the source code can be found here: {}", APP_REPO),
            )?;
            Ok(())
        });

    // run the bot in the executor
    smol::block_on(bot.run(&config.user_config()?, config.channels()))
}
