use crate::character::{CharacterStats, Class, Race};
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;

pub struct Player<'a> {
    // user id
    id: i32,

    pool: &'a PgPool,
}

impl<'a> Player<'a> {
    pub fn new(pool: &'a PgPool, id: i32) -> Self {
        Self { id, pool }
    }

    pub async fn insert(&self) -> Result<()> {
        sqlx::query!(
            r#"
INSERT INTO player
(
    id,
    strength,
    dexterity,
    constitution,
    intelligence,
    wisdom,
    charisma,
    luck,
    has_character,
    race,
    class
)
VALUES (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6,
    $7,
    $8,
    true,
    $9,
    $10
)
            "#,
            self.id,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            Race::default() as Race,
            Class::default() as Class,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn exists(&self) -> Result<bool> {
        let rec = sqlx::query!(
            r#"
SELECT exists(
    SELECT 1
    FROM player
    WHERE id = $1
)
AS "exists"
            "#,
            self.id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(rec.exists.unwrap_or(false))
    }

    pub async fn delete(&self) -> Result<()> {
        sqlx::query!(
            r#"
DELETE FROM player
WHERE id = $1
            "#,
            self.id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn can_enter(&self) -> Result<Option<Duration>> {
        let rec = sqlx::query!(
            r#"
SELECT dungeon_cooldown, has_character
FROM player
WHERE id = $1
            "#,
            self.id
        )
        .fetch_one(self.pool)
        .await?;

        if !rec.has_character {
            return Ok(None);
        }

        if let Some(cooldown) = rec.dungeon_cooldown {
            let now = Utc::now();

            if cooldown < now {
                Ok(None)
            } else {
                Ok(Some(cooldown - now))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_stats(&self) -> Result<CharacterStats> {
        let rec = sqlx::query!(
            r#"
SELECT
    strength as "strength!",
    dexterity as "dexterity!",
    constitution as "constitution!",
    intelligence as "intelligence!",
    wisdom  as "wisdom!",
    charisma as "charisma!",
    luck as "luck!"
FROM player
WHERE id = $1
            "#,
            self.id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(CharacterStats {
            strength: rec.strength,
            dexterity: rec.dexterity,
            constitution: rec.constitution,
            intelligence: rec.intelligence,
            wisdom: rec.wisdom,
            charisma: rec.charisma,
            luck: rec.luck,
        })
    }
}
