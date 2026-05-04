use evt_domain::AppError;
use std::path::PathBuf;

pub fn resolve_web_dist_dir(configured: &str) -> Result<PathBuf, AppError> {
    let configured_path = PathBuf::from(configured);
    let current_dir = std::env::current_dir().ok();
    let mut candidates = Vec::new();

    candidates.push(configured_path.clone());

    if configured_path.is_relative() {
        if let Some(current_dir) = current_dir.as_ref() {
            candidates.push(current_dir.join(&configured_path));
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                candidates.push(exe_dir.join(&configured_path));
                if let Some(parent_dir) = exe_dir.parent() {
                    candidates.push(parent_dir.join(&configured_path));
                }
            }
        }
    }

    candidates.dedup();

    let resolved = candidates
        .into_iter()
        .find(|path| path.join("index.html").is_file())
        .ok_or_else(|| {
            AppError::Internal(format!(
                "web dist dir is missing index.html: configured={configured}"
            ))
        })?;

    if resolved.is_relative() {
        if let Some(current_dir) = current_dir {
            return Ok(current_dir.join(resolved));
        }
    }

    Ok(resolved)
}

pub fn resolve_spa_index_path(configured: &str) -> Result<PathBuf, AppError> {
    Ok(resolve_web_dist_dir(configured)?.join("index.html"))
}

#[cfg(test)]
mod tests {
    use super::{resolve_spa_index_path, resolve_web_dist_dir};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("evt-{label}-{unique}"))
    }

    #[test]
    fn resolves_existing_absolute_web_dist() {
        let dist_dir = unique_temp_dir("web-dist-abs");
        fs::create_dir_all(&dist_dir).expect("create absolute test web dist");
        fs::write(dist_dir.join("index.html"), "evt").expect("write absolute index");

        let resolved = resolve_web_dist_dir(dist_dir.to_str().expect("utf8 path"))
            .expect("resolve absolute dist dir");
        assert_eq!(resolved, dist_dir);
        assert_eq!(
            resolve_spa_index_path(resolved.to_str().expect("utf8 path"))
                .expect("resolve absolute spa index"),
            dist_dir.join("index.html")
        );
    }

    #[test]
    fn resolves_relative_web_dist_from_current_dir() {
        let temp_root = unique_temp_dir("web-dist-relative");
        let dist_dir = temp_root.join("web/dist");
        fs::create_dir_all(&dist_dir).expect("create relative test web dist");
        fs::write(dist_dir.join("index.html"), "evt").expect("write relative index");

        let previous_dir = std::env::current_dir().expect("capture current dir");
        std::env::set_current_dir(&temp_root).expect("enter temp root");

        let resolved = resolve_web_dist_dir("./web/dist").expect("resolve relative dist dir");
        assert_eq!(resolved, dist_dir);

        std::env::set_current_dir(previous_dir).expect("restore current dir");
    }
}
