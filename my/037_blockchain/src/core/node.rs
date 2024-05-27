use url::Url;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(with = "url_serde")]
    pub address: Url,
}