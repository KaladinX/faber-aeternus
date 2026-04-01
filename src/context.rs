use git2::Repository;
use tree_sitter::Parser;
use anyhow::Result;
use std::path::Path;

pub struct ProjectContext<'a> {
    pub repo: Option<Repository>,
    pub parser: Parser,
    _marker: std::marker::PhantomData<&'a ()>, // In a real app we might hold parser language
}

impl<'a> ProjectContext<'a> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path).ok(); // ok if it's not a git repo
        let parser = Parser::new();
        // Setup tree-sitter language for index hook.
        // In a real CLI we would detect language from file extension.
        // But for the stub, just initializing standard Parser is sufficient.
        Ok(Self {
            repo,
            parser,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn tree_sitter_parse(&mut self, source_code: &str) -> Option<tree_sitter::Tree> {
        self.parser.parse(source_code, None)
    }

    pub fn uncommitted_changes(&self) -> bool {
        if let Some(repo) = &self.repo {
            if let Ok(statuses) = repo.statuses(None) {
                return !statuses.is_empty();
            }
        }
        false
    }
}
