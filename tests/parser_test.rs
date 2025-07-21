use protobuf_edition_lsp::parser::*;

#[cfg(test)]
mod parser_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_empty_file() {
        let content = "";
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.syntax, None);
        assert_eq!(parsed.edition, None);
        assert!(parsed.statements.is_empty());
    }

    #[test]
    fn test_parse_edition_2023() {
        let content = r#"edition = "2023";"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.edition, Some("2023".to_string()));
    }

    #[test]
    fn test_parse_syntax_proto3() {
        let content = r#"syntax = "proto3";"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.syntax, Some("proto3".to_string()));
    }

    #[test]
    fn test_parse_package_declaration() {
        let content = r#"package com.example.myapp;"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Statement::Package(pkg) if pkg == "com.example.myapp")));
    }

    #[test]
    fn test_parse_import_statement() {
        let content = r#"import "google/protobuf/timestamp.proto";"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.statements.iter().any(|stmt| matches!(stmt, Statement::Import { path, .. } if path == "google/protobuf/timestamp.proto")));
    }

    #[test]
    fn test_parse_message_with_fields() {
        let content = r#"
message Person {
  string name = 1;
  int32 id = 2;
  repeated string email = 3;
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let message = parsed.statements.iter().find_map(|stmt| {
            if let Statement::Message(msg) = stmt {
                Some(msg)
            } else {
                None
            }
        });

        assert!(message.is_some());
        let message = message.unwrap();
        assert_eq!(message.name, "Person");
        assert_eq!(message.fields.len(), 3);

        assert_eq!(message.fields[0].name, "name");
        assert_eq!(message.fields[0].field_type, "string");
        assert_eq!(message.fields[0].number, 1);
        assert_eq!(message.fields[0].label, None);

        assert_eq!(message.fields[1].name, "id");
        assert_eq!(message.fields[1].field_type, "int32");
        assert_eq!(message.fields[1].number, 2);

        assert_eq!(message.fields[2].name, "email");
        assert_eq!(message.fields[2].label, Some(FieldLabel::Repeated));
    }

    #[test]
    fn test_parse_enum() {
        let content = r#"
enum Status {
  UNKNOWN = 0;
  ACTIVE = 1;
  INACTIVE = 2;
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let enum_def = parsed.statements.iter().find_map(|stmt| {
            if let Statement::Enum(e) = stmt {
                Some(e)
            } else {
                None
            }
        });

        assert!(enum_def.is_some());
        let enum_def = enum_def.unwrap();
        assert_eq!(enum_def.name, "Status");
        assert_eq!(enum_def.values.len(), 3);
        assert_eq!(enum_def.values[0].name, "UNKNOWN");
        assert_eq!(enum_def.values[0].number, 0);
    }

    #[test]
    fn test_parse_service_with_rpc() {
        let content = r#"
service Greeter {
  rpc SayHello (HelloRequest) returns (HelloResponse);
  rpc SayGoodbye (GoodbyeRequest) returns (GoodbyeResponse) {}
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let service = parsed.statements.iter().find_map(|stmt| {
            if let Statement::Service(svc) = stmt {
                Some(svc)
            } else {
                None
            }
        });

        assert!(service.is_some());
        let service = service.unwrap();
        assert_eq!(service.name, "Greeter");
        assert_eq!(service.methods.len(), 2);

        assert_eq!(service.methods[0].name, "SayHello");
        assert_eq!(service.methods[0].request_type, "HelloRequest");
        assert_eq!(service.methods[0].response_type, "HelloResponse");
    }

    #[test]
    fn test_parse_oneof() {
        let content = r#"
message TestMessage {
  oneof test_oneof {
    string name = 1;
    int32 sub_number = 2;
  }
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let message = parsed.statements.iter().find_map(|stmt| {
            if let Statement::Message(msg) = stmt {
                Some(msg)
            } else {
                None
            }
        });

        assert!(message.is_some());
        let message = message.unwrap();
        assert_eq!(message.oneofs.len(), 1);
        assert_eq!(message.oneofs[0].name, "test_oneof");
        assert_eq!(message.oneofs[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_field_options() {
        let content = r#"
message TestMessage {
  string name = 1 [deprecated = true];
  int32 id = 2 [(custom_option) = "value"];
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let message = parsed.statements.iter().find_map(|stmt| {
            if let Statement::Message(msg) = stmt {
                Some(msg)
            } else {
                None
            }
        });

        assert!(message.is_some());
        let message = message.unwrap();
        assert!(message.fields[0].options.contains_key("deprecated"));
        assert!(message.fields[1].options.contains_key("(custom_option)"));
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
// This is a comment
syntax = "proto3";

/* Multi-line
   comment */
message Test {
  // Field comment
  string field = 1;
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_messages() {
        let content = r#"
message Outer {
  message Inner {
    string value = 1;
  }
  Inner inner_field = 1;
}
"#;
        let result = parse_proto(content);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let outer = parsed.statements.iter().find_map(|stmt| {
            if let Statement::Message(msg) = stmt {
                Some(msg)
            } else {
                None
            }
        });

        assert!(outer.is_some());
        let outer = outer.unwrap();
        assert_eq!(outer.name, "Outer");
        assert_eq!(outer.nested_messages.len(), 1);
        assert_eq!(outer.nested_messages[0].name, "Inner");
    }

    #[test]
    fn test_error_invalid_syntax() {
        let content = r#"syntax = invalid;"#;
        let result = parse_proto(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_duplicate_field_number() {
        let content = r#"
message Test {
  string field1 = 1;
  int32 field2 = 1;
}
"#;
        let result = parse_proto(content);
        let parsed = result.unwrap();
        let errors = validate_proto(&parsed);
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .any(|e| e.message.to_lowercase().contains("duplicate field number")));
    }
}
