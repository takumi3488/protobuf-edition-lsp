"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const config = vscode.workspace.getConfiguration('protobuf-edition-lsp');
    // Get server path from configuration or use bundled server
    let serverPath = config.get('serverPath');
    if (!serverPath || serverPath.trim() === '') {
        // Use bundled server
        serverPath = context.asAbsolutePath(path.join('server', 'protobuf-edition-lsp'));
    }
    // Server options
    const serverOptions = {
        command: serverPath,
        args: [],
        transport: node_1.TransportKind.stdio
    };
    // Client options
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'proto' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.proto')
        }
    };
    // Create the language client
    client = new node_1.LanguageClient('protobuf-edition-lsp', 'Protobuf Edition LSP', serverOptions, clientOptions);
    // Start the client
    client.start();
    console.log('Protobuf Edition LSP extension activated');
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map