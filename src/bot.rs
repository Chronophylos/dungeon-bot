use anyhow::{Context, Result};
use log::{debug, error, info, trace};
use std::{collections::HashMap, convert::TryInto};
use twitchchat::{
    messages::Commands, messages::Privmsg, runner::NotifyHandle, AsyncRunner, Status, UserConfig,
};

pub struct Args<'a, 'b: 'a> {
    pub msg: &'a Privmsg<'b>,
    pub writer: &'a mut twitchchat::Writer,
    pub quit: NotifyHandle,
    pub arguments: Vec<&'a str>,
}

impl Args<'_, '_> {
    pub fn user_id(&self) -> Result<i32> {
        Ok(self.msg.user_id().context("missing user id")?.try_into()?)
    }
}

pub trait Command: Send + Sync {
    fn handle(&mut self, args: Args<'_, '_>) -> Result<()>;
}

impl<F> Command for F
where
    F: Fn(Args<'_, '_>) -> Result<()>,
    F: Send + Sync,
{
    fn handle(&mut self, args: Args<'_, '_>) -> Result<()> {
        (self)(args)
    }
}

pub struct Bot {
    prefix: char,
    commands: HashMap<String, Box<dyn Command>>,
}

impl Bot {
    pub fn new(prefix: char) -> Self {
        Self {
            prefix,
            commands: HashMap::new(),
        }
    }

    // add this command to the bot
    pub fn with_command<S, C>(mut self, name: S, cmd: C) -> Self
    where
        S: ToString,
        C: Command + 'static,
    {
        self.commands.insert(name.to_string(), Box::new(cmd));
        self
    }

    // run the bot until its done
    pub async fn run(
        &mut self,
        user_config: &UserConfig,
        channels: &[String],
    ) -> anyhow::Result<()> {
        // this can fail if DNS resolution cannot happen
        let connector = twitchchat::connector::smol::Connector::twitch()?;

        let mut runner = AsyncRunner::connect(connector, user_config).await?;
        info!("connecting, I'm: {}", runner.identity.username());

        for channel in channels {
            info!("joining: {}", channel);
            if let Err(err) = runner.join(channel).await {
                error!("error while joining '{}': {}", channel, err);
            }
        }

        debug!("starting main loop");
        self.main_loop(&mut runner).await
    }

    // the main loop of the bot
    async fn main_loop(&mut self, runner: &mut AsyncRunner) -> anyhow::Result<()> {
        // this is clonable, but we can just share it via &mut
        // this is rate-limited writer
        let mut writer = runner.writer();
        // this is clonable, but using it consumes it.
        // this is used to 'quit' the main loop
        let quit = runner.quit_handle();

        loop {
            // this drives the internal state of the crate
            match runner.next_message().await? {
                // if we get a Privmsg (you'll get an Commands enum for all messages received)
                Status::Message(Commands::Privmsg(pm)) => {
                    trace!("got privmsg: {}", pm.data());

                    // see if its a command and do stuff with it
                    if let Some((Some(cmd), arguments)) = self.parse_command(pm.data()) {
                        if let Some(command) = self.commands.get_mut(cmd) {
                            debug!("dispatching to: {}", cmd.escape_debug());

                            let args = Args {
                                msg: &pm,
                                writer: &mut writer,
                                quit: quit.clone(),
                                arguments,
                            };

                            if let Err(err) = command.handle(args) {
                                error!("Could not execute command: {}", err);
                            }
                        }
                    }
                }
                // stop if we're stopping
                Status::Quit | Status::Eof => break,
                // ignore the rest
                Status::Message(..) => continue,
            }
        }

        debug!("end of main loop");
        Ok(())
    }

    fn parse_command<'a>(&self, input: &'a str) -> Option<(Option<&'a str>, Vec<&'a str>)> {
        input.strip_prefix(self.prefix).map(|input| {
            let mut split = input.trim_start().split_whitespace();
            (split.next(), split.collect())
        })
    }
}
