#!/usr/bin/env bash
set -euo pipefail

# Build WASM package
wasm-pack build --target web --no-default-features

cd "$(dirname "$0")/pkg"

# Patch generated package.json
node -e "
const pkg = require('./package.json');
pkg.name = 'ysc-wasm';
pkg.repository = {
  type: 'git',
  url: 'https://github.com/DevYatsu/ysc.git'
};
pkg.bugs = 'https://github.com/DevYatsu/ysc/issues';
pkg.homepage = 'https://github.com/DevYatsu/ysc#readme';
pkg.keywords = ['ysc', 'scripting', 'language', 'wasm', 'interpreter', 'playground', 'monaco', 'lsp'];
pkg.description = 'ysc language — interpreter (WASM), AST viewer, bytecode disassembler, syntax highlighting, and LSP';
pkg.exports = {
  '.': {
    'import': './ys_wasm.js',
    'types': './ys_wasm.d.ts'
  },
  './syntax': {
    'import': './syntax.js',
    'types': './syntax.d.ts'
  }
};
pkg.files = [
  'ys_wasm_bg.wasm',
  'ys_wasm.js',
  'ys_wasm.d.ts',
  'ys_wasm_bg.wasm.d.ts',
  'syntax.js',
  'syntax.d.ts',
  'ysc.tmLanguage.json'
];
require('fs').writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
console.log('✓ package.json patched');
"

# Copy assets
cp ../README.md .
cp ../ys_wasm.d.ts .
cp ../syntax.js .
cp ../syntax.d.ts .
cp ../../editors/vscode/syntaxes/ysc.tmLanguage.json .

echo "✓ Package ready in pkg/"
