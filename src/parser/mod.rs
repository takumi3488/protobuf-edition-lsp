use anyhow::Result;
use std::collections::HashMap;
use thiserror::Error;

mod lexer;
mod parser_impl;
mod validator;

pub use parser_impl::parse_proto;
pub use validator::{validate_proto, ValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoFile {
    pub syntax: Option<String>,
    pub edition: Option<String>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Package(String),
    Import {
        path: String,
        public: bool,
        weak: bool,
    },
    Message(Message),
    Enum(Enum),
    Service(Service),
    Option {
        name: String,
        value: OptionValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub oneofs: Vec<Oneof>,
    pub nested_messages: Vec<Message>,
    pub nested_enums: Vec<Enum>,
    pub options: HashMap<String, OptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub number: u32,
    pub label: Option<FieldLabel>,
    pub options: HashMap<String, OptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldLabel {
    Optional,
    Required,
    Repeated,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Oneof {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub values: Vec<EnumValue>,
    pub options: HashMap<String, OptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: String,
    pub number: i32,
    pub options: HashMap<String, OptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Service {
    pub name: String,
    pub methods: Vec<Method>,
    pub options: HashMap<String, OptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub request_type: String,
    pub response_type: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub options: HashMap<String, OptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    String(String),
    Number(f64),
    Bool(bool),
    Identifier(String),
}

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),

    #[error("Expected {expected}, found {found}")]
    Expected { expected: String, found: String },

    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),

    #[error("Unterminated string")]
    UnterminatedString,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("End of file reached unexpectedly")]
    UnexpectedEof,
}
