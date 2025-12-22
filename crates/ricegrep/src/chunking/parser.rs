use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;
use tree_sitter::{Node, Parser, Tree};

use crate::chunking::{
    errors::{ChunkingError, ChunkingResult},
    LanguageKind,
};

lazy_static! {
    static ref NODE_KINDS: HashMap<LanguageKind, Vec<&'static str>> = {
        use LanguageKind::*;
        HashMap::from([
            (
                Rust,
                vec!["function_item", "impl_item", "struct_item", "enum_item"],
            ),
            (Python, vec!["function_definition", "class_definition"]),
            (
                JavaScript,
                vec![
                    "function_declaration",
                    "method_definition",
                    "class_declaration",
                ],
            ),
            (
                TypeScript,
                vec![
                    "function_declaration",
                    "method_definition",
                    "class_declaration",
                ],
            ),
            (
                Tsx,
                vec![
                    "function_declaration",
                    "method_definition",
                    "class_declaration",
                ],
            ),
            (
                Java,
                vec![
                    "class_declaration",
                    "interface_declaration",
                    "method_declaration",
                ],
            ),
            (
                Go,
                vec!["function_declaration", "method_declaration", "type_spec"],
            ),
            (C, vec!["function_definition", "struct_specifier"]),
            (
                Cpp,
                vec!["function_definition", "class_specifier", "struct_specifier"],
            ),
        ])
    };
}

#[derive(Clone)]
pub struct ParserPool {
    supported: HashSet<LanguageKind>,
}

impl ParserPool {
    pub fn new(languages: &[LanguageKind]) -> Self {
        Self {
            supported: languages.iter().copied().collect(),
        }
    }

    pub fn parse(&self, language: LanguageKind, source: &str) -> ChunkingResult<Tree> {
        if !self.supported.contains(&language) {
            return Err(ChunkingError::UnsupportedLanguage {
                path: format!("{language:?}"),
            });
        }

        let mut parser = Parser::new();
        parser
            .set_language(self.ts_language(language))
            .map_err(|e| ChunkingError::Parser(e.to_string()))?;
        parser
            .parse(source, None)
            .ok_or_else(|| ChunkingError::Parser("failed to parse source".into()))
    }

    pub fn collect_semantic_units(
        &self,
        language: LanguageKind,
        tree: &Tree,
        source: &str,
    ) -> ChunkingResult<Vec<SemanticUnit>> {
        let mut units = Vec::new();
        let root = tree.root_node();
        let target_kinds = NODE_KINDS
            .get(&language)
            .map(|kinds| kinds.as_slice())
            .unwrap_or(&["source_file"]);
        self.traverse(root, target_kinds, &mut units, source)?;
        Ok(units)
    }

    fn traverse(
        &self,
        node: Node<'_>,
        target_kinds: &[&str],
        units: &mut Vec<SemanticUnit>,
        source: &str,
    ) -> ChunkingResult<()> {
        if target_kinds.contains(&node.kind()) {
            units.push(SemanticUnit::from_node(node, source));
            return Ok(());
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse(child, target_kinds, units, source)?;
        }
        Ok(())
    }

    fn ts_language(&self, language: LanguageKind) -> tree_sitter::Language {
        use LanguageKind::*;
        match language {
            Rust => tree_sitter_rust::language(),
            Python => tree_sitter_python::language(),
            JavaScript => tree_sitter_javascript::language(),
            TypeScript => tree_sitter_typescript::language_typescript(),
            Tsx => tree_sitter_typescript::language_tsx(),
            Java => tree_sitter_java::language(),
            Go => tree_sitter_go::language(),
            C => tree_sitter_c::language(),
            Cpp => tree_sitter_cpp::language(),
            LanguageKind::PlainText => tree_sitter_python::language(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticUnit {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub end_line: u32,
}

impl SemanticUnit {
    pub fn from_node(node: Node<'_>, _source: &str) -> Self {
        let start_position = node.start_position();
        let end_position = node.end_position();
        Self {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_line: start_position.row as u32 + 1,
            end_line: end_position.row as u32 + 1,
        }
    }

    pub fn extract_text(&self, content: &str) -> String {
        let bytes = content.as_bytes();
        let end = self.end_byte.min(bytes.len());
        let start = self.start_byte.min(end);
        String::from_utf8_lossy(&bytes[start..end]).to_string()
    }
}
