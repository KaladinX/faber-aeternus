// src/brain/dream.rs
use std::path::{Path, PathBuf};
use tokio::sync::{mpsc, watch};
use std::time::Duration;
use ignore::WalkBuilder;
use std::sync::Arc;

pub struct DreamDaemon {
    project_path: PathBuf,
    shutdown_rx: watch::Receiver<bool>,
    notify_tx: mpsc::Sender<String>,
}

impl DreamDaemon {
    pub fn new(
        project_path: impl AsRef<Path>,
        shutdown_rx: watch::Receiver<bool>,
        notify_tx: mpsc::Sender<String>,
    ) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            shutdown_rx,
            notify_tx,
        }
    }

    pub async fn run(mut self, p_index: Arc<crate::brain::context::ProjectIndex>) {
        let _ = self.notify_tx.send("☁️ Dream daemon spun up... orienting.".into()).await;

        loop {
            if *self.shutdown_rx.borrow() {
                let _ = self.notify_tx.send("💤 Dream daemon shutting down gracefully.".into()).await;
                break;
            }

            // Phase 1: Orient & Gather
            let mut file_count = 0;
            let mut total_indexed = 0;
            let walker = WalkBuilder::new(&self.project_path).build();
            for result in walker {
                if let Ok(entry) = result {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        file_count += 1;
                        let path = entry.path().to_string_lossy().to_string();
                        
                        // Limit to actual code texts
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            let mut doc = tantivy::TantivyDocument::new();
                            doc.add_text(p_index.path_field, path);
                            doc.add_text(p_index.body_field, content);
                            if let Ok(mut writer) = p_index.writer.write() {
                                let _ = writer.add_document(doc);
                                total_indexed += 1;
                            }
                        }
                    }
                }
            }

            if total_indexed > 0 {
                if let Ok(mut writer) = p_index.writer.write() {
                    let _ = writer.commit();
                }
            }

            let _ = self.notify_tx.send(format!("☁️ Dream: Oriented. Found {} files. Synthesized {} documents.", file_count, total_indexed)).await;

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(60)) => {}
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        let _ = self.notify_tx.send("💤 Dream daemon shutting down gracefully.".into()).await;
                        break;
                    }
                }
            }
        }
    }
}
