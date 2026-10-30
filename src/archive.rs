use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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
) -> Result<(), String> {
    if sources.is_empty() {
        return Err("No files or folders selected to compress".to_string());
    }

    match format {
        ArchiveFormat::Zip => compress_zip(sources, target_archive),
        ArchiveFormat::TarGz => compress_tar_gz(sources, target_archive),
        ArchiveFormat::SevenZip => compress_sevenz(sources, target_archive),
    }
}

fn compress_zip(sources: &[PathBuf], target_archive: &Path) -> Result<(), String> {
    let file = File::create(target_archive).map_err(|e| format!("Failed to create archive: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

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
    options: zip::write::SimpleFileOptions,
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
    options: zip::write::SimpleFileOptions,
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

fn compress_sevenz(sources: &[PathBuf], target_archive: &Path) -> Result<(), String> {
    if sources.len() == 1 {
        sevenz_rust::compress_to_path(&sources[0], target_archive)
            .map_err(|e| format!("7-Zip compression failed: {}", e))?;
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

    let res = sevenz_rust::compress_to_path(&temp_dir, target_archive)
        .map_err(|e| format!("7-Zip compression failed: {}", e));
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
    if !archive_path.exists() {
        return Err("Archive file does not exist".to_string());
    }
    fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create destination: {}", e))?;

    let name = archive_path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();

    if name.ends_with(".zip") {
        extract_zip(archive_path, dest_dir)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir)
    } else if name.ends_with(".tar") {
        extract_tar(archive_path, dest_dir)
    } else if name.ends_with(".7z") {
        sevenz_rust::decompress_file(archive_path, dest_dir)
            .map_err(|e| format!("Failed to extract 7z: {}", e))
    } else if name.ends_with(".rar") {
        extract_rar(archive_path, dest_dir)
    } else if name.ends_with(".gz") {
        extract_single_gz(archive_path, dest_dir)
    } else {
        Err(format!("Unsupported archive format: {}", name))
    }
}

fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Zip entry error: {}", e))?;
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
        compress_local(&[src_file], &archive_zip, ArchiveFormat::Zip).unwrap();
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
        compress_local(&[src_file], &archive_tar, ArchiveFormat::TarGz).unwrap();
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
        compress_local(&[src_file], &archive_7z, ArchiveFormat::SevenZip).unwrap();
        assert!(archive_7z.exists());

        let extract_dir = temp.join("extracted_7z");
        extract_local(&archive_7z, &extract_dir).unwrap();

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(extracted_file).unwrap(), "hello 7z world");

        let _ = fs::remove_dir_all(&temp);
    }
}
