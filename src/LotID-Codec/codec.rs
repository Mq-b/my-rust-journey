/// 43字符编码表（与 C 原版 s_barcodeTable 完全一致）
const TABLE: [char; 43] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '-', '+',
    '/', '$', '.', '%', ' ',
];

pub fn encode(id: u8, lot: u8) -> [char; 3] {
    let mut val = ((lot as u32) << 8) | (id as u32);
    val ^= 0xE19A;
    let mut out = [' '; 3];
    for c in &mut out {
        let remainder = (val % 43) as usize;
        val /= 43;
        *c = TABLE[remainder];
    }
    out
}

pub fn decode(chars: [char; 3]) -> Option<(u8, u8)> {
    let mut indices = [0u32; 3];
    for (i, ch) in chars.iter().enumerate() {
        let idx = TABLE.iter().position(|&t| t == ch.to_ascii_uppercase())?;
        indices[i] = idx as u32;
    }
    let val = (indices[0] + indices[1] * 43 + indices[2] * 43 * 43) ^ 0xE19A;
    Some(((val & 0xFF) as u8, ((val >> 8) & 0xFF) as u8))
}
