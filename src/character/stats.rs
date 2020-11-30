const BASE_DAMAGE: f32 = 1.0;
const BASE_HEALTH: f32 = 10.0;
const DEXTERITY_MODIFIER: f32 = 0.03;
const CONSTITUTION_MODIFIER: f32 = 1.0;

#[derive(Debug, Copy, Clone)]
pub struct CharacterStats {
    pub strength: i16,
    pub dexterity: i16,
    pub constitution: i16,
    pub intelligence: i16,
    pub wisdom: i16,
    pub charisma: i16,
    pub luck: i16,
}

impl CharacterStats {
    pub fn dps(&self) -> f32 {
        let speed = Into::<f32>::into(self.dexterity) * DEXTERITY_MODIFIER;

        BASE_DAMAGE + Into::<f32>::into(self.strength) * speed
    }

    pub fn health(&self) -> f32 {
        BASE_HEALTH + Into::<f32>::into(self.constitution) * CONSTITUTION_MODIFIER
    }
}
