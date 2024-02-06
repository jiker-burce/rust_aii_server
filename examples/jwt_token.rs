use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ApiKey {
    uid: i64,
    nonce: String,
}
fn main() {
    println!("Hello, world!");

    let content = serde_json::to_vec(&ApiKey {
        uid: 13i64,
        nonce: "hello world".to_string(),
    })
    .unwrap();

    let (_, token) = rc_token::create_token_pair("123456", content, 60 * 5, 60).unwrap();

    println!("token: {}", token);

    let raw_token = "eyJhbGciOiJIUzI1NiJ9.eyJkIjpbMTIzLDM0LDExNywxMDUsMTAwLDM0LDU4LDQ5LDUxLDQ0LDM0LDExMCwxMTEsMTEwLDk5LDEwMSwzNCw1OCwzNCwxMDQsMTAxLDEwOCwxMDgsMTExLDMyLDExOSwxMTEsMTE0LDEwOCwxMDAsMzQsMTI1XSwiZSI6IjIwMjMtMDQtMjZUMDE6MjE6NDAuNjA1ODc0WiIsIm4iOiJPT3NjSkdoOFNHUUFBQUFBIiwidCI6IkFjY2Vzc1Rva2VuIn0.iNQC_CFx-TpQJ4_-vA1wMPf_N0E0HyiDHX5v_2tvPRc";

    // 因为上面的content是Vec<u8>,这里反序列化也采用相同的类型
    let (token_type, data) = rc_token::parse_token::<Vec<u8>>("123456", raw_token, false).unwrap();
    println!("token_type: {:?}", token_type);
    println!("data: {:?}", std::str::from_utf8(&data));

    let key = serde_json::from_slice::<ApiKey>(&data).ok();
    println!("key: {:?}", key.unwrap());
}
