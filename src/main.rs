#![forbid(unsafe_code)]

use anyhow::Result;
use dungeon_bot::{
    bot::{Args, Bot},
    db::Player,
    Config,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use sqlx::{postgres::PgPoolOptions, PgPool};
use twitchchat::PrivmsgExt as _;

fn register(args: Args, pool: &PgPool) -> Result<()> {
    let uid = args.user_id()?;
    let player = Player::new(pool, uid);

    if smol::block_on(player.exists())? {
        args.writer
            .reply(args.msg, "You are already on my list 📝")?;
        return Ok(());
    }

    smol::block_on(player.insert())?;

    args.writer.reply(
        args.msg,
        "I added you to my records. Your character has been created",
    )?;

    Ok(())
}

fn unregister(args: Args, pool: &PgPool) -> Result<()> {
    let uid = args.user_id()?;
    let player = Player::new(pool, uid);

    if !smol::block_on(player.exists())? {
        args.writer.reply(
            args.msg,
            "I cannot delete what doesn't exist: You are not in my records",
        )?;

        return Ok(());
    }

    match args.arguments.get(0) {
        Some(&"confirm") => {
            smol::block_on(player.delete())?;
            args.writer
                .reply(args.msg, "I removed you from my records 🔥🗒🔥")?;
        }
        Some(_) => {
            args.writer.reply(
                args.msg,
                &format!(
                    "Type `{} unregister confirm` if you want to unregister",
                    '>'
                ),
            )?;
        }
        _ => {
            args.writer.reply(args.msg, &format!("❗ This will delete your character and all of your progress ❗ Are you sure about that? Type `{} unregister confirm` if you want to unregister", '>'))?;
        }
    }

    Ok(())
}

fn enter(args: Args, pool: &PgPool) -> Result<()> {
    let uid = args.user_id()?;
    let player = Player::new(pool, uid);

    if !smol::block_on(player.exists())? {
        args.writer.reply(
            args.msg,
            &format!("You're not registered. Register with `{} register`", '>'),
        )?;

        return Ok(());
    }

    if let Some(cooldown) = smol::block_on(player.can_enter())? {
        args.writer.reply(
            args.msg,
            &format!("You cannot enter the dungeon. Please wait for {}", cooldown),
        )?;

        return Ok(());
    }

    let stats = smol::block_on(player.get_stats())?;
    dbg!(stats);

    Ok(())
}

fn main() -> Result<()> {
    SimpleLogger::new().with_level(LevelFilter::Debug).init()?;

    let config = Config::load("config.ron")?;

    let pool = smol::block_on(
        PgPoolOptions::new()
            .max_connections(10)
            .connect(config.database_url()),
    )?;

    smol::block_on(sqlx::migrate!("db/migrations").run(&pool))?;

    let mut bot = Bot::new('>')
        .with_command("bot", |args: Args| {
            let output = format!("I am a bot :)");

            args.writer.reply(args.msg, &output)?;

            Ok(())
        })
        .with_command("register", {
            let pool = pool.clone();
            move |args: Args| register(args, &pool)
        })
        .with_command("unregister", {
            let pool = pool.clone();
            move |args: Args| unregister(args, &pool)
        })
        //.with_command("test", |args: Args| {
        //    args.writer
        //        .reply(args.msg, &format!("args: {:?}", args.arguments))?;
        //    Ok(())
        //})
        .with_command("enter", {
            let pool = pool.clone();
            move |args: Args| enter(args, &pool)
        });

    // run the bot in the executor
    smol::block_on(async move { bot.run(&config.user_config()?, config.channels()).await })
}
