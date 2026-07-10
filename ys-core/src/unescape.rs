//! String unescaping utility.
//!
//! Processes escape sequences in string literals and template literals.
//! Pure function — no parser or runtime dependencies.

/// Process escape sequences in a string slice.
///
/// Supports: `\n`, `\r`, `\t`, `\\`, `\"`, `\uXXXX` (4-digit Unicode).
pub fn unescape_string(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => res.push('\n'),
                Some('r') => res.push('\r'),
                Some('t') => res.push('\t'),
                Some('\\') => res.push('\\'),
                Some('"') => res.push('"'),
                Some('u') => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        if let Some(h) = chars.next() {
                            hex.push(h);
                        }
                    }
                    if let Ok(n) = u32::from_str_radix(&hex, 16) {
                        if let Some(uc) = std::char::from_u32(n) {
                            res.push(uc);
                        }
                    }
                }
                Some(other) => {
                    res.push('\\');
                    res.push(other);
                }
                None => res.push('\\'),
            }
        } else {
            res.push(c);
        }
    }
    res
}
