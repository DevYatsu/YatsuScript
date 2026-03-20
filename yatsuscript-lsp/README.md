# yatsuscript-lsp

> A full Language Server Protocol (LSP) implementation for the YatsuScript programming language.

`yatsuscript-lsp` makes YatsuScript feel like a modern, professional language by providing IDE features like intelligent highlighting, real-time diagnostics, and documentation hover.

## Features

- **Semantic Tokens**: Context-aware syntax highlighting (not just regex-based).
- **Diagnostics**: Real-time error reporting as you type.
- **Hover Provider**: Built-in documentation and function signature lookup.
- **Document Symbols**: Quickly navigate through functions and variables.
- **Auto-Completion**: Discoverable native built-ins and user-defined functions.

## Features Checklist

- [x] Diagnostics (Lexing, Parsing)
- [x] Semantic Tokens (Highlighting)
- [x] Hover (Built-in Documentation)
- [x] Document Symbols (Breadcrumbs)
- [ ] Go To Definition (Planned)
- [ ] Rename Refactoring (Planned)

## Quick Start

### Installation

```bash
cargo build -p yatsuscript-lsp
```

### Editor Integration

#### Visual Studio Code

Point your client-side LSP configuration to the `yatsuscript-lsp` binary. For example, using the `vscode-lsp-client` extension:

```json
{
    "yatsuscript.serverPath": "/path/to/yatsuscript-lsp"
}
```

#### Neovim

Add this to your `init.lua` (using `nvim-lspconfig`):

```lua
local configs = require('lspconfig.configs')
configs.yatsuscript = {
    default_config = {
        cmd = { 'yatsuscript-lsp' },
        filetypes = { 'ys', 'yatsuscript' },
        root_dir = function(fname)
            return lspconfig.util.find_git_ancestor(fname) or vim.loop.os_homedir()
        end,
        settings = {},
    },
}

require('lspconfig').yatsuscript.setup {
    on_attach = my_custom_on_attach,
}
```

## Architecture

1. **[Backend](src/backend.rs)**: The core LSP server implementation using the `tower-lsp` framework.
2. **[Analysis](src/analysis.rs)**: Orchestrates the `ys-core` parser to extract semantic data from the source.
3. **[Built-in Docs](src/builtin_docs.rs)**: Contains the documentation data for native YatsuScript functions.
4. **[Main](src/main.rs)**: Configures the transport layer (stdio) and handles the server lifecycle.

## License

MIT © Yanis
