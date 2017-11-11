#[derive(Deserialize, Debug, Clone)]
pub struct GithubUser {
    pub id: u32,
    pub login: String,
}