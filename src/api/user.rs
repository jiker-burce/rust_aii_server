use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;

/// User info
#[derive(Debug, Object, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserInfo {
    #[oai(read_only)]
    pub id: u64,
    pub email: Option<String>,
    pub name: String,
    #[oai(skip)]
    pub age: i32,
    #[oai(skip)]
    pub password: String,
    #[oai(skip)]
    pub salt: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl UserInfo {
    pub fn set_password(&mut self, password: String) {
        let user_pw = rc_utilities::password::generate_pw(&password, &self.salt);
        self.password = user_pw;
    }

    pub fn check_pw(&self, input_pw: &str) -> bool {
        rc_utilities::password::check_pw(input_pw, &self.salt, &self.password)
    }
}
