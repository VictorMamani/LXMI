use std::path::PathBuf;

const MAX_VDF_BYTES: usize = 2 * 1024 * 1024;
const MAX_NESTING_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdfParseError {
    pub offset: usize,
    pub message: &'static str,
}

impl std::fmt::Display for VdfParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for VdfParseError {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum KeyValuesValue {
    Scalar(String),
    Object(Vec<(String, KeyValuesValue)>),
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Scalar(String),
    OpenBrace,
    CloseBrace,
    End,
}

struct Lexer<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            offset: 0,
        }
    }

    fn next(&mut self) -> Result<Token, VdfParseError> {
        self.skip_whitespace_and_comments();
        match self.bytes.get(self.offset).copied() {
            None => Ok(Token::End),
            Some(b'{') => {
                self.offset += 1;
                Ok(Token::OpenBrace)
            }
            Some(b'}') => {
                self.offset += 1;
                Ok(Token::CloseBrace)
            }
            Some(b'"') => self.read_quoted(),
            Some(_) => self.read_unquoted(),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self
                .bytes
                .get(self.offset)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.offset += 1;
            }

            if self.bytes.get(self.offset..self.offset + 2) == Some(b"//") {
                while let Some(byte) = self.bytes.get(self.offset) {
                    self.offset += 1;
                    if *byte == b'\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn read_quoted(&mut self) -> Result<Token, VdfParseError> {
        let start = self.offset;
        self.offset += 1;
        let mut output = Vec::new();

        while let Some(byte) = self.bytes.get(self.offset).copied() {
            self.offset += 1;
            match byte {
                b'"' => {
                    let value = String::from_utf8(output).map_err(|_| VdfParseError {
                        offset: start,
                        message: "quoted value is not valid UTF-8",
                    })?;
                    return Ok(Token::Scalar(value));
                }
                b'\\' => {
                    let escaped = self.bytes.get(self.offset).copied().ok_or(VdfParseError {
                        offset: self.offset,
                        message: "unterminated escape sequence",
                    })?;
                    self.offset += 1;
                    output.push(match escaped {
                        b'"' => b'"',
                        b'\\' => b'\\',
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b't' => b'\t',
                        other => other,
                    });
                }
                other => output.push(other),
            }
        }

        Err(VdfParseError {
            offset: start,
            message: "unterminated quoted value",
        })
    }

    fn read_unquoted(&mut self) -> Result<Token, VdfParseError> {
        let start = self.offset;
        while let Some(byte) = self.bytes.get(self.offset).copied() {
            if byte.is_ascii_whitespace() || matches!(byte, b'{' | b'}') {
                break;
            }
            self.offset += 1;
        }

        if self.offset == start {
            return Err(VdfParseError {
                offset: start,
                message: "expected a key or value",
            });
        }

        let value =
            std::str::from_utf8(&self.bytes[start..self.offset]).map_err(|_| VdfParseError {
                offset: start,
                message: "unquoted value is not valid UTF-8",
            })?;
        Ok(Token::Scalar(value.to_owned()))
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Token,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self, VdfParseError> {
        let mut lexer = Lexer::new(input);
        let lookahead = lexer.next()?;
        Ok(Self {
            lexer,
            lookahead,
            offset: 0,
        })
    }

    fn parse(mut self) -> Result<Vec<(String, KeyValuesValue)>, VdfParseError> {
        self.parse_entries(false, 0)
    }

    fn parse_entries(
        &mut self,
        nested: bool,
        depth: usize,
    ) -> Result<Vec<(String, KeyValuesValue)>, VdfParseError> {
        if depth > MAX_NESTING_DEPTH {
            return Err(VdfParseError {
                offset: self.offset,
                message: "object nesting limit exceeded",
            });
        }
        let mut entries = Vec::new();

        loop {
            match self.lookahead {
                Token::End if nested => {
                    return Err(VdfParseError {
                        offset: self.offset,
                        message: "unclosed object",
                    });
                }
                Token::End => return Ok(entries),
                Token::CloseBrace if nested => {
                    self.advance()?;
                    return Ok(entries);
                }
                Token::CloseBrace => {
                    return Err(VdfParseError {
                        offset: self.offset,
                        message: "unexpected closing brace",
                    });
                }
                Token::OpenBrace => {
                    return Err(VdfParseError {
                        offset: self.offset,
                        message: "expected a key before object",
                    });
                }
                Token::Scalar(_) => {}
            }

            let key = match std::mem::replace(&mut self.lookahead, Token::End) {
                Token::Scalar(value) => value,
                _ => {
                    return Err(VdfParseError {
                        offset: self.offset,
                        message: "expected a key",
                    });
                }
            };
            self.advance()?;

            let value = match self.lookahead {
                Token::OpenBrace => {
                    self.advance()?;
                    KeyValuesValue::Object(self.parse_entries(true, depth + 1)?)
                }
                Token::Scalar(_) => match std::mem::replace(&mut self.lookahead, Token::End) {
                    Token::Scalar(value) => {
                        self.advance()?;
                        KeyValuesValue::Scalar(value)
                    }
                    _ => {
                        return Err(VdfParseError {
                            offset: self.offset,
                            message: "expected a value",
                        });
                    }
                },
                Token::End | Token::CloseBrace => {
                    return Err(VdfParseError {
                        offset: self.offset,
                        message: "missing value",
                    });
                }
            };
            entries.push((key, value));
        }
    }

    fn advance(&mut self) -> Result<(), VdfParseError> {
        self.offset = self.lexer.offset;
        self.lookahead = self.lexer.next()?;
        Ok(())
    }
}

pub(crate) fn parse_document(input: &str) -> Result<Vec<(String, KeyValuesValue)>, VdfParseError> {
    if input.len() > MAX_VDF_BYTES {
        return Err(VdfParseError {
            offset: MAX_VDF_BYTES,
            message: "KeyValues document exceeds the 2 MiB size limit",
        });
    }

    Parser::new(input)?.parse()
}

pub fn parse_library_folders(input: &str) -> Result<Vec<PathBuf>, VdfParseError> {
    let entries = parse_document(input)?;
    let library_folders = entries
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("libraryfolders"))
        .and_then(|(_, value)| match value {
            KeyValuesValue::Object(entries) => Some(entries),
            KeyValuesValue::Scalar(_) => None,
        })
        .ok_or(VdfParseError {
            offset: 0,
            message: "libraryfolders object is missing",
        })?;

    let mut paths = Vec::new();
    for (key, value) in library_folders {
        if !key.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }

        let path = match value {
            KeyValuesValue::Scalar(value) => value,
            KeyValuesValue::Object(fields) => fields
                .iter()
                .find(|(field, _)| field.eq_ignore_ascii_case("path"))
                .and_then(|(_, value)| match value {
                    KeyValuesValue::Scalar(value) => Some(value),
                    KeyValuesValue::Object(_) => None,
                })
                .ok_or(VdfParseError {
                    offset: 0,
                    message: "numeric library entry has no path value",
                })?,
        };

        if path.trim().is_empty() {
            return Err(VdfParseError {
                offset: 0,
                message: "library path is empty",
            });
        }
        paths.push(PathBuf::from(path));
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::{parse_library_folders, MAX_NESTING_DEPTH};
    use std::path::PathBuf;

    #[test]
    fn parses_modern_and_legacy_library_entries() {
        let input = r#"
            // The default library uses the new object format.
            "libraryfolders"
            {
                "0"
                {
                    "path" "/home/tester/.local/share/Steam"
                    "label" ""
                }
                "1" "/mnt/games/SteamLibrary"
                "contentstatsid" "123"
            }
        "#;

        let paths = parse_library_folders(input).expect("fixture should parse");
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/home/tester/.local/share/Steam"),
                PathBuf::from("/mnt/games/SteamLibrary"),
            ]
        );
    }

    #[test]
    fn parses_escaped_quotes_and_backslashes() {
        let input = r#""libraryfolders" { "0" "C:\\Games\\Steam \"Library\"" }"#;
        let paths = parse_library_folders(input).expect("escaped values should parse");
        assert_eq!(paths, vec![PathBuf::from("C:\\Games\\Steam \"Library\"")]);
    }

    #[test]
    fn reports_malformed_or_missing_configuration() {
        assert!(parse_library_folders(r#""libraryfolders" { "0" {"#).is_err());
        assert!(parse_library_folders(r#""other" { }"#).is_err());
    }

    #[test]
    fn rejects_empty_library_paths() {
        assert!(parse_library_folders(r#""libraryfolders" { "0" "" }"#).is_err());
    }

    #[test]
    fn rejects_excessively_nested_input() {
        let mut input = String::from(r#""libraryfolders" "#);
        for _ in 0..(MAX_NESTING_DEPTH + 2) {
            input.push_str(r#""x" {"#);
        }

        assert!(parse_library_folders(&input).is_err());
    }
}
