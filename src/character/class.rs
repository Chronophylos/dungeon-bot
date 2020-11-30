#[derive(sqlx::Type, Debug)]
#[sqlx(rename_all = "lowercase")]
#[sqlx(rename = "class")]
pub enum Class {
    Fighter,
}

impl Default for Class {
    fn default() -> Self {
        Self::Fighter
    }
}
