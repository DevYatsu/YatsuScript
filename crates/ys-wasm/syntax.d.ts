/** Monarch tokenizer definition for Monaco Editor. */
export const monarchLanguage: {
  defaultToken: string;
  tokenPostfix: string;
  ignoreCase: boolean;
  keywords: string[];
  operators: string[];
  symbols: RegExp;
  escapes: RegExp;
  tokenizer: Record<string, unknown>;
};

/**
 * Register the ysc language in Monaco Editor.
 * Call once before creating any editors.
 */
export function registerMonaco(): void;
