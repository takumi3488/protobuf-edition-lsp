use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

#[derive(Debug, Clone)]
pub struct Document {
    pub content: String,
    pub version: i32,
}

pub struct DocumentStore {
    documents: HashMap<Url, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn open_document(&mut self, uri: Url, content: String, version: i32) {
        self.documents.insert(uri, Document { content, version });
    }

    pub fn update_document(&mut self, uri: Url, content: String, version: i32) {
        if let Some(doc) = self.documents.get_mut(&uri) {
            doc.content = content;
            doc.version = version;
        }
    }

    pub fn close_document(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub fn get_document(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }
}
