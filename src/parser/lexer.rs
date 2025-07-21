use super::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Syntax,
    Edition,
    Package,
    Import,
    Public,
    Weak,
    Message,
    Enum,
    Service,
    Rpc,
    Returns,
    Stream,
    Optional,
    Required,
    Repeated,
    Oneof,
    Option,
    True,
    False,

    // Identifiers and literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),

    // Symbols
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Equals,
    Dot,

    // End of file
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace_and_comments();

        if self.position >= self.input.len() {
            return Ok(Token::Eof);
        }

        let ch = self.current_char();

        match ch {
            '"' => self.read_string(),
            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '=' => {
                self.advance();
                Ok(Token::Equals)
            }
            '.' => {
                self.advance();
                Ok(Token::Dot)
            }
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ if ch.is_numeric() || ch == '-' => self.read_number(),
            _ => Err(ParseError::UnexpectedToken(ch.to_string())),
        }
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn peek_char(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char();

            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            if ch == '/' {
                if let Some(next_ch) = self.peek_char() {
                    if next_ch == '/' {
                        // Single-line comment
                        self.advance();
                        self.advance();
                        while self.position < self.input.len() && self.current_char() != '\n' {
                            self.advance();
                        }
                        continue;
                    } else if next_ch == '*' {
                        // Multi-line comment
                        self.advance();
                        self.advance();
                        while self.position + 1 < self.input.len() {
                            if self.current_char() == '*' && self.peek_char() == Some('/') {
                                self.advance();
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    }
                }
            }

            break;
        }
    }

    fn read_string(&mut self) -> Result<Token, ParseError> {
        self.advance(); // Skip opening quote
        let mut value = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();

            if ch == '"' {
                self.advance();
                return Ok(Token::StringLiteral(value));
            }

            if ch == '\\' {
                self.advance();
                if self.position >= self.input.len() {
                    return Err(ParseError::UnterminatedString);
                }

                let escaped = match self.current_char() {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    ch => ch,
                };
                value.push(escaped);
                self.advance();
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(ParseError::UnterminatedString)
    }

    fn read_identifier(&mut self) -> Result<Token, ParseError> {
        let mut identifier = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = match identifier.as_str() {
            "syntax" => Token::Syntax,
            "edition" => Token::Edition,
            "package" => Token::Package,
            "import" => Token::Import,
            "public" => Token::Public,
            "weak" => Token::Weak,
            "message" => Token::Message,
            "enum" => Token::Enum,
            "service" => Token::Service,
            "rpc" => Token::Rpc,
            "returns" => Token::Returns,
            "stream" => Token::Stream,
            "optional" => Token::Optional,
            "required" => Token::Required,
            "repeated" => Token::Repeated,
            "oneof" => Token::Oneof,
            "option" => Token::Option,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(identifier),
        };

        Ok(token)
    }

    fn read_number(&mut self) -> Result<Token, ParseError> {
        let mut number = String::new();

        if self.current_char() == '-' {
            number.push('-');
            self.advance();
        }

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_numeric() || ch == '.' {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::NumberLiteral(number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let mut lexer = Lexer::new(r#"syntax = "proto3";"#);

        assert_eq!(lexer.next_token().unwrap(), Token::Syntax);
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::StringLiteral("proto3".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_tokenize_message() {
        let mut lexer = Lexer::new(r#"message Person { string name = 1; }"#);

        assert_eq!(lexer.next_token().unwrap(), Token::Message);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("Person".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBrace);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("string".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("name".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::NumberLiteral("1".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::RightBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_skip_comments() {
        let mut lexer = Lexer::new(
            r#"
// This is a comment
syntax = "proto3";
/* Multi-line
   comment */
message Test {}
"#,
        );

        assert_eq!(lexer.next_token().unwrap(), Token::Syntax);
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::StringLiteral("proto3".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Message);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("Test".to_string())
        );
    }
}
