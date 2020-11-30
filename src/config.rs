use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fs::File, path::Path};
use twitchchat::UserConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config<'a> {
    username: Cow<'a, str>,
    token: Cow<'a, str>,
    channels: Vec<String>,
    database_url: Cow<'a, str>,
}

impl Config<'_> {
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let config = ron::de::from_reader(file)?;

        Ok(config)
    }

    pub fn user_config(&self) -> Result<UserConfig> {
        let user_config = UserConfig::builder()
            .name(self.username.as_ref())
            .token(self.token.as_ref())
            .enable_all_capabilities()
            .build()?;

        Ok(user_config)
    }

    pub fn channels(&self) -> &[String] {
        self.channels.as_slice()
    }

    pub fn add_channel<S>(&mut self, channel: S)
    where
        S: ToString,
    {
        self.channels.push(channel.to_string())
    }

    pub fn database_url(&self) -> &str {
        self.database_url.as_ref()
    }
}
