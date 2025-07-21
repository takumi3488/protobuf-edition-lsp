use protobuf_edition_lsp::lsp_server::*;
use protobuf_edition_lsp::parser::{parse_proto, validate_proto};
use tower_lsp::lsp_types::*;

#[cfg(test)]
mod lsp_server_tests {
    use super::*;

    #[test]
    fn test_parser_integration() {
        // Test that parser works correctly with duplicate field numbers
        let content = r#"
message Test {
  string field1 = 1;
  int32 field2 = 1;
}
"#;

        let parsed = parse_proto(content).unwrap();
        let errors = validate_proto(&parsed);

        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .any(|e| e.message.to_lowercase().contains("duplicate field number")));
    }

    #[tokio::test]
    async fn test_lsp_server_creation() {
        use tower_lsp::{LanguageServer, LspService};

        // Test that we can create the LSP server
        let (service, _socket) = LspService::new(ProtobufLanguageServer::new);

        // Test initialize
        let init_params = InitializeParams::default();
        let result = service.inner().initialize(init_params).await.unwrap();

        // Check capabilities
        assert!(result.capabilities.text_document_sync.is_some());
        assert!(result.capabilities.diagnostic_provider.is_some());
        assert!(result.capabilities.completion_provider.is_some());
        assert!(result.capabilities.hover_provider.is_some());
    }

    #[test]
    fn test_compute_diagnostics() {
        use protobuf_edition_lsp::lsp_server::handlers::compute_diagnostics;

        let content = r#"
message Test {
  string field1 = 1;
  int32 field2 = 1;
}
"#;

        let diagnostics = compute_diagnostics(content);
        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .any(|d| d.message.to_lowercase().contains("duplicate field number")));
    }

    #[test]
    fn test_compute_completions() {
        use protobuf_edition_lsp::lsp_server::handlers::compute_completions;

        let content = r#"
message Test {
  str
}
"#;

        let position = Position {
            line: 2,
            character: 2, // Position at the beginning of "str"
        };
        let completions = compute_completions(content, position);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "string"));
    }

    #[test]
    fn test_compute_hover() {
        use protobuf_edition_lsp::lsp_server::handlers::compute_hover;

        let content = r#"
message Test {
  string name = 1;
}
"#;

        let position = Position {
            line: 2,
            character: 4,
        };
        let hover = compute_hover(content, position);

        assert!(hover.is_some());
    }
}
