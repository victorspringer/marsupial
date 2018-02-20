extern crate bson;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    #[serde(with = "bson::compat::u2f")]
    pub version: u64,
    pub user: String,
    pub created_at: String,
    pub code: String,
    pub language: String,
    pub path: String,
    pub region: String,
    pub aws_key: String,
    pub aws_secret: String
}
