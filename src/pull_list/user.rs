#[derive(Deserialize, Debug)]
pub struct User {
    pub id: u32,
    pub login: String,
}