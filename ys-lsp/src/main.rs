//! # yatsuscript-lsp
//!
//! Language Server Protocol implementation for the YatsuScript scripting language.

mod analysis;
mod backend;
mod builtin_docs;

use backend::YatsuBackend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Read from stdin, write to stdout — standard LSP wire protocol.
    let stdin  = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| YatsuBackend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
