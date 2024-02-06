pub mod password {
    use sha3::{Digest, Sha3_256};

    pub fn check_pw(input_pw: &str, user_salt: &str, user_pw: &str) -> bool {
        let result = generate_pw(input_pw, user_salt);

        if user_pw == result {
            return true;
        }

        false
    }

    pub fn generate_pw(input_pw: &str, user_salt: &str) -> String {
        let mut hasher = Sha3_256::new();

        hasher.update(input_pw);
        hasher.update(user_salt);
        let result = hasher.finalize();
        let result = format!("{:02x}", result);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_password() {
        // use hex_literal::hex;
        use sha3::{Digest, Sha3_256};
        // create a SHA3-256 object
        let mut hasher = Sha3_256::new();

        // write input message
        hasher.update(b"abc");
        hasher.update(b"salt");
        // hasher.update(b"abcsalt"); 上面2条和下面一条效果一样

        // read hash digest
        let result = hasher.finalize();
        // let char2: Vec<char> = result.iter().map(|b| *b as char).collect::<Vec<_>>();
        // let char2: Vec<char> = result.iter().map(|b| char::from(*b)).collect::<Vec<_>>();
        // let char2: Vec<char> = String::from_utf8_lossy(&result[..]);

        println!("result: {:02x}", result);
    }

    #[test]
    fn test_check_pw() {
        let input_pw = "abc";
        let user_salt = "salt";

        let user_pw = "cb705d51c54f75b070004fc8d630612d586b0a468bdbc9fdf47d9993728cdfed";

        assert_eq!(true, password::check_pw(input_pw, user_salt, user_pw));
    }

    #[test]
    fn test_generate_pw() {
        let input_pw = "abc";
        let user_salt = "salt";

        let user_pw = "cb705d51c54f75b070004fc8d630612d586b0a468bdbc9fdf47d9993728cdfed";

        assert_eq!(user_pw, password::generate_pw(input_pw, user_salt));
    }
}
