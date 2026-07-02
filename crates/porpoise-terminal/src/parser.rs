#[derive(Debug, Clone, PartialEq)]
pub enum ParsedOutput {
    Text(String),
    OSequence(String),
    Bell,
    CarriageReturn,
    LineFeed,
    ClearScreen,
    CursorMove { row: u16, col: u16 },
    ColorChange { fg: Option<String>, bg: Option<String> },
    Unknown(Vec<u8>),
}

pub struct OutputParser;

impl OutputParser {
    pub fn parse(data: &[u8]) -> Vec<ParsedOutput> {
        let mut results = Vec::new();
        let mut i = 0;
        while i < data.len() {
            if data[i] == 0x1b {
                if i + 1 < data.len() && data[i + 1] == b'['
                    && let Some((parsed, consumed)) = Self::parse_csi(&data[i..]) {
                        results.push(parsed);
                        i += consumed;
                        continue;
                    }
                if i + 1 < data.len() && data[i + 1] == b']'
                    && let Some((parsed, consumed)) = Self::parse_osc(&data[i..]) {
                        results.push(parsed);
                        i += consumed;
                        continue;
                    }
                results.push(ParsedOutput::Unknown(vec![data[i]]));
                i += 1;
            } else if data[i] == 0x07 {
                results.push(ParsedOutput::Bell);
                i += 1;
            } else if data[i] == b'\r' {
                results.push(ParsedOutput::CarriageReturn);
                i += 1;
            } else if data[i] == b'\n' {
                results.push(ParsedOutput::LineFeed);
                i += 1;
            } else if data[i] == 0x0c {
                results.push(ParsedOutput::ClearScreen);
                i += 1;
            } else {
                let start = i;
                while i < data.len() && data[i] >= 0x20 && data[i] <= 0x7e {
                    i += 1;
                }
                if i > start {
                    results.push(ParsedOutput::Text(
                        String::from_utf8_lossy(&data[start..i]).to_string()
                    ));
                } else {
                    results.push(ParsedOutput::Unknown(vec![data[i]]));
                    i += 1;
                }
            }
        }
        results
    }

    fn parse_csi(data: &[u8]) -> Option<(ParsedOutput, usize)> {
        if data.len() < 3 || data[0] != 0x1b || data[1] != b'[' {
            return None;
        }
        let _end = 2;
        for i in 2..data.len() {
            let b = data[i];
            if (0x40..=0x7e).contains(&b) {
                let params = std::str::from_utf8(&data[2..i]).ok()?;
                match b {
                    b'J' if params.is_empty() || params == "2" => {
                        return Some((ParsedOutput::ClearScreen, i + 1));
                    }
                    b'H' => {
                        let parts: Vec<&str> = params.split(';').collect();
                        let row = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
                        let col = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                        return Some((ParsedOutput::CursorMove { row, col }, i + 1));
                    }
                    b'm' => {
                        return Some((ParsedOutput::ColorChange { fg: None, bg: None }, i + 1));
                    }
                    _ => {}
                }
                return Some((ParsedOutput::Unknown(data[..=i].to_vec()), i + 1));
            }
        }
        None
    }

    fn parse_osc(data: &[u8]) -> Option<(ParsedOutput, usize)> {
        if data.len() < 4 || data[0] != 0x1b || data[1] != b']' {
            return None;
        }
        for i in 2..data.len() {
            if data[i] == 0x07 || (i + 1 < data.len() && data[i] == 0x1b && data[i + 1] == b'\\') {
                let end = i;
                let content = String::from_utf8_lossy(&data[2..end]).to_string();
                let consumed = if data[i] == 0x07 { i + 1 } else { i + 2 };
                return Some((ParsedOutput::OSequence(content), consumed));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let result = OutputParser::parse(b"hello world");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ParsedOutput::Text("hello world".to_string()));
    }

    #[test]
    fn test_parse_newline() {
        let result = OutputParser::parse(b"hello\nworld");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ParsedOutput::Text("hello".to_string()));
        assert_eq!(result[1], ParsedOutput::LineFeed);
        assert_eq!(result[2], ParsedOutput::Text("world".to_string()));
    }
}
