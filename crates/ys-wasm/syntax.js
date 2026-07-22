/**
 * YSC — Monarch tokenizer for Monaco Editor.
 *
 * Usage:
 *   import { monarchLanguage } from 'ysc-wasm/syntax.js';
 *   monaco.languages.register({ id: 'ysc' });
 *   monaco.languages.setMonarchTokensProvider('ysc', monarchLanguage);
 */
export const monarchLanguage = {
  defaultToken: '',
  tokenPostfix: '.ys',
  ignoreCase: false,

  keywords: [
    'fun', 'ret', 'if', 'else', 'for', 'while', 'in',
    'and', 'or', 'not', 'nil', 'true', 'false',
    'async', 'await', 'exp', 'use', 'switch',
    'break', 'move', 'except', 'fail', 'error', 'yield',
  ],

  operators: [
    '+=', '-=', '*=', '/=', '%=',
    '+', '-', '*', '/', '%',
    '==', '!=', '<', '<=', '>', '>=',
    '=', ':', '..', '->', '|', '!', ',', ';', '.',
  ],

  symbols: /[=><!~?:&|+\-*\/^%]+/,
  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Comments
      [/\/\/.*$/, 'comment'],
      [/#.*$/, 'comment'],

      // Strings
      [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],

      // Numbers
      [/\d+[eE][+-]?\d+/, 'number.float'],
      [/\d+\.\d+([eE][+-]?\d+)?/, 'number.float'],
      [/\d+/, 'number'],

      // Identifiers and keywords
      [/[a-zA-Z_$][\w$]*/, {
        cases: {
          '@keywords': 'keyword',
          '@default': 'identifier'
        }
      }],

      // Operators and brackets
      [/[{}()\[\]]/, '@brackets'],
      [/[.]/, 'delimiter'],
      [/[;,.]/, 'delimiter'],
      [/@symbols/, { cases: { '@operators': 'operator', '@default': '' } }],

      { include: '@whitespace' },
    ],

    string: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
    ],

    whitespace: [
      [/[ \t\r\n]+/, 'white'],
    ],
  },
};

/**
 * Register the ysc language in Monaco Editor.
 * Call once before creating any editors.
 */
export function registerMonaco() {
  if (typeof monaco === 'undefined') {
    console.warn('ysc syntax: monaco is not loaded');
    return;
  }
  const existing = monaco.languages.getEncodedLanguageId('ysc');
  if (existing !== 0) return; // already registered
  monaco.languages.register({ id: 'ysc' });
  monaco.languages.setMonarchTokensProvider('ysc', monarchLanguage);
}
