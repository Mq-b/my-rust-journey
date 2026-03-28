const EXTENDED_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.-";

fn char_to_index(c: u8) -> Option<usize> {
    EXTENDED_CHARSET.iter().position(|&b| b == c)
}

fn index_to_char(x: usize) -> u8 {
    EXTENDED_CHARSET[x % EXTENDED_CHARSET.len()]
}

pub fn encrypt(input: &str, key: &str) -> Result<String, String> {
    if key.is_empty() {
        return Err("Key cannot be empty".into());
    }

    let key_bytes = key.as_bytes();
    let mut encrypted = Vec::with_capacity(input.len());

    for (i, &c) in input.as_bytes().iter().enumerate() {
        let Some(c_idx) = char_to_index(c) else {
            continue;
        };
        let k_idx =
            char_to_index(key_bytes[i % key_bytes.len()]).ok_or("Invalid character in key")?;
        let value = (c_idx ^ k_idx) % EXTENDED_CHARSET.len();
        encrypted.push(index_to_char(value));
    }

    String::from_utf8(encrypted).map_err(|e| e.to_string())
}

pub fn decrypt(encrypted: &str, key: &str) -> Result<String, String> {
    if key.is_empty() {
        return Err("Key cannot be empty".into());
    }

    let key_bytes = key.as_bytes();
    let mut decrypted = Vec::with_capacity(encrypted.len());

    for (i, &c) in encrypted.as_bytes().iter().enumerate() {
        let Some(c_idx) = char_to_index(c) else {
            continue;
        };
        let k_idx =
            char_to_index(key_bytes[i % key_bytes.len()]).ok_or("Invalid character in key")?;
        let value = (c_idx ^ k_idx) % EXTENDED_CHARSET.len();
        decrypted.push(index_to_char(value));
    }

    String::from_utf8(decrypted).map_err(|e| e.to_string())
}

pub fn encrypt_lines(input: &str, key: &str) -> Result<String, String> {
    input
        .lines()
        .map(|line| encrypt(line, key))
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.join("\n"))
}

pub fn decrypt_lines(input: &str, key: &str) -> Result<String, String> {
    input
        .lines()
        .map(|line| decrypt(line, key))
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.join("\n"))
}
