use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headers() {
        let text = "# Header 1\n## Header 2\n### Header 3";
        let elements = MarkdownParser::parse(text);

        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], MarkdownElement::Header(1, _)));
        assert!(matches!(elements[1], MarkdownElement::Header(2, _)));
        assert!(matches!(elements[2], MarkdownElement::Header(3, _)));
    }

    #[test]
    fn test_parse_code_block() {
        let text = "```rust\nfn main() {}\n```";
        let elements = MarkdownParser::parse(text);

        assert_eq!(elements.len(), 1);
        assert!(matches!(
            elements[0],
            MarkdownElement::CodeBlock(Some(_), _)
        ));
    }

    #[test]
    fn test_parse_list() {
        let text = "- Item 1\n- Item 2\n- Item 3";
        let elements = MarkdownParser::parse(text);

        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], MarkdownElement::ListItem(_)));
    }

    #[test]
    fn test_parse_inline_bold() {
        let text = "This is **bold** text";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Bold(_))));
    }

    #[test]
    fn test_parse_inline_italic() {
        let text = "This is *italic* text";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Italic(_))));
    }

    #[test]
    fn test_parse_inline_code() {
        let text = "Use `let x = 5;` for variables";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Code(_))));
    }

    #[test]
    fn test_parse_link() {
        let text = "Visit [example](https://example.com)";
        let elements = MarkdownParser::parse_inline(text);

        assert!(elements
            .iter()
            .any(|e| matches!(e, MarkdownElement::Link(_, _))));
    }

    #[test]
    fn test_render_plain() {
        let elements = vec![
            MarkdownElement::Header(1, "Title".to_string()),
            MarkdownElement::Text("Some text".to_string()),
        ];

        let output = MarkdownParser::render_plain(&elements);
        assert!(output.contains("# Title"));
        assert!(output.contains("Some text"));
    }
}