use rand::Rng;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UserId(pub u128 /* Rsa<Public> */);

impl UserId {
    pub fn generate() -> UserId {
        let mut rng = rand::thread_rng();
        UserId(rng.gen())
    }
}