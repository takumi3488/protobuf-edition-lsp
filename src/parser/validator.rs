use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ValidationError {
    fn new(message: String) -> Self {
        Self {
            message,
            line: 0,
            column: 0,
        }
    }
}

pub fn validate_proto(proto_file: &ProtoFile) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut validator = Validator::new();

    validator.validate_proto_file(proto_file, &mut errors);
    errors
}

struct Validator {
    used_field_numbers: HashMap<String, HashSet<u32>>,
    used_enum_values: HashMap<String, HashSet<i32>>,
}

impl Validator {
    fn new() -> Self {
        Self {
            used_field_numbers: HashMap::new(),
            used_enum_values: HashMap::new(),
        }
    }

    fn validate_proto_file(&mut self, proto_file: &ProtoFile, errors: &mut Vec<ValidationError>) {
        // Validate edition if present
        if let Some(edition) = &proto_file.edition {
            if edition != "2023" {
                errors.push(ValidationError::new(format!(
                    "Unsupported edition '{edition}'. Only edition 2023 is supported."
                )));
            }
        }

        // Validate syntax if present
        if let Some(syntax) = &proto_file.syntax {
            if syntax != "proto2" && syntax != "proto3" {
                errors.push(ValidationError::new(format!(
                    "Invalid syntax '{syntax}'. Must be 'proto2' or 'proto3'."
                )));
            }
        }

        // Validate statements
        for statement in &proto_file.statements {
            self.validate_statement(statement, errors);
        }
    }

    fn validate_statement(&mut self, statement: &Statement, errors: &mut Vec<ValidationError>) {
        match statement {
            Statement::Message(message) => {
                self.validate_message(message, errors);
            }
            Statement::Enum(enum_def) => {
                self.validate_enum(enum_def, errors);
            }
            Statement::Service(service) => {
                self.validate_service(service, errors);
            }
            _ => {}
        }
    }

    fn validate_message(&mut self, message: &Message, errors: &mut Vec<ValidationError>) {
        let message_key = message.name.clone();

        // Check for duplicate field numbers
        let field_numbers = self
            .used_field_numbers
            .entry(message_key.clone())
            .or_default();

        for field in &message.fields {
            if !field_numbers.insert(field.number) {
                errors.push(ValidationError::new(format!(
                    "Duplicate field number {} in message '{}'",
                    field.number, message.name
                )));
            }

            // Validate field number range
            if field.number == 0 {
                errors.push(ValidationError::new(format!(
                    "Field number cannot be 0 in field '{}' of message '{}'",
                    field.name, message.name
                )));
            }

            if field.number >= 19000 && field.number <= 19999 {
                errors.push(ValidationError::new(
                    format!("Field number {} is reserved for protocol buffer implementation in field '{}' of message '{}'", 
                            field.number, field.name, message.name)
                ));
            }
        }

        // Validate oneof fields
        for oneof in &message.oneofs {
            for field in &oneof.fields {
                if !field_numbers.insert(field.number) {
                    errors.push(ValidationError::new(format!(
                        "Duplicate field number {} in oneof '{}' of message '{}'",
                        field.number, oneof.name, message.name
                    )));
                }
            }
        }

        // Validate nested messages
        for nested in &message.nested_messages {
            self.validate_message(nested, errors);
        }

        // Validate nested enums
        for nested in &message.nested_enums {
            self.validate_enum(nested, errors);
        }
    }

    fn validate_enum(&mut self, enum_def: &Enum, errors: &mut Vec<ValidationError>) {
        let enum_key = enum_def.name.clone();
        let enum_values = self.used_enum_values.entry(enum_key).or_default();

        let mut has_zero = false;

        for value in &enum_def.values {
            if !enum_values.insert(value.number) {
                errors.push(ValidationError::new(format!(
                    "Duplicate enum value {} in enum '{}'",
                    value.number, enum_def.name
                )));
            }

            if value.number == 0 {
                has_zero = true;
            }
        }

        // In proto3, enums must have a zero value
        if !has_zero && !enum_def.values.is_empty() {
            errors.push(ValidationError::new(format!(
                "Enum '{}' must have a zero value",
                enum_def.name
            )));
        }
    }

    fn validate_service(&mut self, service: &Service, errors: &mut Vec<ValidationError>) {
        let mut method_names = HashSet::new();

        for method in &service.methods {
            if !method_names.insert(&method.name) {
                errors.push(ValidationError::new(format!(
                    "Duplicate method name '{}' in service '{}'",
                    method.name, service.name
                )));
            }
        }
    }
}
