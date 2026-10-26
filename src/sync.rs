use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::model::{Protocol, SyncAction, SyncCriterion, SyncDirection, SyncItem};
use crate::network::list_remote_directory;

#[derive(Clone, Debug)]
pub struct ScannedFile {
    pub rel_path: String,
    pub filename: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: i64,
}

/// Recursively or non-recursively scans a local directory and returns a map of normalized relative paths to `ScannedFile`.
pub fn scan_local_dir(
    root: &Path,
    recursive: bool,
    show_hidden: bool,
) -> Result<HashMap<String, ScannedFile>, String> {
    let mut result = HashMap::new();
    if !root.exists() {
        return Err(format!("Local path does not exist: {}", root.display()));
    }

    let mut dir_queue: Vec<PathBuf> = vec![PathBuf::new()];

    while let Some(current_rel) = dir_queue.pop() {
        let current_dir = root.join(&current_rel);
        let read_dir = match std::fs::read_dir(&current_dir) {
            Ok(rd) => rd,
            Err(e) => return Err(format!("Cannot read directory {}: {}", current_dir.display(), e)),
        };

        for entry_res in read_dir {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();
            if name == "." || name == ".." || name == ".DS_Store" {
                continue;
            }
            if !show_hidden && name.starts_with('.') {
                continue;
            }

            let item_path = entry.path();
            let ft = entry.file_type().ok();
            let mut is_dir = ft.as_ref().map(|f| f.is_dir()).unwrap_or(false);

            let meta = entry.metadata().ok().or_else(|| std::fs::symlink_metadata(&item_path).ok());
            let (size, mtime) = if let Some(m) = meta {
                is_dir = m.is_dir();
                let s = if is_dir { 0 } else { m.len() };
                let t = m
                    .modified()
                    .ok()
                    .and_then(|dur| dur.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                (s, t)
            } else {
                (0, 0)
            };

            let rel_path_buf = current_rel.join(&name);
            let rel_str = rel_path_buf
                .to_string_lossy()
                .trim_start_matches('/')
                .replace('\\', "/");

            result.insert(
                rel_str.clone(),
                ScannedFile {
                    rel_path: rel_str,
                    filename: name,
                    is_dir,
                    size,
                    mtime,
                },
            );

            if is_dir && recursive {
                dir_queue.push(rel_path_buf);
            }
        }
    }

    Ok(result)
}

/// Recursively or non-recursively scans a remote directory and returns a map of normalized relative paths to `ScannedFile`.
pub fn scan_remote_dir(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_root: &str,
    recursive: bool,
    show_hidden: bool,
) -> Result<HashMap<String, ScannedFile>, String> {
    let mut result = HashMap::new();
    let root_clean = remote_root.trim_end_matches('/').to_string();
    let root_clean = if root_clean.is_empty() { "/".to_string() } else { root_clean };

    let mut queue: Vec<(String, String)> = vec![(root_clean.clone(), String::new())];
    let mut visited: HashSet<String> = HashSet::new();

    while let Some((curr_remote, curr_rel)) = queue.pop() {
        if visited.contains(&curr_remote) {
            continue;
        }
        visited.insert(curr_remote.clone());

        let entries = list_remote_directory(
            protocol,
            host,
            port,
            username,
            password,
            &curr_remote,
            show_hidden,
        )?;

        for child in entries {
            if child.name == "." || child.name == ".." || child.name == ".DS_Store" {
                continue;
            }
            if !show_hidden && child.name.starts_with('.') {
                continue;
            }

            let rel_str = if curr_rel.is_empty() {
                child.name.clone()
            } else {
                format!("{}/{}", curr_rel, child.name)
            };

            result.insert(
                rel_str.clone(),
                ScannedFile {
                    rel_path: rel_str.clone(),
                    filename: child.name.clone(),
                    is_dir: child.is_dir,
                    size: child.size,
                    mtime: child.modified_ts,
                },
            );

            if child.is_dir && recursive {
                let next_remote = if curr_remote == "/" {
                    format!("/{}", child.name)
                } else {
                    format!("{}/{}", curr_remote.trim_end_matches('/'), child.name)
                };
                queue.push((next_remote, rel_str));
            }
        }
    }

    Ok(result)
}

/// Compares local and remote directory maps based on direction, criteria, and mirror option.
pub fn compare_directories(
    local_map: &HashMap<String, ScannedFile>,
    remote_map: &HashMap<String, ScannedFile>,
    direction: SyncDirection,
    criterion: SyncCriterion,
    mirror_delete: bool,
) -> Vec<SyncItem> {
    let mut items = Vec::new();
    let mut all_paths: HashSet<&str> = HashSet::new();

    for k in local_map.keys() {
        all_paths.insert(k.as_str());
    }
    for k in remote_map.keys() {
        all_paths.insert(k.as_str());
    }

    let mut sorted_paths: Vec<&str> = all_paths.into_iter().collect();
    sorted_paths.sort();

    let mut next_id = 1;

    for rel_path in sorted_paths {
        let loc = local_map.get(rel_path);
        let rem = remote_map.get(rel_path);

        match direction {
            SyncDirection::LocalToRemote => {
                // Upload local files to remote
                match (loc, rem) {
                    (Some(l), None) => {
                        let reason = if l.is_dir {
                            "Nová složka (na serveru chybí)".to_string()
                        } else {
                            "Nový soubor (na serveru chybí)".to_string()
                        };
                        items.push(SyncItem {
                            id: next_id,
                            rel_path: rel_path.to_string(),
                            filename: l.filename.clone(),
                            action: SyncAction::Upload,
                            is_dir: l.is_dir,
                            local_size: l.size,
                            remote_size: 0,
                            local_mtime: l.mtime,
                            remote_mtime: 0,
                            reason,
                            selected: true,
                        });
                        next_id += 1;
                    }
                    (Some(l), Some(r)) => {
                        if !l.is_dir && !r.is_dir {
                            let should_sync = match criterion {
                                SyncCriterion::TimeAndSize => {
                                    l.mtime > r.mtime + 2 || l.size != r.size
                                }
                                SyncCriterion::TimeOnly => l.mtime > r.mtime + 2,
                                SyncCriterion::SizeOnly => l.size != r.size,
                            };

                            if should_sync {
                                let reason = if l.mtime > r.mtime + 2 {
                                    "Novější lokální verze".to_string()
                                } else {
                                    "Rozdílná velikost".to_string()
                                };
                                items.push(SyncItem {
                                    id: next_id,
                                    rel_path: rel_path.to_string(),
                                    filename: l.filename.clone(),
                                    action: SyncAction::Upload,
                                    is_dir: false,
                                    local_size: l.size,
                                    remote_size: r.size,
                                    local_mtime: l.mtime,
                                    remote_mtime: r.mtime,
                                    reason,
                                    selected: true,
                                });
                                next_id += 1;
                            }
                        }
                    }
                    (None, Some(r)) => {
                        if mirror_delete {
                            let reason = if r.is_dir {
                                "Smazat na serveru (na lokálu chybí)".to_string()
                            } else {
                                "Smazat na serveru (na lokálu chybí)".to_string()
                            };
                            items.push(SyncItem {
                                id: next_id,
                                rel_path: rel_path.to_string(),
                                filename: r.filename.clone(),
                                action: SyncAction::DeleteRemote,
                                is_dir: r.is_dir,
                                local_size: 0,
                                remote_size: r.size,
                                local_mtime: 0,
                                remote_mtime: r.mtime,
                                reason,
                                selected: true,
                            });
                            next_id += 1;
                        }
                    }
                    (None, None) => {}
                }
            }
            SyncDirection::RemoteToLocal => {
                // Download remote files to local
                match (loc, rem) {
                    (None, Some(r)) => {
                        let reason = if r.is_dir {
                            "Nová složka (na lokálu chybí)".to_string()
                        } else {
                            "Nový soubor (na lokálu chybí)".to_string()
                        };
                        items.push(SyncItem {
                            id: next_id,
                            rel_path: rel_path.to_string(),
                            filename: r.filename.clone(),
                            action: SyncAction::Download,
                            is_dir: r.is_dir,
                            local_size: 0,
                            remote_size: r.size,
                            local_mtime: 0,
                            remote_mtime: r.mtime,
                            reason,
                            selected: true,
                        });
                        next_id += 1;
                    }
                    (Some(l), Some(r)) => {
                        if !l.is_dir && !r.is_dir {
                            let should_sync = match criterion {
                                SyncCriterion::TimeAndSize => {
                                    r.mtime > l.mtime + 2 || r.size != l.size
                                }
                                SyncCriterion::TimeOnly => r.mtime > l.mtime + 2,
                                SyncCriterion::SizeOnly => r.size != l.size,
                            };

                            if should_sync {
                                let reason = if r.mtime > l.mtime + 2 {
                                    "Novější verze na serveru".to_string()
                                } else {
                                    "Rozdílná velikost".to_string()
                                };
                                items.push(SyncItem {
                                    id: next_id,
                                    rel_path: rel_path.to_string(),
                                    filename: r.filename.clone(),
                                    action: SyncAction::Download,
                                    is_dir: false,
                                    local_size: l.size,
                                    remote_size: r.size,
                                    local_mtime: l.mtime,
                                    remote_mtime: r.mtime,
                                    reason,
                                    selected: true,
                                });
                                next_id += 1;
                            }
                        }
                    }
                    (Some(l), None) => {
                        if mirror_delete {
                            let reason = if l.is_dir {
                                "Smazat lokálně (na serveru chybí)".to_string()
                            } else {
                                "Smazat lokálně (na serveru chybí)".to_string()
                            };
                            items.push(SyncItem {
                                id: next_id,
                                rel_path: rel_path.to_string(),
                                filename: l.filename.clone(),
                                action: SyncAction::DeleteLocal,
                                is_dir: l.is_dir,
                                local_size: l.size,
                                remote_size: 0,
                                local_mtime: l.mtime,
                                remote_mtime: 0,
                                reason,
                                selected: true,
                            });
                            next_id += 1;
                        }
                    }
                    (None, None) => {}
                }
            }
            SyncDirection::TwoWay => {
                // Two-way synchronization: newer wins
                match (loc, rem) {
                    (Some(l), None) => {
                        items.push(SyncItem {
                            id: next_id,
                            rel_path: rel_path.to_string(),
                            filename: l.filename.clone(),
                            action: SyncAction::Upload,
                            is_dir: l.is_dir,
                            local_size: l.size,
                            remote_size: 0,
                            local_mtime: l.mtime,
                            remote_mtime: 0,
                            reason: "Chybí na serveru".to_string(),
                            selected: true,
                        });
                        next_id += 1;
                    }
                    (None, Some(r)) => {
                        items.push(SyncItem {
                            id: next_id,
                            rel_path: rel_path.to_string(),
                            filename: r.filename.clone(),
                            action: SyncAction::Download,
                            is_dir: r.is_dir,
                            local_size: 0,
                            remote_size: r.size,
                            local_mtime: 0,
                            remote_mtime: r.mtime,
                            reason: "Chybí na lokálu".to_string(),
                            selected: true,
                        });
                        next_id += 1;
                    }
                    (Some(l), Some(r)) => {
                        if !l.is_dir && !r.is_dir {
                            if l.mtime > r.mtime + 2 {
                                items.push(SyncItem {
                                    id: next_id,
                                    rel_path: rel_path.to_string(),
                                    filename: l.filename.clone(),
                                    action: SyncAction::Upload,
                                    is_dir: false,
                                    local_size: l.size,
                                    remote_size: r.size,
                                    local_mtime: l.mtime,
                                    remote_mtime: r.mtime,
                                    reason: "Novější lokálně".to_string(),
                                    selected: true,
                                });
                                next_id += 1;
                            } else if r.mtime > l.mtime + 2 {
                                items.push(SyncItem {
                                    id: next_id,
                                    rel_path: rel_path.to_string(),
                                    filename: r.filename.clone(),
                                    action: SyncAction::Download,
                                    is_dir: false,
                                    local_size: l.size,
                                    remote_size: r.size,
                                    local_mtime: l.mtime,
                                    remote_mtime: r.mtime,
                                    reason: "Novější na serveru".to_string(),
                                    selected: true,
                                });
                                next_id += 1;
                            } else if l.size != r.size {
                                // Sizes differ but timestamps are close: upload local as primary
                                items.push(SyncItem {
                                    id: next_id,
                                    rel_path: rel_path.to_string(),
                                    filename: l.filename.clone(),
                                    action: SyncAction::Upload,
                                    is_dir: false,
                                    local_size: l.size,
                                    remote_size: r.size,
                                    local_mtime: l.mtime,
                                    remote_mtime: r.mtime,
                                    reason: "Rozdílná velikost".to_string(),
                                    selected: true,
                                });
                                next_id += 1;
                            }
                        }
                    }
                    (None, None) => {}
                }
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_directories_local_to_remote() {
        let mut local = HashMap::new();
        let mut remote = HashMap::new();

        local.insert(
            "file1.txt".to_string(),
            ScannedFile {
                rel_path: "file1.txt".to_string(),
                filename: "file1.txt".to_string(),
                is_dir: false,
                size: 100,
                mtime: 1700000010,
            },
        );

        local.insert(
            "file2.txt".to_string(),
            ScannedFile {
                rel_path: "file2.txt".to_string(),
                filename: "file2.txt".to_string(),
                is_dir: false,
                size: 200,
                mtime: 1700000020,
            },
        );

        remote.insert(
            "file1.txt".to_string(),
            ScannedFile {
                rel_path: "file1.txt".to_string(),
                filename: "file1.txt".to_string(),
                is_dir: false,
                size: 100,
                mtime: 1700000000,
            },
        );

        // In LocalToRemote: file1 is newer on local, file2 is missing on remote -> both upload!
        let items = compare_directories(
            &local,
            &remote,
            SyncDirection::LocalToRemote,
            SyncCriterion::TimeAndSize,
            false,
        );

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].rel_path, "file1.txt");
        assert_eq!(items[0].action, SyncAction::Upload);
        assert_eq!(items[1].rel_path, "file2.txt");
        assert_eq!(items[1].action, SyncAction::Upload);
    }

    #[test]
    fn test_compare_directories_remote_to_local() {
        let local = HashMap::new();
        let mut remote = HashMap::new();

        remote.insert(
            "server.log".to_string(),
            ScannedFile {
                rel_path: "server.log".to_string(),
                filename: "server.log".to_string(),
                is_dir: false,
                size: 500,
                mtime: 1700000050,
            },
        );

        let items = compare_directories(
            &local,
            &remote,
            SyncDirection::RemoteToLocal,
            SyncCriterion::TimeAndSize,
            false,
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rel_path, "server.log");
        assert_eq!(items[0].action, SyncAction::Download);
    }

    #[test]
    fn test_compare_directories_mirror_delete() {
        let local = HashMap::new();
        let mut remote = HashMap::new();

        remote.insert(
            "obsolete.txt".to_string(),
            ScannedFile {
                rel_path: "obsolete.txt".to_string(),
                filename: "obsolete.txt".to_string(),
                is_dir: false,
                size: 30,
                mtime: 1700000000,
            },
        );

        let items = compare_directories(
            &local,
            &remote,
            SyncDirection::LocalToRemote,
            SyncCriterion::TimeAndSize,
            true, // mirror_delete enabled
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rel_path, "obsolete.txt");
        assert_eq!(items[0].action, SyncAction::DeleteRemote);
    }
}
