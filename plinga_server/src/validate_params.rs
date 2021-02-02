
pub trait ValidateParams{
    fn is_valid(&self, secret_key: &str) -> bool;
}