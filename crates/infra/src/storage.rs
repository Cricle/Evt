use std::path::{Path, PathBuf};

use evt_domain::AppError;
use tokio::fs;
use uuid::Uuid;

#[derive(Clone)]
pub struct LocalAttachmentStorage {
    root: PathBuf,
}

impl LocalAttachmentStorage {
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self, AppError> {
        let root = root.into();
        fs::create_dir_all(&root)
            .await
            .map_err(|err| AppError::Internal(format!("create attachment dir failed: {err}")))?;
        Ok(Self { root })
    }

    pub async fn save(&self, original_name: &str, bytes: &[u8]) -> Result<String, AppError> {
        let safe_name = sanitize_file_name(original_name);
        let storage_key = format!("{}_{}", Uuid::new_v4().simple(), safe_name);
        let target = self.root.join(&storage_key);
        fs::write(&target, bytes)
            .await
            .map_err(|err| AppError::Internal(format!("write attachment failed: {err}")))?;
        Ok(storage_key)
    }

    pub async fn read(&self, storage_key: &str) -> Result<Vec<u8>, AppError> {
        let target = self.root.join(storage_key);
        fs::read(&target)
            .await
            .map_err(|err| AppError::Internal(format!("read attachment failed: {err}")))
    }
}

fn sanitize_file_name(file_name: &str) -> String {
    let base = Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");

    let sanitized: String = base
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '_',
        })
        .collect();

    if sanitized.is_empty() {
        "file".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_file_name;

    #[test]
    fn sanitize_file_name_removes_paths_and_unsafe_chars() {
        assert_eq!(
            sanitize_file_name("../../hello world?.png"),
            "hello_world_.png"
        );
        assert_eq!(sanitize_file_name("plain.txt"), "plain.txt");
    }
}
