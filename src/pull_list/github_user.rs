#[derive(Deserialize, Debug)]
pub struct GithubUser {
    pub id: u32,
    pub login: String,
}