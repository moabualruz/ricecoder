use tantivy::schema::{Field, NumericOptions, Schema, SchemaBuilder, STORED, TEXT};

/// Wrapper around the Tantivy schema for BM25 indexing.
#[derive(Clone)]
pub struct LexicalSchema {
    schema: Schema,
    pub identifier_field: Field,
    pub comment_field: Field,
    pub code_field: Field,
    pub chunk_id_field: Field,
    pub file_path_field: Field,
    pub language_field: Field,
    pub repository_field: Field,
    pub token_count_field: Field,
}

impl LexicalSchema {
    pub fn build() -> Self {
        let mut builder = SchemaBuilder::default();
        let identifier_field = builder.add_text_field("identifier_terms", TEXT | STORED);
        let comment_field = builder.add_text_field("comment_terms", TEXT | STORED);
        let code_field = builder.add_text_field("code_terms", TEXT | STORED);
        let chunk_id_field = builder.add_i64_field("chunk_id", STORED);
        let file_path_field = builder.add_text_field("file_path", STORED);
        let language_field = builder.add_text_field("language", STORED);
        let repository_field = builder.add_i64_field("repository_id", STORED);
        let token_count_field = builder.add_i64_field(
            "token_count",
            NumericOptions::default().set_fast().set_stored(),
        );

        Self {
            schema: builder.build(),
            identifier_field,
            comment_field,
            code_field,
            chunk_id_field,
            file_path_field,
            language_field,
            repository_field,
            token_count_field,
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn field_name(&self, field: Field) -> &str {
        self.schema.get_field_entry(field).name()
    }
}
