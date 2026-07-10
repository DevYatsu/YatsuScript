//! Template literal parsing helpers.
//!
//! Extracts the pure-string-splitting logic (no parser state needed).
//! The actual codegen happens in `Parser::parse_template_literal` which
//! calls this function.

/// A part of a template literal — either a literal string or an expression.
pub enum TemplatePart<'source> {
    Literal(&'source str),
    Expr(&'source str),
}

/// Split a template literal body into literal/expression parts.
///
/// For example, `` `hello ${name}!` `` produces:
/// `[Literal("hello "), Expr("name"), Literal("!")]`
pub fn split_template_parts(s: &str) -> Vec<TemplatePart<'_>> {
    let mut parts = Vec::new();
    let mut last = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            if i > last {
                parts.push(TemplatePart::Literal(&s[last..i]));
            }
            let mut depth = 1;
            let start = i + 2;
            i += 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'{' {
                    depth += 1;
                } else if bytes[i] == b'}' {
                    depth -= 1;
                }
                i += 1;
            }
            if depth == 0 {
                parts.push(TemplatePart::Expr(&s[start..i - 1]));
            }
            last = i;
        } else {
            i += 1;
        }
    }
    if last < s.len() {
        parts.push(TemplatePart::Literal(&s[last..]));
    }
    parts
}
