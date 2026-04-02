// src/brain/context.rs
use git2::Repository;
use tree_sitter::Parser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct ProjectIndex {
    pub index: Index,
    pub writer: Arc<RwLock<IndexWriter>>,
    pub path_field: Field,
    pub body_field: Field,
    pub folder: Arc<crate::brain::fold::FoldEngine>,
}

impl ProjectIndex {
    pub fn new() -> Result<Self> {
        let mut schema_builder = Schema::builder();
        let path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();
        
        let index = Index::create_in_ram(schema);
        let index_writer = index.writer(15_000_000)?;
        
        Ok(Self {
            index,
            writer: Arc::new(RwLock::new(index_writer)),
            path_field,
            body_field,
            folder: Arc::new(crate::brain::fold::FoldEngine::new()),
        })
    }
}

pub struct ProjectContext<'a> {
    pub repo: Option<Repository>,
    pub parser: Parser,
    pub p_index: Arc<ProjectIndex>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ProjectContext<'a> {
    pub fn new<P: AsRef<Path>>(path: P, p_index: Arc<ProjectIndex>) -> Result<Self> {
        let repo = Repository::open(path.as_ref()).ok();
        
        let mut parser = Parser::new();
        #[cfg(feature = "full-ast")]
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into()).unwrap_or_default();
        #[cfg(not(feature = "full-ast"))]
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap_or_default();

        Ok(Self {
            repo,
            parser,
            p_index,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn reactive_search(&self, query: &str) -> Result<Vec<String>> {
        let reader = self.p_index.index.reader()?;
        let searcher = reader.searcher();
        let query_parser = tantivy::query::QueryParser::for_index(&self.p_index.index, vec![self.p_index.body_field]);
        
        let q = match query_parser.parse_query(query) {
            Ok(q) => q,
            Err(_) => return Ok(vec![]), // fallback on syntax fail
        };
        
        let top_docs = searcher.search(&q, &tantivy::collector::TopDocs::with_limit(3))?;
        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            if let Some(path_val) = retrieved_doc.get_first(self.p_index.path_field) {
                if let Some(p) = path_val.as_str() {
                    results.push(p.to_string());
                }
            }
        }
        Ok(results)
    }

    pub fn extract_origami_context(&mut self, paths: Vec<String>, prompt: &str) -> String {
        let mut combined = String::new();
        combined.push_str("/// NOTE: The original full source is always used when applying the actual edit (Coder never sees the folded version for writes).\n\n");
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let folded = self.p_index.folder.fold_document(&path, &content, prompt, &mut self.parser);
                combined.push_str(&format!("--- FILE: {} ---\n{}\n\n", path, folded));
            }
        }
        combined
    }
}
