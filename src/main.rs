use protobuf_edition_lsp::lsp_server::ProtobufLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    // Create the LSP service
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(ProtobufLanguageServer::new);

    // Run the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
