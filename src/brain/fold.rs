// src/brain/fold.rs
use std::collections::{HashMap, HashSet};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher};
use tree_sitter::Parser;

/// Caches folded code snippets to avoid reprocessing ASTs on high-frequency LLM runs
pub struct FoldEngine {
    cache: Arc<Mutex<LruCache<String, String>>>,
    foldable_kinds: HashMap<&'static str, Vec<&'static str>>,
}

fn hash_prompt(prompt: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prompt.hash(&mut hasher);
    hasher.finish()
}

impl FoldEngine {
    pub fn new() -> Self {
        let mut foldable = HashMap::new();
        foldable.insert("rust", vec!["function_item", "impl_item", "struct_item"]);
        foldable.insert("python", vec!["function_definition", "class_definition"]);
        foldable.insert("go", vec!["function_declaration", "method_declaration"]);
        foldable.insert("javascript", vec!["function_declaration", "class_declaration", "method_definition", "arrow_function"]);
        foldable.insert("typescript", vec!["function_declaration", "class_declaration", "method_definition", "interface_declaration", "arrow_function"]);

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
            foldable_kinds: foldable,
        }
    }

    /// Extracts alphanumeric words > 3 characters from prompt to use as Marble-origami heuristic triggers
    fn extract_keywords(prompt: &str) -> HashSet<String> {
        prompt.split_whitespace()
              .map(|s| s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<String>())
              .filter(|s| s.len() > 3)
              .collect()
    }

    /// Recursively folds block/AST nodes that do not contain prompt keywords
    pub fn fold_document(&self, path: &str, source: &str, prompt: &str, parser: &mut Parser) -> String {
        let cache_key = format!("{}_{}", path, hash_prompt(prompt));
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return cached.clone();
        }

        let tree = match parser.parse(source, None) {
             Some(t) => t,
             None => return source.to_string(), // fallback
        };

        let ext = std::path::Path::new(path).extension().and_then(|x| x.to_str()).unwrap_or("");
        let lang_id = match ext {
            "rs" => "rust",
            "py" => "python",
            "go" => "go",
            "js" => "javascript",
            "ts" => "typescript",
            _ => "rust" // default structural assumption
        };

        let allowed_kinds = match self.foldable_kinds.get(lang_id) {
            Some(v) => v,
            None => return source.to_string(), // No AST mapping, return raw
        };

        let keywords = Self::extract_keywords(prompt);
        let bytes = source.as_bytes();

        let mut replacements = Vec::new();
        let mut stack = vec![tree.root_node()];
        
        while let Some(node) = stack.pop() {
            let mut will_fold = false;
            
            if allowed_kinds.contains(&node.kind()) {
                if let Ok(node_text) = std::str::from_utf8(&bytes[node.start_byte()..node.end_byte()]) {
                    let node_text_low = node_text.to_lowercase();
                    // Keyword Intersection Heuristic (Origami expansion trigger)
                    let matches = keywords.iter().any(|kw| node_text_low.contains(kw));
                    
                    if !matches {
                        // Snip node
                        for i in 0..node.child_count() {
                            let child = node.child(i).unwrap();
                            let kind = child.kind();
                            if kind == "block" || kind == "statement_block" || kind == "block_suite" {
                                replacements.push((child.start_byte(), child.end_byte()));
                                will_fold = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !will_fold {
                for i in 0..node.child_count() {
                    stack.push(node.child(i).unwrap());
                }
            }
        }

        // Sort ascending
        replacements.sort_by_key(|&(start, _)| start);

        let mut output = String::with_capacity(source.len());
        let mut last_end = 0;

        for (start, end) in replacements {
            if start >= last_end {
                output.push_str(std::str::from_utf8(&bytes[last_end..start]).unwrap_or(""));
                output.push_str(" { /* ... folded by faber-aeternus ... */ }");
                last_end = end;
            }
        }

        if last_end < bytes.len() {
             output.push_str(std::str::from_utf8(&bytes[last_end..]).unwrap_or(""));
        }

        self.cache.lock().unwrap().put(cache_key, output.clone());
        output
    }
}
