# Faber-Aeternus Architecture

## Philosophy
Zero-cost-abstraction, safe-first, native terminal agent. Bringing the "Bring Your Own Agent" (BYOA) dream to real engineers.
Everything is built natively in Rust. Strict memory safety bounds and rigorous error handling with `anyhow`.

## Hexagonal Core

1. **TUI Layer (`iocraft` decl UI)**: 
   - Managed via `src/tui/`. Left file tree, center generation, right sandbox preview.
   - Strictly responsive and live-streamed.
2. **State & Context**:
   - `src/state.rs`: Holds global variables and Snapshot Manager logic for instantaneous reversions of corrupted operations.
   - `src/context.rs`: Wraps `tree-sitter` and `git2` for 100% locally-driven AST/VCS context injections.
3. **Sandbox & Tools**:
   - `src/sandbox.rs`: Uses Linux Bubblewrap (`bwrap`) where supported for fully isolated command executions spanning `/tmp`.
   - `src/tools/`: Replicates HIGH/MED/LOW permissions with a trait-based registry leveraging JSON schemas for model ingestion.
4. **LLM Abstraction**:
   - `src/llm/provider.rs`: Bridges abstract streams, switching seamlessly between `RemoteProvider` (HTTP hooks) and `CandleProvider` (local GGML execution).
