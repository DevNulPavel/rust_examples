use hand_made_hmac::hmac_sha256;

fn main() {
    let msg = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    let key = "secret key";
    let hash = hmac_sha256(msg.as_bytes(), key.as_bytes());
    let hash_str = hash.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join("");
    println!("HMAC SHA256: {}", hash_str);
}
