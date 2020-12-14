use anyhow::{Context, Result};
use log::{debug, error, info, trace};
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use std::{collections::HashMap, convert::TryInto};
use twitchchat::{
    messages::Commands, messages::Privmsg, runner::NotifyHandle, AsyncRunner, Status, UserConfig,
};

const GLOBAL_PREFIX: char = '!';

pub struct Args<'a, 'b: 'a> {
    pub raw: &'a Privmsg<'b>,
    pub msg: Message<'a>,
    pub writer: &'a mut twitchchat::Writer,
    pub quit: NotifyHandle,
}

impl Args<'_, '_> {
    pub fn user_id(&self) -> Result<i32> {
        Ok(self.raw.user_id().context("missing user id")?.try_into()?)
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
    bot_command: Box<dyn Command>,
    commands: HashMap<String, Box<dyn Command>>,
    aliases: HashMap<String, String>,
}

impl Bot {
    pub fn new(prefix: char) -> Self {
        Self {
            prefix,
            bot_command: Box::new(|_: Args| unimplemented!()),
            commands: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    // add this command to the bot
    pub fn with_command<S, C>(mut self, name: S, aliases: Vec<S>, cmd: C) -> Self
    where
        S: ToString,
        C: Command + 'static,
    {
        self.commands.insert(name.to_string(), Box::new(cmd));

        for alias in aliases {
            self.aliases.insert(alias.to_string(), name.to_string());
        }
        self.aliases.insert(name.to_string(), name.to_string());

        self
    }

    pub fn with_bot_command<C>(mut self, cmd: C) -> Self
    where
        C: Command + 'static,
    {
        self.bot_command = Box::new(cmd);
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
                    if let Some(msg) = self.parse_command(pm.data()) {
                        if let Some(command) = self.get_command(&msg) {
                            debug!("dispatching to: {}", msg.command.escape_debug());

                            let args = Args {
                                raw: &pm,
                                msg,
                                writer: &mut writer,
                                quit: quit.clone(),
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

    fn parse_command<'a>(&self, input: &'a str) -> Option<Message<'a>> {
        Message::parse(input).ok()
    }

    fn get_command(&mut self, message: &Message<'_>) -> Option<&mut Box<dyn Command>> {
        let command = message.command.to_ascii_lowercase();
        if message.prefix == self.prefix || message.prefix == GLOBAL_PREFIX {
            if command == "bot" {
                return Some(&mut self.bot_command);
            }
        }

        if message.prefix != self.prefix {
            None
        } else if let Some(name) = self.aliases.get(&command) {
            self.commands.get_mut(name)
        } else {
            None
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Message<'a> {
    pub prefix: char,
    pub command: &'a str,
    pub arguments: Vec<&'a str>,
}

#[derive(Parser)]
#[grammar = "message.pest"]
struct MessageParser;

impl Message<'_> {
    pub fn parse(text: &str) -> Result<Message<'_>> {
        Self::process(MessageParser::parse(Rule::message, text.as_ref())?)
    }

    fn process(pairs: Pairs<'_, Rule>) -> Result<Message<'_>> {
        let mut prefix = None;
        let mut command = None;
        let mut arguments = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::prefix => prefix = pair.as_str().chars().next(),
                Rule::command => command = Some(pair.as_str()),
                Rule::argument => arguments.push(pair.as_str()),
                Rule::EOI => break,
                _ => unreachable!(),
            }
        }
        Ok(Message {
            prefix: prefix.context("Missing prefix in message")?,
            command: command.context("Missing command in message")?,
            arguments,
        })
    }
}

#[cfg(test)]
mod test_message_parser {
    use super::*;

    #[test]
    fn empty() {
        assert!(Message::parse("").is_err());
    }

    #[test]
    fn no_prefix() {
        assert!(Message::parse("text").is_err());
    }

    #[test]
    fn as_expected() {
        assert_eq!(
            Message::parse(">bot").unwrap(),
            Message {
                prefix: '>',
                command: "bot",
                arguments: Vec::new()
            }
        )
    }

    #[test]
    fn with_arguments() {
        assert_eq!(
            Message::parse(">bot 1 2 3 4 5  a d gba      akj ab1  1kjl12ljk @@@@@@@@@@q   ")
                .unwrap(),
            Message {
                prefix: '>',
                command: "bot",
                arguments: vec![
                    "1",
                    "2",
                    "3",
                    "4",
                    "5",
                    "a",
                    "d",
                    "gba",
                    "akj",
                    "ab1",
                    "1kjl12ljk",
                    "@@@@@@@@@@q"
                ]
            }
        )
    }

    #[test]
    fn chatterino_special_char() {
        assert_eq!(
            Message::parse(">bot\u{E0000}").unwrap(),
            Message {
                prefix: '>',
                command: "bot",
                arguments: vec![]
            }
        );

        assert_eq!(
            Message::parse(">bot\u{E0000}aaaaa").unwrap(),
            Message {
                prefix: '>',
                command: "bot",
                arguments: vec!["aaaaa"]
            }
        );
    }
}
