import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('protobuf-edition-lsp');
    
    // Get server path from configuration or use bundled server
    let serverPath = config.get<string>('serverPath');
    if (!serverPath || serverPath.trim() === '') {
        // Use bundled server
        serverPath = context.asAbsolutePath(path.join('server', 'protobuf-edition-lsp'));
    }
    
    // Server options
    const serverOptions: ServerOptions = {
        command: serverPath,
        args: [],
        transport: TransportKind.stdio
    };
    
    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'proto' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.proto')
        }
    };
    
    // Create the language client
    client = new LanguageClient(
        'protobuf-edition-lsp',
        'Protobuf Edition LSP',
        serverOptions,
        clientOptions
    );
    
    // Start the client
    client.start();
    
    console.log('Protobuf Edition LSP extension activated');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}