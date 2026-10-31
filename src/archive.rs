use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use crate::model::FileEntry;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    SevenZip,
}

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::SevenZip => "7z",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "ZIP (.zip)",
            ArchiveFormat::TarGz => "TAR.GZ (.tar.gz)",
            ArchiveFormat::SevenZip => "7-Zip (.7z)",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        match ext.as_str() {
            "zip" => Some(ArchiveFormat::Zip),
            "tar.gz" | "tgz" => Some(ArchiveFormat::TarGz),
            "7z" => Some(ArchiveFormat::SevenZip),
            _ => None,
        }
    }
}

pub fn is_archive_file(path_or_name: &str) -> bool {
    let lower = path_or_name.to_lowercase();
    lower.ends_with(".zip")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tar")
        || lower.ends_with(".7z")
        || lower.ends_with(".rar")
        || lower.ends_with(".gz")
}

pub fn compress_local(
    sources: &[PathBuf],
    target_archive: &Path,
    format: ArchiveFormat,
    password: Option<&str>,
) -> Result<(), String> {
    if sources.is_empty() {
        return Err("No files or folders selected to compress".to_string());
    }

    match format {
        ArchiveFormat::Zip => compress_zip(sources, target_archive, password),
        ArchiveFormat::TarGz => compress_tar_gz(sources, target_archive),
        ArchiveFormat::SevenZip => compress_sevenz(sources, target_archive, password),
    }
}

fn compress_zip(sources: &[PathBuf], target_archive: &Path, password: Option<&str>) -> Result<(), String> {
    let file = File::create(target_archive).map_err(|e| format!("Failed to create archive: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let mut options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    if let Some(pass) = password.filter(|p| !p.trim().is_empty()) {
        options = options.with_aes_encryption(zip::AesMode::Aes256, pass);
    }

    for src in sources {
        let base_name = src.file_name().unwrap_or_default().to_string_lossy();
        if src.is_dir() {
            add_dir_to_zip(&mut zip, src, Path::new(&*base_name), options)?;
        } else if src.is_file() {
            add_file_to_zip(&mut zip, src, &base_name, options)?;
        }
    }

    zip.finish().map_err(|e| format!("Failed to finalize zip archive: {}", e))?;
    Ok(())
}

fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir_path: &Path,
    archive_prefix: &Path,
    options: zip::write::FileOptions<'_, ()>,
) -> Result<(), String> {
    let dir_name_str = archive_prefix.to_string_lossy();
    let dir_entry_name = if dir_name_str.ends_with('/') {
        dir_name_str.to_string()
    } else {
        format!("{}/", dir_name_str)
    };
    zip.add_directory(&dir_entry_name, options)
        .map_err(|e| format!("Failed to add directory entry: {}", e))?;

    let entries = fs::read_dir(dir_path).map_err(|e| format!("Failed to read dir: {}", e))?;
    for entry in entries.flatten() {
        let p = entry.path();
        let fname = entry.file_name();
        let next_prefix = archive_prefix.join(fname);
        if p.is_dir() {
            add_dir_to_zip(zip, &p, &next_prefix, options)?;
        } else if p.is_file() {
            add_file_to_zip(zip, &p, &next_prefix.to_string_lossy(), options)?;
        }
    }
    Ok(())
}

fn add_file_to_zip<W: Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    file_path: &Path,
    entry_name: &str,
    options: zip::write::FileOptions<'_, ()>,
) -> Result<(), String> {
    zip.start_file(entry_name, options)
        .map_err(|e| format!("Failed to start file in zip: {}", e))?;
    let mut f = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    io::copy(&mut f, zip).map_err(|e| format!("Failed to write file to zip: {}", e))?;
    Ok(())
}

fn compress_tar_gz(sources: &[PathBuf], target_archive: &Path) -> Result<(), String> {
    let file = File::create(target_archive).map_err(|e| format!("Failed to create archive: {}", e))?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    for src in sources {
        let base_name = src.file_name().unwrap_or_default().to_string_lossy();
        if src.is_dir() {
            tar.append_dir_all(&*base_name, src)
                .map_err(|e| format!("Failed to append dir to tar.gz: {}", e))?;
        } else if src.is_file() {
            let mut f = File::open(src).map_err(|e| format!("Failed to open file: {}", e))?;
            tar.append_file(&*base_name, &mut f)
                .map_err(|e| format!("Failed to append file to tar.gz: {}", e))?;
        }
    }

    tar.finish().map_err(|e| format!("Failed to finalize tar.gz: {}", e))?;
    Ok(())
}

fn compress_sevenz(sources: &[PathBuf], target_archive: &Path, password: Option<&str>) -> Result<(), String> {
    let clean_pass = password.unwrap_or("").trim();
    if sources.len() == 1 {
        if clean_pass.is_empty() {
            sevenz_rust::compress_to_path(&sources[0], target_archive)
                .map_err(|e| format!("7-Zip compression failed: {}", e))?;
        } else {
            sevenz_rust::compress_to_path_encrypted(&sources[0], target_archive, clean_pass.into())
                .map_err(|e| format!("7-Zip compression failed: {}", e))?;
        }
        return Ok(());
    }

    // For multiple sources, create a temporary directory or compress each
    let temp_dir = std::env::temp_dir().join(format!("ruster_7z_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    for src in sources {
        let fname = src.file_name().unwrap_or_default();
        let target = temp_dir.join(fname);
        if src.is_dir() {
            copy_dir_all(src, &target)?;
        } else {
            fs::copy(src, target).map_err(|e| e.to_string())?;
        }
    }

    let res = if clean_pass.is_empty() {
        sevenz_rust::compress_to_path(&temp_dir, target_archive)
    } else {
        sevenz_rust::compress_to_path_encrypted(&temp_dir, target_archive, clean_pass.into())
    }.map_err(|e| format!("7-Zip compression failed: {}", e));

    let _ = fs::remove_dir_all(&temp_dir);
    res
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn extract_local(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    extract_local_with_password(archive_path, dest_dir, None)
}

pub fn extract_local_with_password(
    archive_path: &Path,
    dest_dir: &Path,
    password: Option<&str>,
) -> Result<(), String> {
    if !archive_path.exists() {
        return Err("Archive file does not exist".to_string());
    }
    fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create destination: {}", e))?;

    let name = archive_path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();

    if name.ends_with(".zip") {
        extract_zip(archive_path, dest_dir, password)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir)
    } else if name.ends_with(".tar") {
        extract_tar(archive_path, dest_dir)
    } else if name.ends_with(".7z") {
        if let Some(pass) = password.filter(|p| !p.trim().is_empty()) {
            sevenz_rust::decompress_file_with_password(archive_path, dest_dir, pass.into())
                .map_err(|e| format!("Failed to extract 7z: {}", e))
        } else {
            sevenz_rust::decompress_file(archive_path, dest_dir)
                .map_err(|e| format!("Failed to extract 7z: {}", e))
        }
    } else if name.ends_with(".rar") {
        extract_rar(archive_path, dest_dir)
    } else if name.ends_with(".gz") {
        extract_single_gz(archive_path, dest_dir)
    } else {
        Err(format!("Unsupported archive format: {}", name))
    }
}

fn extract_zip(archive_path: &Path, dest_dir: &Path, password: Option<&str>) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = if let Some(pass) = password.filter(|p| !p.trim().is_empty()) {
            archive.by_index_decrypt(i, pass.as_bytes()).map_err(|e| format!("Zip decryption error: {}", e))?
        } else {
            archive.by_index(i).map_err(|e| format!("Zip entry error: {}", e))?
        };
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("Cannot create dir: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("Cannot create dir: {}", e))?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| format!("Cannot create file: {}", e))?;
            io::copy(&mut file, &mut outfile).map_err(|e| format!("Extract error: {}", e))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                let _ = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode));
            }
        }
    }
    Ok(())
}

fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Cannot open tar.gz: {}", e))?;
    let tar = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(dest_dir).map_err(|e| format!("Failed to unpack tar.gz: {}", e))?;
    Ok(())
}

fn extract_tar(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Cannot open tar: {}", e))?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(dest_dir).map_err(|e| format!("Failed to unpack tar: {}", e))?;
    Ok(())
}

fn extract_rar(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let path_str = archive_path.to_str().ok_or("Invalid path for RAR")?;
    let mut archive = unrar::Archive::new(path_str)
        .open_for_processing()
        .map_err(|e| format!("Failed to open RAR: {}", e))?;
    while let Ok(Some(header)) = archive.read_header() {
        archive = header
            .extract_with_base(dest_dir)
            .map_err(|e| format!("Failed to extract RAR entry: {}", e))?;
    }
    Ok(())
}

fn extract_single_gz(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Cannot open .gz: {}", e))?;
    let mut gz = flate2::read::GzDecoder::new(file);
    let orig_name = archive_path.file_name().unwrap_or_default().to_string_lossy();
    let uncompressed_name = orig_name.strip_suffix(".gz").unwrap_or(&orig_name);
    let outpath = dest_dir.join(uncompressed_name);
    let mut outfile = File::create(&outpath).map_err(|e| format!("Cannot create output file: {}", e))?;
    io::copy(&mut gz, &mut outfile).map_err(|e| format!("Failed to decompress gz: {}", e))?;
    Ok(())
}

// ==========================================
// Virtual Archive Browsing (VFS)
// ==========================================

#[derive(Clone, Debug)]
pub struct RawArchiveEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_str: String,
    pub modified_ts: i64,
}

/// Splits a path like `/path/to/archive.zip/inner/sub` into:
/// - Real archive file on disk: `/path/to/archive.zip`
/// - Inner relative path inside archive: `inner/sub` (or `""` if at archive root).
pub fn split_archive_path(path: &Path) -> Option<(PathBuf, String)> {
    let mut current = path;
    let mut sub_parts = Vec::new();

    // Check if the path itself is an existing archive file
    if current.is_file() && is_archive_file(&current.to_string_lossy()) {
        return Some((current.to_path_buf(), String::new()));
    }

    while let Some(parent) = current.parent() {
        if let Some(name) = current.file_name() {
            sub_parts.push(name.to_string_lossy().to_string());
        }
        if parent.is_file() && is_archive_file(&parent.to_string_lossy()) {
            sub_parts.reverse();
            let inner = sub_parts.join("/");
            return Some((parent.to_path_buf(), inner));
        }
        current = parent;
    }
    None
}

pub fn is_valid_archive_path(path: &Path) -> bool {
    split_archive_path(path).is_some()
}

pub fn read_archive_directory(
    archive_path: &Path,
    inner_path: &str,
    show_hidden: bool,
) -> Vec<FileEntry> {
    let name_lower = archive_path.to_string_lossy().to_lowercase();
    let raw_entries = if name_lower.ends_with(".zip") {
        read_zip_raw_entries(archive_path).unwrap_or_default()
    } else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
        read_tar_raw_entries(archive_path, true).unwrap_or_default()
    } else if name_lower.ends_with(".tar") {
        read_tar_raw_entries(archive_path, false).unwrap_or_default()
    } else if name_lower.ends_with(".7z") {
        read_sevenz_raw_entries(archive_path).unwrap_or_default()
    } else if name_lower.ends_with(".rar") {
        read_rar_raw_entries(archive_path).unwrap_or_default()
    } else if name_lower.ends_with(".gz") {
        let base_name = archive_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let inner_name = base_name.strip_suffix(".gz").unwrap_or(&base_name).to_string();
        let meta = fs::metadata(archive_path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let ts = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mod_str = format_unix_timestamp(ts);
        vec![RawArchiveEntry {
            name: inner_name,
            is_dir: false,
            size,
            modified_str: mod_str,
            modified_ts: ts as i64,
        }]
    } else {
        Vec::new()
    };

    build_archive_directory_view(archive_path, inner_path, raw_entries, show_hidden)
}

fn read_zip_raw_entries(archive_path: &Path) -> Result<Vec<RawArchiveEntry>, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entries = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let raw_name = entry.name().to_string();
        let is_dir = entry.is_dir() || raw_name.ends_with('/');
        let (mod_str, mod_ts) = if let Some(dt) = entry.last_modified() {
            (
                format!("{:04}-{:02}-{:02} {:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute()),
                0,
            )
        } else {
            ("-".to_string(), 0)
        };

        entries.push(RawArchiveEntry {
            name: raw_name,
            is_dir,
            size: entry.size(),
            modified_str: mod_str,
            modified_ts: mod_ts,
        });
    }

    Ok(entries)
}

fn read_tar_raw_entries(archive_path: &Path, is_gz: bool) -> Result<Vec<RawArchiveEntry>, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();

    if is_gz {
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        if let Ok(tar_entries) = archive.entries() {
            for entry_res in tar_entries.flatten() {
                let path = entry_res.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                let is_dir = entry_res.header().entry_type().is_dir() || path.ends_with('/');
                let size = entry_res.header().size().unwrap_or(0);
                let mtime = entry_res.header().mtime().unwrap_or(0);
                entries.push(RawArchiveEntry {
                    name: path,
                    is_dir,
                    size,
                    modified_str: format_unix_timestamp(mtime),
                    modified_ts: mtime as i64,
                });
            }
        }
    } else {
        let mut archive = tar::Archive::new(file);
        if let Ok(tar_entries) = archive.entries() {
            for entry_res in tar_entries.flatten() {
                let path = entry_res.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                let is_dir = entry_res.header().entry_type().is_dir() || path.ends_with('/');
                let size = entry_res.header().size().unwrap_or(0);
                let mtime = entry_res.header().mtime().unwrap_or(0);
                entries.push(RawArchiveEntry {
                    name: path,
                    is_dir,
                    size,
                    modified_str: format_unix_timestamp(mtime),
                    modified_ts: mtime as i64,
                });
            }
        }
    }

    Ok(entries)
}

fn read_sevenz_raw_entries(archive_path: &Path) -> Result<Vec<RawArchiveEntry>, String> {
    let reader = sevenz_rust::SevenZReader::open(archive_path, sevenz_rust::Password::empty())
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for f in &reader.archive().files {
        entries.push(RawArchiveEntry {
            name: f.name().to_string(),
            is_dir: f.is_directory(),
            size: f.size(),
            modified_str: "-".to_string(),
            modified_ts: 0,
        });
    }

    Ok(entries)
}

fn read_rar_raw_entries(archive_path: &Path) -> Result<Vec<RawArchiveEntry>, String> {
    let path_str = archive_path.to_str().ok_or("Invalid path")?;
    let mut archive = unrar::Archive::new(path_str)
        .open_for_listing()
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    while let Ok(Some(header)) = archive.read_header() {
        entries.push(RawArchiveEntry {
            name: header.entry().filename.to_string_lossy().to_string(),
            is_dir: header.entry().is_directory(),
            size: header.entry().unpacked_size as u64,
            modified_str: "-".to_string(),
            modified_ts: 0,
        });
        archive = header.skip().map_err(|e| e.to_string())?;
    }

    Ok(entries)
}

fn format_unix_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "-".to_string();
    }
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        let time_val = ts as libc::time_t;
        libc::localtime_r(&time_val, &mut tm);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min
        )
    }
}

fn build_archive_directory_view(
    archive_path: &Path,
    inner_path: &str,
    raw_entries: Vec<RawArchiveEntry>,
    show_hidden: bool,
) -> Vec<FileEntry> {
    let mut result = Vec::new();
    let clean_inner = inner_path.trim_matches('/').to_string();
    let prefix = if clean_inner.is_empty() {
        String::new()
    } else {
        format!("{}/", clean_inner)
    };

    // 1. Add ".." entry
    let parent_path = if clean_inner.is_empty() {
        archive_path
            .parent()
            .unwrap_or(Path::new("/"))
            .to_string_lossy()
            .to_string()
    } else if let Some((parent_inner, _)) = clean_inner.rsplit_once('/') {
        format!("{}/{}", archive_path.to_string_lossy(), parent_inner)
    } else {
        archive_path.to_string_lossy().to_string()
    };

    result.push(FileEntry {
        name: "..".to_string(),
        path: parent_path,
        is_dir: true,
        is_symlink: false,
        size: 0,
        modified: String::new(),
        modified_ts: i64::MAX,
        created: None,
        permissions_str: "drwxr-xr-x".to_string(),
        octal_mode: 0o755,
        owner: "archive".to_string(),
        selected: false,
        space_marked: false,
        calculating_size: false,
        calculated_dir_size: None,
    });

    let mut subdirs: BTreeMap<String, (String, i64)> = BTreeMap::new();
    let mut files: Vec<FileEntry> = Vec::new();

    for entry in raw_entries {
        let raw = entry.name.trim_matches('/').to_string();
        if raw.is_empty() {
            continue;
        }

        let rel = if !prefix.is_empty() {
            if !raw.starts_with(&prefix) {
                continue;
            }
            &raw[prefix.len()..]
        } else {
            &raw
        };

        let clean_rel = rel.trim_matches('/');
        if clean_rel.is_empty() {
            continue;
        }

        if let Some((first_segment, _)) = clean_rel.split_once('/') {
            if !show_hidden && first_segment.starts_with('.') {
                continue;
            }
            subdirs
                .entry(first_segment.to_string())
                .or_insert_with(|| (entry.modified_str.clone(), entry.modified_ts));
        } else if entry.is_dir {
            if !show_hidden && clean_rel.starts_with('.') {
                continue;
            }
            subdirs
                .entry(clean_rel.to_string())
                .or_insert_with(|| (entry.modified_str.clone(), entry.modified_ts));
        } else {
            if !show_hidden && clean_rel.starts_with('.') {
                continue;
            }
            let item_path = if clean_inner.is_empty() {
                format!("{}/{}", archive_path.to_string_lossy(), clean_rel)
            } else {
                format!("{}/{}/{}", archive_path.to_string_lossy(), clean_inner, clean_rel)
            };

            files.push(FileEntry {
                name: clean_rel.to_string(),
                path: item_path,
                is_dir: false,
                is_symlink: false,
                size: entry.size,
                modified: entry.modified_str,
                modified_ts: entry.modified_ts,
                created: None,
                permissions_str: "-rw-r--r--".to_string(),
                octal_mode: 0o644,
                owner: "archive".to_string(),
                selected: false,
                space_marked: false,
                calculating_size: false,
                calculated_dir_size: None,
            });
        }
    }

    // Append directories first
    for (dir_name, (mod_str, mod_ts)) in subdirs {
        let dir_path = if clean_inner.is_empty() {
            format!("{}/{}", archive_path.to_string_lossy(), dir_name)
        } else {
            format!("{}/{}/{}", archive_path.to_string_lossy(), clean_inner, dir_name)
        };

        result.push(FileEntry {
            name: dir_name,
            path: dir_path,
            is_dir: true,
            is_symlink: false,
            size: 0,
            modified: mod_str,
            modified_ts: mod_ts,
            created: None,
            permissions_str: "drwxr-xr-x".to_string(),
            octal_mode: 0o755,
            owner: "archive".to_string(),
            selected: false,
            space_marked: false,
            calculating_size: false,
            calculated_dir_size: None,
        });
    }

    result.extend(files);
    result
}

/// Extracts a single file inside an archive to a temporary file on disk so it can be viewed / edited.
pub fn extract_single_file_to_temp(archive_path: &Path, inner_path: &str) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir().join(format!("ruster_preview_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let filename = Path::new(inner_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let temp_file = temp_dir.join(&*filename);

    let name_lower = archive_path.to_string_lossy().to_lowercase();
    if name_lower.ends_with(".zip") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let clean_inner = inner_path.trim_matches('/');
        let mut found = None;
        for i in 0..zip.len() {
            if let Ok(entry) = zip.by_index(i) {
                if entry.name().trim_matches('/') == clean_inner {
                    found = Some(i);
                    break;
                }
            }
        }
        let idx = found.ok_or_else(|| format!("File '{}' not found in zip", inner_path))?;
        let mut zip_entry = zip.by_index(idx).map_err(|e| e.to_string())?;
        let mut out = File::create(&temp_file).map_err(|e| e.to_string())?;
        io::copy(&mut zip_entry, &mut out).map_err(|e| e.to_string())?;
        Ok(temp_file)
    } else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut tar = tar::Archive::new(gz);
        let clean_inner = inner_path.trim_matches('/');
        for entry_res in tar.entries().map_err(|e| e.to_string())?.flatten() {
            let p = entry_res.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            if p.trim_matches('/') == clean_inner {
                let mut out = File::create(&temp_file).map_err(|e| e.to_string())?;
                let mut reader = entry_res;
                io::copy(&mut reader, &mut out).map_err(|e| e.to_string())?;
                return Ok(temp_file);
            }
        }
        Err(format!("File '{}' not found in tar.gz", inner_path))
    } else if name_lower.ends_with(".tar") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let mut tar = tar::Archive::new(file);
        let clean_inner = inner_path.trim_matches('/');
        for entry_res in tar.entries().map_err(|e| e.to_string())?.flatten() {
            let p = entry_res.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            if p.trim_matches('/') == clean_inner {
                let mut out = File::create(&temp_file).map_err(|e| e.to_string())?;
                let mut reader = entry_res;
                io::copy(&mut reader, &mut out).map_err(|e| e.to_string())?;
                return Ok(temp_file);
            }
        }
        Err(format!("File '{}' not found in tar", inner_path))
    } else if name_lower.ends_with(".gz") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let mut gz = flate2::read::GzDecoder::new(file);
        let mut out = File::create(&temp_file).map_err(|e| e.to_string())?;
        io::copy(&mut gz, &mut out).map_err(|e| e.to_string())?;
        Ok(temp_file)
    } else {
        // Fallback for 7z / rar: extract entire archive to temp subdirectory and return file path
        let sub_temp = temp_dir.join("extracted");
        extract_local(archive_path, &sub_temp)?;
        let full_path = sub_temp.join(inner_path.trim_matches('/'));
        if full_path.exists() {
            Ok(full_path)
        } else {
            Err(format!("File '{}' not found in archive", inner_path))
        }
    }
}

/// Extracts a specific entry (file or folder) from inside an archive to a destination directory.
pub fn extract_entry_to(archive_path: &Path, inner_path: &str, dest_dir: &Path) -> Result<(), String> {
    let clean_inner = inner_path.trim_matches('/');
    if clean_inner.is_empty() {
        return extract_local(archive_path, dest_dir);
    }

    let name_lower = archive_path.to_string_lossy().to_lowercase();
    if name_lower.ends_with(".gz") && !name_lower.ends_with(".tar.gz") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let mut gz = flate2::read::GzDecoder::new(file);
        let target_path = dest_dir.join(clean_inner);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = File::create(&target_path).map_err(|e| e.to_string())?;
        io::copy(&mut gz, &mut out).map_err(|e| e.to_string())?;
        return Ok(());
    }

    if name_lower.ends_with(".zip") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
            let entry_name = file.name().trim_matches('/').to_string();
            if entry_name == clean_inner || entry_name.starts_with(&format!("{}/", clean_inner)) {
                let rel_path: PathBuf = if entry_name == clean_inner {
                    PathBuf::from(Path::new(&clean_inner).file_name().unwrap_or_default())
                } else {
                    let suffix = &entry_name[clean_inner.len()..].trim_matches('/');
                    let base = Path::new(&clean_inner).file_name().unwrap_or_default();
                    Path::new(base).join(suffix)
                };

                let target_path = dest_dir.join(rel_path);
                if file.is_dir() || file.name().ends_with('/') {
                    fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
                } else {
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let mut out = File::create(&target_path).map_err(|e| e.to_string())?;
                    io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    } else {
        // Fallback for other formats: extract to temp and copy the requested entry
        let temp_dir = std::env::temp_dir().join(format!("ruster_extract_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        extract_local(archive_path, &temp_dir)?;
        let src = temp_dir.join(clean_inner);
        let dst = dest_dir.join(src.file_name().unwrap_or_default());
        if src.is_dir() {
            copy_dir_all(&src, &dst)?;
        } else if src.is_file() {
            fs::copy(&src, &dst).map_err(|e| e.to_string())?;
        }
        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_archive_file() {
        assert!(is_archive_file("test.zip"));
        assert!(is_archive_file("archive.tar.gz"));
        assert!(is_archive_file("package.tgz"));
        assert!(is_archive_file("bundle.7z"));
        assert!(is_archive_file("data.rar"));
        assert!(is_archive_file("file.txt.gz"));
        assert!(!is_archive_file("document.pdf"));
        assert!(!is_archive_file("image.png"));
    }

    #[test]
    fn test_zip_roundtrip() {
        let temp = std::env::temp_dir().join(format!("ruster_test_zip_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();

        let src_file = temp.join("hello.txt");
        fs::write(&src_file, "hello zip world").unwrap();

        let archive_zip = temp.join("archive.zip");
        compress_local(&[src_file], &archive_zip, ArchiveFormat::Zip, None).unwrap();
        assert!(archive_zip.exists());

        let extract_dir = temp.join("extracted_zip");
        extract_local(&archive_zip, &extract_dir).unwrap();

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(extracted_file).unwrap(), "hello zip world");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_tar_gz_roundtrip() {
        let temp = std::env::temp_dir().join(format!("ruster_test_targz_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();

        let src_file = temp.join("hello.txt");
        fs::write(&src_file, "hello tar.gz world").unwrap();

        let archive_tar = temp.join("archive.tar.gz");
        compress_local(&[src_file], &archive_tar, ArchiveFormat::TarGz, None).unwrap();
        assert!(archive_tar.exists());

        let extract_dir = temp.join("extracted_tar");
        extract_local(&archive_tar, &extract_dir).unwrap();

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(extracted_file).unwrap(), "hello tar.gz world");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_sevenz_roundtrip() {
        let temp = std::env::temp_dir().join(format!("ruster_test_7z_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();

        let src_file = temp.join("hello.txt");
        fs::write(&src_file, "hello 7z world").unwrap();

        let archive_7z = temp.join("archive.7z");
        compress_local(&[src_file], &archive_7z, ArchiveFormat::SevenZip, None).unwrap();
        assert!(archive_7z.exists());

        let extract_dir = temp.join("extracted_7z");
        extract_local(&archive_7z, &extract_dir).unwrap();

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(extracted_file).unwrap(), "hello 7z world");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_archive_vfs_browsing_and_extraction() {
        let temp = std::env::temp_dir().join(format!("ruster_test_vfs_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("folder")).unwrap();

        let sub_file = temp.join("folder").join("nested.txt");
        fs::write(&sub_file, "nested content").unwrap();

        let root_file = temp.join("root.txt");
        fs::write(&root_file, "root content").unwrap();

        let archive_zip = temp.join("test_vfs.zip");
        compress_local(&[temp.join("folder"), root_file], &archive_zip, ArchiveFormat::Zip, None).unwrap();
        assert!(archive_zip.exists());

        // 1. Test split_archive_path
        let (arch, inner) = split_archive_path(&archive_zip).expect("archive should be recognized");
        assert_eq!(arch, archive_zip);
        assert_eq!(inner, "");

        let inner_virtual_folder = archive_zip.join("folder");
        let (arch2, inner2) = split_archive_path(&inner_virtual_folder).expect("inner path should be recognized");
        assert_eq!(arch2, archive_zip);
        assert_eq!(inner2, "folder");

        let inner_virtual_file = archive_zip.join("folder").join("nested.txt");
        let (arch3, inner3) = split_archive_path(&inner_virtual_file).expect("inner file path should be recognized");
        assert_eq!(arch3, archive_zip);
        assert_eq!(inner3, "folder/nested.txt");

        let regular_dir = temp.join("folder");
        assert!(split_archive_path(&regular_dir).is_none());

        // 2. Test read_archive_directory at root
        let root_entries = read_archive_directory(&archive_zip, "", false);
        let names: Vec<&str> = root_entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&".."));
        assert!(names.contains(&"folder"));
        assert!(names.contains(&"root.txt"));

        // 3. Test read_archive_directory inside "folder"
        let folder_entries = read_archive_directory(&archive_zip, "folder", false);
        let folder_names: Vec<&str> = folder_entries.iter().map(|e| e.name.as_str()).collect();
        assert!(folder_names.contains(&".."));
        assert!(folder_names.contains(&"nested.txt"));

        // 4. Test extract_single_file_to_temp
        let temp_extracted = extract_single_file_to_temp(&archive_zip, "folder/nested.txt").unwrap();
        assert!(temp_extracted.exists());
        assert_eq!(fs::read_to_string(&temp_extracted).unwrap(), "nested content");

        // 5. Test extract_entry_to
        let target_dir = temp.join("extracted_target");
        extract_entry_to(&archive_zip, "folder", &target_dir).unwrap();
        assert!(target_dir.join("folder").join("nested.txt").exists());
        assert_eq!(fs::read_to_string(target_dir.join("folder").join("nested.txt")).unwrap(), "nested content");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_zip_encrypted_roundtrip() {
        let temp = std::env::temp_dir().join(format!("ruster_test_zip_enc_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();

        let src_file = temp.join("secret.txt");
        fs::write(&src_file, "secret payload 12345").unwrap();

        let archive_zip = temp.join("encrypted.zip");
        compress_local(&[src_file], &archive_zip, ArchiveFormat::Zip, Some("mypassword123")).unwrap();
        assert!(archive_zip.exists());

        let extract_dir = temp.join("extracted_enc");
        extract_local_with_password(&archive_zip, &extract_dir, Some("mypassword123")).unwrap();

        let extracted_file = extract_dir.join("secret.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(extracted_file).unwrap(), "secret payload 12345");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_sevenz_encrypted_roundtrip() {
        let temp = std::env::temp_dir().join(format!("ruster_test_7z_enc_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();

        let src_file = temp.join("secret_7z.txt");
        fs::write(&src_file, "secret 7z payload 98765").unwrap();

        let archive_7z = temp.join("encrypted.7z");
        compress_local(&[src_file], &archive_7z, ArchiveFormat::SevenZip, Some("pass7z_test")).unwrap();
        assert!(archive_7z.exists());

        let extract_dir = temp.join("extracted_7z_enc");
        extract_local_with_password(&archive_7z, &extract_dir, Some("pass7z_test")).unwrap();

        let extracted_file = extract_dir.join("secret_7z.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(extracted_file).unwrap(), "secret 7z payload 98765");

        let _ = fs::remove_dir_all(&temp);
    }
}
