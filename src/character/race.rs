#[derive(sqlx::Type, Debug)]
#[sqlx(rename_all = "lowercase")]
#[sqlx(rename = "race")]
pub enum Race {
    Human,
}

impl Default for Race {
    fn default() -> Self {
        Self::Human
    }
}
