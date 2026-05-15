/// QC 6位条码解析结果
#[derive(Debug, Clone, Default)]
pub struct QcBarcode {
    /// 项目ID (第1-2位, 十六进制)
    pub project_id: u8,
    /// 批次号 (第3-5位, 十六进制)
    pub lot_number: u16,
    /// 质控水平 (第6位, 十六进制)
    pub level: u8,
}

/// 解析错误
#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidLength,
    InvalidHex,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "条码长度必须为6位"),
            Self::InvalidHex => write!(f, "包含无效的十六进制字符"),
        }
    }
}

impl QcBarcode {
    /// 解析6位十六进制条码
    ///
    /// 格式: [项目ID 2位][批次号 3位][Level 1位]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let s = input.trim();
        if s.len() != 6 {
            return Err(ParseError::InvalidLength);
        }

        // 验证全部是十六进制字符
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseError::InvalidHex);
        }

        // 项目ID: 第1-2位
        let project_id = u8::from_str_radix(&s[0..2], 16).unwrap();
        // 批次号: 第3-5位
        let lot_number = u16::from_str_radix(&s[2..5], 16).unwrap();
        // Level: 第6位
        let level = u8::from_str_radix(&s[5..6], 16).unwrap();

        Ok(Self {
            project_id,
            lot_number,
            level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let barcode = QcBarcode::parse("1aa293").unwrap();
        assert_eq!(barcode.project_id, 0x1a);
        assert_eq!(barcode.lot_number, 0xa29);
        assert_eq!(barcode.level, 3);
    }

    #[test]
    fn test_parse_uppercase() {
        let barcode = QcBarcode::parse("1AA293").unwrap();
        assert_eq!(barcode.project_id, 0x1a);
    }

    #[test]
    fn test_parse_invalid_length() {
        assert!(QcBarcode::parse("1a293").is_err());
        assert!(QcBarcode::parse("1aa2931").is_err());
    }

    #[test]
    fn test_parse_invalid_hex() {
        assert!(QcBarcode::parse("1ag293").is_err());
    }
}
