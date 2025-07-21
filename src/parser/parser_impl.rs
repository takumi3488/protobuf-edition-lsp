use super::*;
use crate::parser::lexer::{Lexer, Token};
use std::collections::HashMap;

pub fn parse_proto(input: &str) -> Result<ProtoFile> {
    let mut parser = Parser::new(input);
    parser.parse()
}

struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Self {
            lexer,
            current_token,
        }
    }

    fn parse(&mut self) -> Result<ProtoFile> {
        let mut proto_file = ProtoFile {
            syntax: None,
            edition: None,
            statements: Vec::new(),
        };

        while self.current_token != Token::Eof {
            match &self.current_token {
                Token::Syntax => {
                    proto_file.syntax = Some(self.parse_syntax()?);
                }
                Token::Edition => {
                    proto_file.edition = Some(self.parse_edition()?);
                }
                Token::Package => {
                    proto_file
                        .statements
                        .push(Statement::Package(self.parse_package()?));
                }
                Token::Import => {
                    proto_file.statements.push(self.parse_import()?);
                }
                Token::Message => {
                    proto_file
                        .statements
                        .push(Statement::Message(self.parse_message()?));
                }
                Token::Enum => {
                    proto_file
                        .statements
                        .push(Statement::Enum(self.parse_enum()?));
                }
                Token::Service => {
                    proto_file
                        .statements
                        .push(Statement::Service(self.parse_service()?));
                }
                Token::Option => {
                    let (name, value) = self.parse_option()?;
                    proto_file
                        .statements
                        .push(Statement::Option { name, value });
                }
                Token::Semicolon => {
                    self.advance()?;
                }
                _ => {
                    return Err(
                        ParseError::UnexpectedToken(format!("{:?}", self.current_token)).into(),
                    );
                }
            }
        }

        Ok(proto_file)
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.current_token == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(ParseError::Expected {
                expected: format!("{expected:?}"),
                found: format!("{:?}", self.current_token),
            }
            .into())
        }
    }

    fn parse_syntax(&mut self) -> Result<String> {
        self.expect(Token::Syntax)?;
        self.expect(Token::Equals)?;

        let syntax = match &self.current_token {
            Token::StringLiteral(s) => s.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "string literal".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::Semicolon)?;
        Ok(syntax)
    }

    fn parse_edition(&mut self) -> Result<String> {
        self.expect(Token::Edition)?;
        self.expect(Token::Equals)?;

        let edition = match &self.current_token {
            Token::StringLiteral(s) => s.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "string literal".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::Semicolon)?;
        Ok(edition)
    }

    fn parse_package(&mut self) -> Result<String> {
        self.expect(Token::Package)?;

        let mut package_name = String::new();
        loop {
            match &self.current_token {
                Token::Identifier(name) => {
                    package_name.push_str(name);
                    self.advance()?;

                    if self.current_token == Token::Dot {
                        package_name.push('.');
                        self.advance()?;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        self.expect(Token::Semicolon)?;
        Ok(package_name)
    }

    fn parse_import(&mut self) -> Result<Statement> {
        self.expect(Token::Import)?;

        let mut public = false;
        let mut weak = false;

        if self.current_token == Token::Public {
            public = true;
            self.advance()?;
        } else if self.current_token == Token::Weak {
            weak = true;
            self.advance()?;
        }

        let path = match &self.current_token {
            Token::StringLiteral(s) => s.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "string literal".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::Import { path, public, weak })
    }

    fn parse_message(&mut self) -> Result<Message> {
        self.expect(Token::Message)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::LeftBrace)?;

        let mut message = Message {
            name,
            fields: Vec::new(),
            oneofs: Vec::new(),
            nested_messages: Vec::new(),
            nested_enums: Vec::new(),
            options: HashMap::new(),
        };

        while self.current_token != Token::RightBrace {
            match &self.current_token {
                Token::Message => {
                    message.nested_messages.push(self.parse_message()?);
                }
                Token::Enum => {
                    message.nested_enums.push(self.parse_enum()?);
                }
                Token::Oneof => {
                    message.oneofs.push(self.parse_oneof()?);
                }
                Token::Option => {
                    let (name, value) = self.parse_option()?;
                    message.options.insert(name, value);
                }
                Token::Optional | Token::Required | Token::Repeated => {
                    let label = self.parse_field_label()?;
                    let mut field = self.parse_field()?;
                    field.label = Some(label);
                    message.fields.push(field);
                }
                Token::Identifier(_) => {
                    message.fields.push(self.parse_field()?);
                }
                Token::Semicolon => {
                    self.advance()?;
                }
                _ => {
                    return Err(
                        ParseError::UnexpectedToken(format!("{:?}", self.current_token)).into(),
                    );
                }
            }
        }

        self.expect(Token::RightBrace)?;
        Ok(message)
    }

    fn parse_field_label(&mut self) -> Result<FieldLabel> {
        let label = match &self.current_token {
            Token::Optional => FieldLabel::Optional,
            Token::Required => FieldLabel::Required,
            Token::Repeated => FieldLabel::Repeated,
            _ => {
                return Err(ParseError::Expected {
                    expected: "field label".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        Ok(label)
    }

    fn parse_field(&mut self) -> Result<Field> {
        let field_type = match &self.current_token {
            Token::Identifier(t) => t.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "field type".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "field name".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::Equals)?;

        let number = match &self.current_token {
            Token::NumberLiteral(n) => n
                .parse::<u32>()
                .map_err(|_| ParseError::InvalidNumber(n.clone()))?,
            _ => {
                return Err(ParseError::Expected {
                    expected: "field number".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;

        let mut options = HashMap::new();
        if self.current_token == Token::LeftBracket {
            options = self.parse_field_options()?;
        }

        self.expect(Token::Semicolon)?;

        Ok(Field {
            name,
            field_type,
            number,
            label: None,
            options,
        })
    }

    fn parse_field_options(&mut self) -> Result<HashMap<String, OptionValue>> {
        let mut options = HashMap::new();

        self.expect(Token::LeftBracket)?;

        loop {
            let name = self.parse_option_name()?;
            self.expect(Token::Equals)?;
            let value = self.parse_option_value()?;
            options.insert(name, value);

            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        self.expect(Token::RightBracket)?;
        Ok(options)
    }

    fn parse_option_name(&mut self) -> Result<String> {
        let mut name = String::new();

        if self.current_token == Token::LeftParen {
            name.push('(');
            self.advance()?;

            match &self.current_token {
                Token::Identifier(id) => name.push_str(id),
                _ => {
                    return Err(ParseError::Expected {
                        expected: "identifier".to_string(),
                        found: format!("{:?}", self.current_token),
                    }
                    .into())
                }
            }

            self.advance()?;
            self.expect(Token::RightParen)?;
            name.push(')');
        } else {
            match &self.current_token {
                Token::Identifier(id) => {
                    name = id.clone();
                    self.advance()?;
                }
                _ => {
                    return Err(ParseError::Expected {
                        expected: "option name".to_string(),
                        found: format!("{:?}", self.current_token),
                    }
                    .into())
                }
            }
        }

        Ok(name)
    }

    fn parse_option_value(&mut self) -> Result<OptionValue> {
        let value = match &self.current_token {
            Token::StringLiteral(s) => OptionValue::String(s.clone()),
            Token::NumberLiteral(n) => {
                let num = n
                    .parse::<f64>()
                    .map_err(|_| ParseError::InvalidNumber(n.clone()))?;
                OptionValue::Number(num)
            }
            Token::True => OptionValue::Bool(true),
            Token::False => OptionValue::Bool(false),
            Token::Identifier(id) => OptionValue::Identifier(id.clone()),
            _ => {
                return Err(ParseError::Expected {
                    expected: "option value".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        Ok(value)
    }

    fn parse_oneof(&mut self) -> Result<Oneof> {
        self.expect(Token::Oneof)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "oneof name".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::LeftBrace)?;

        let mut fields = Vec::new();

        while self.current_token != Token::RightBrace {
            match &self.current_token {
                Token::Identifier(_) => {
                    fields.push(self.parse_field()?);
                }
                Token::Semicolon => {
                    self.advance()?;
                }
                _ => {
                    return Err(
                        ParseError::UnexpectedToken(format!("{:?}", self.current_token)).into(),
                    );
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Oneof { name, fields })
    }

    fn parse_enum(&mut self) -> Result<Enum> {
        self.expect(Token::Enum)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "enum name".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::LeftBrace)?;

        let mut enum_def = Enum {
            name,
            values: Vec::new(),
            options: HashMap::new(),
        };

        while self.current_token != Token::RightBrace {
            match &self.current_token {
                Token::Option => {
                    let (name, value) = self.parse_option()?;
                    enum_def.options.insert(name, value);
                }
                Token::Identifier(value_name) => {
                    let value_name = value_name.clone();
                    self.advance()?;
                    self.expect(Token::Equals)?;

                    let number = match &self.current_token {
                        Token::NumberLiteral(n) => n
                            .parse::<i32>()
                            .map_err(|_| ParseError::InvalidNumber(n.clone()))?,
                        _ => {
                            return Err(ParseError::Expected {
                                expected: "enum value number".to_string(),
                                found: format!("{:?}", self.current_token),
                            }
                            .into())
                        }
                    };

                    self.advance()?;

                    let mut options = HashMap::new();
                    if self.current_token == Token::LeftBracket {
                        options = self.parse_field_options()?;
                    }

                    self.expect(Token::Semicolon)?;

                    enum_def.values.push(EnumValue {
                        name: value_name,
                        number,
                        options,
                    });
                }
                Token::Semicolon => {
                    self.advance()?;
                }
                _ => {
                    return Err(
                        ParseError::UnexpectedToken(format!("{:?}", self.current_token)).into(),
                    );
                }
            }
        }

        self.expect(Token::RightBrace)?;
        Ok(enum_def)
    }

    fn parse_service(&mut self) -> Result<Service> {
        self.expect(Token::Service)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "service name".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::LeftBrace)?;

        let mut service = Service {
            name,
            methods: Vec::new(),
            options: HashMap::new(),
        };

        while self.current_token != Token::RightBrace {
            match &self.current_token {
                Token::Rpc => {
                    service.methods.push(self.parse_rpc()?);
                }
                Token::Option => {
                    let (name, value) = self.parse_option()?;
                    service.options.insert(name, value);
                }
                Token::Semicolon => {
                    self.advance()?;
                }
                _ => {
                    return Err(
                        ParseError::UnexpectedToken(format!("{:?}", self.current_token)).into(),
                    );
                }
            }
        }

        self.expect(Token::RightBrace)?;
        Ok(service)
    }

    fn parse_rpc(&mut self) -> Result<Method> {
        self.expect(Token::Rpc)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "method name".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::LeftParen)?;

        let mut client_streaming = false;
        if self.current_token == Token::Stream {
            client_streaming = true;
            self.advance()?;
        }

        let request_type = match &self.current_token {
            Token::Identifier(t) => t.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "request type".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::RightParen)?;
        self.expect(Token::Returns)?;
        self.expect(Token::LeftParen)?;

        let mut server_streaming = false;
        if self.current_token == Token::Stream {
            server_streaming = true;
            self.advance()?;
        }

        let response_type = match &self.current_token {
            Token::Identifier(t) => t.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "response type".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::RightParen)?;

        let mut options = HashMap::new();

        if self.current_token == Token::LeftBrace {
            self.advance()?;

            while self.current_token != Token::RightBrace {
                match &self.current_token {
                    Token::Option => {
                        let (name, value) = self.parse_option()?;
                        options.insert(name, value);
                    }
                    Token::Semicolon => {
                        self.advance()?;
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken(format!(
                            "{:?}",
                            self.current_token
                        ))
                        .into());
                    }
                }
            }

            self.expect(Token::RightBrace)?;
        } else {
            self.expect(Token::Semicolon)?;
        }

        Ok(Method {
            name,
            request_type,
            response_type,
            client_streaming,
            server_streaming,
            options,
        })
    }

    fn parse_option(&mut self) -> Result<(String, OptionValue)> {
        self.expect(Token::Option)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::Expected {
                    expected: "option name".to_string(),
                    found: format!("{:?}", self.current_token),
                }
                .into())
            }
        };

        self.advance()?;
        self.expect(Token::Equals)?;

        let value = self.parse_option_value()?;
        self.expect(Token::Semicolon)?;

        Ok((name, value))
    }
}
