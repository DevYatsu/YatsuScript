//! Supplementary lexer tests, adding edge-cases on top of the existing inline
//! `#[cfg(test)]` block in `lexer.rs`.

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use logos::Logos;

    // Helpers

    fn tokens(input: &str) -> Vec<Result<Token<'_>, crate::lexer::LexingError>> {
        Token::lexer(input).collect()
    }

    fn ok_tokens(input: &str) -> Vec<Token<'_>> {
        tokens(input)
            .into_iter()
            .map(|r| r.expect("expected only valid tokens"))
            .collect()
    }

    // Number literals

    #[test]
    fn lex_integer_literal() {
        assert_eq!(ok_tokens("42"), vec![Token::Number(42.0)]);
    }

    #[test]
    fn lex_float_literal() {
        let toks = ok_tokens("3.14");
        assert_eq!(toks.len(), 1);
        if let Token::Number(n) = toks[0] {
            assert!((n - 3.14).abs() < 1e-10);
        } else {
            panic!("expected Number token");
        }
    }

    #[test]
    fn lex_negative_number() {
        assert_eq!(ok_tokens("-7"), vec![Token::Number(-7.0)]);
    }

    #[test]
    fn lex_number_with_underscores() {
        // 1_000_000 → 1000000.0
        assert_eq!(ok_tokens("1_000_000"), vec![Token::Number(1_000_000.0)]);
    }

    #[test]
    fn lex_scientific_notation() {
        let toks = ok_tokens("1e3");
        assert_eq!(toks.len(), 1);
        if let Token::Number(n) = toks[0] {
            assert!((n - 1000.0).abs() < 1e-10);
        } else {
            panic!("expected Number token");
        }
    }

    // String literals

    #[test]
    fn lex_empty_string() {
        assert_eq!(ok_tokens("\"\""), vec![Token::String("")]);
    }

    #[test]
    fn lex_string_with_spaces() {
        assert_eq!(
            ok_tokens("\"hello world\""),
            vec![Token::String("hello world")]
        );
    }

    #[test]
    fn lex_string_escapes_preserved_in_slice() {
        // The lexer keeps the raw escape sequences in the slice; unescaping
        // happens in the parser.
        let toks = ok_tokens(r#""hello\nworld""#);
        assert_eq!(toks.len(), 1);
        if let Token::String(s) = toks[0] {
            assert!(s.contains("\\n"), "raw escape must be present in token");
        } else {
            panic!("expected String token");
        }
    }

    // Boolean literals

    #[test]
    fn lex_true() {
        assert_eq!(ok_tokens("true"), vec![Token::Bool(true)]);
    }

    #[test]
    fn lex_false() {
        assert_eq!(ok_tokens("false"), vec![Token::Bool(false)]);
    }

    // Identifiers

    #[test]
    fn lex_identifier_alpha() {
        assert_eq!(ok_tokens("foo"), vec![Token::Identifier("foo")]);
    }

    #[test]
    fn lex_identifier_with_underscore_prefix() {
        assert_eq!(ok_tokens("_bar"), vec![Token::Identifier("_bar")]);
    }

    #[test]
    fn lex_identifier_with_digits() {
        assert_eq!(ok_tokens("abc123"), vec![Token::Identifier("abc123")]);
    }

    #[test]
    fn lex_keywords_not_identifiers() {
        // Keywords must NOT lex as identifiers.
        let toks = ok_tokens("let mut fn");
        assert!(
            matches!(toks[0], Token::ImmutableVar),
            "let must be ImmutableVar"
        );
        assert!(
            matches!(toks[1], Token::MutableVar),
            "mut must be MutableVar"
        );
        assert!(matches!(toks[2], Token::Fn), "fn must be Fn");
    }

    // Comments

    #[test]
    fn line_comment_is_skipped() {
        // The raw logos lexer emits LineComment as a token; the Parser filters it.
        // So the raw iterator gives: [LineComment, Newline].
        let toks = ok_tokens("// this is a comment\n");
        assert!(
            toks.contains(&Token::Newline),
            "newline after comment must be present"
        );
        assert!(
            !toks.iter().any(|t| matches!(
                t,
                Token::Identifier(_) | Token::Number(_) | Token::Bool(_) | Token::String(_)
            )),
            "comment should not produce value tokens"
        );
    }

    #[test]
    fn block_comment_is_skipped() {
        let toks = ok_tokens("/* block */ 42");
        assert_eq!(toks, vec![Token::Number(42.0)]);
    }

    #[test]
    fn inline_comment_does_not_consume_next_line() {
        let toks = ok_tokens("42 // comment\n99");
        // Raw logos includes LineComment as a token; parser strips it.
        // Expects: Number(42) LineComment Newline Number(99)
        assert!(toks.contains(&Token::Number(42.0)), "must have 42");
        assert!(toks.contains(&Token::Number(99.0)), "must have 99");
        assert!(toks.contains(&Token::Newline), "must have Newline");
    }

    // Whitespace

    #[test]
    fn horizontal_whitespace_is_skipped() {
        let toks = ok_tokens("   42   ");
        assert_eq!(toks, vec![Token::Number(42.0)]);
    }

    #[test]
    fn tabs_are_skipped() {
        let toks = ok_tokens("\t\t99");
        assert_eq!(toks, vec![Token::Number(99.0)]);
    }

    #[test]
    fn newline_is_a_token() {
        let toks = ok_tokens("\n");
        assert_eq!(toks, vec![Token::Newline]);
    }

    // Operators

    #[test]
    fn lex_range_before_dot() {
        // '..' must win over '.' when two dots are adjacent.
        let toks = ok_tokens("..");
        assert_eq!(toks, vec![Token::Range]);
    }

    #[test]
    fn lex_single_dot_after_range() {
        let toks = ok_tokens(".. .");
        assert_eq!(toks, vec![Token::Range, Token::Dot]);
    }

    #[test]
    fn lex_le_before_lt() {
        // '<=' must win over '<' when followed by '='.
        let toks = ok_tokens("<=");
        assert_eq!(toks, vec![Token::Le]);
    }

    #[test]
    fn lex_ge_before_gt() {
        let toks = ok_tokens(">=");
        assert_eq!(toks, vec![Token::Ge]);
    }

    #[test]
    fn lex_eq_two_chars() {
        let toks = ok_tokens("==");
        assert_eq!(toks, vec![Token::Eq]);
    }

    #[test]
    fn lex_ne_two_chars() {
        let toks = ok_tokens("!=");
        assert_eq!(toks, vec![Token::Ne]);
    }

    // Invalid / error cases

    #[test]
    fn non_ascii_character_produces_error() {
        use crate::lexer::LexingError;
        let toks = tokens("é");
        assert!(
            toks.iter()
                .any(|r| matches!(r, Err(LexingError::NonAsciiCharacter(_)))),
            "non-ASCII character should produce a NonAsciiCharacter error"
        );
    }
}
