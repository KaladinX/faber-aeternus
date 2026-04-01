use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "faber-aeternus — BYOA TUI coding agent harness")]
pub struct Cli {
    #[arg(long, default_value = "grok")]
    pub provider: String,
    
    #[arg(long)]
    pub model: Option<String>,
    
    #[arg(long)]
    pub project: Option<String>,

    #[arg(long, help = "Enforce strict bubblewrap sandbox mode")]
    pub sandbox_strict: bool,
}