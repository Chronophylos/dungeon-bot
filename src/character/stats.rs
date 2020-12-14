use std::ops::{Deref, DerefMut};

use crate::{Dice, D20, D6};

const BASE_DAMAGE: f32 = 0.0;
const BASE_HEALTH: f32 = 10.0;
const BASE_SPEED: f32 = 1.0;
const DEXTERITY_MODIFIER: f32 = 0.01;
const HEALTH_MODIFIER: f32 = 2.0;

// 0 is equivalent to DnD's 10
#[derive(Debug, Copy, Clone, Default)]
pub struct Attribute(i16);

impl Deref for Attribute {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attribute {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<i16> for Attribute {
    fn from(v: i16) -> Self {
        Self(v)
    }
}

impl Into<i16> for Attribute {
    fn into(self) -> i16 {
        self.0
    }
}

impl Attribute {
    pub fn modifier(&self) -> i16 {
        todo!()
    }

    pub fn as_f32(&self) -> f32 {
        self.0.into()
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct CharacterStats {
    pub strength: Attribute,
    pub dexterity: Attribute,
    pub constitution: Attribute,
    pub intelligence: Attribute,
    pub wisdom: Attribute,
    pub charisma: Attribute,
    pub luck: Attribute,
}

impl CharacterStats {
    pub fn primary_attribute(&self) -> Attribute {
        self.strength
    }

    pub fn dps(&self) -> f32 {
        // base + dex * dex mod
        let attacks_per_second = BASE_SPEED + self.dexterity.as_f32() * DEXTERITY_MODIFIER;

        // base + 1D6 + primary attribute modifier
        let damage_per_attack =
            BASE_DAMAGE + D6::avg() + f32::from(self.primary_attribute().modifier());

        damage_per_attack * attacks_per_second
    }

    pub fn max_health(&self) -> f32 {
        // ( base + constitution modifier ) * health modifier
        (BASE_HEALTH + f32::from(self.constitution.modifier())) * HEALTH_MODIFIER
    }

    pub fn roll_attack(&self) -> i16 {
        // 1D20 + primary attribute modifier + weapon level
        D20::roll_once() + self.primary_attribute().modifier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dps() {
        assert_eq!(
            CharacterStats {
                strength: 1.into(),
                dexterity: 1.into(),
                ..CharacterStats::default()
            }
            .dps(),
            0.5
        );
    }

    #[test]
    fn health() {
        assert_eq!(
            CharacterStats {
                constitution: 1.into(),
                ..CharacterStats::default()
            }
            .max_health(),
            20.0
        );
    }
}
