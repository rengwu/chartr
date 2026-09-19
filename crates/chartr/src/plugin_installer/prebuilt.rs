//! Fetch ready-to-run native packages. No compiler, shell, install hook or
//! package executable is ever invoked. Web assets are already ready to run.
use super::*;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

const MAX_DOWNLOAD: u64 = 1024 * 1024 * 1024;
const MAX_UNPACKED: u64 = 4 * 1024 * 1024 * 1024;

pub(super) fn resolve(package: &Path, cancel: &AtomicBool) -> Result<()> {
    let source = Manifest::read(package)?;
    if source.kind != Kind::Embedded {
        return Ok(());
    }
    let library = chartr_plugin_host::embedded_library(&source)?;
    safe_path(Path::new(library))?;
    if package.join(library).is_file() {
        return Ok(());
    }
    let release = source.release.as_deref().context(
        "This plugin has no prebuilt package for this platform. Installation never compiles source.")?;
    let base = url::Url::parse(release)?;
    if base.scheme() != "https"
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        bail!("Prebuilt plugin releases must use an HTTPS directory URL");
    }
    let archive = format!("chartr-plugin-{}.tar.gz", chartr_plugin_host::platform_key());
    let base = release.trim_end_matches('/');
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(Duration::from_secs(600)))
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_body(Some(Duration::from_secs(30)))
        .build()
        .into();
    check_cancelled(cancel)?;
    let checksum = agent.get(format!("{base}/{archive}.sha256")).call()
        .context("No downloadable build is available for this platform; installation never compiles source")?
        .body_mut().with_config().limit(1024).read_to_string()?;
    let checksum = parse_checksum(&checksum, &archive)?;
    let mut response =
        agent.get(format!("{base}/{archive}")).call().context("Downloading the prebuilt plugin")?;
    let mut file = tempfile::tempfile_in(package.parent().context("staging directory")?)?;
    download(&mut response.body_mut().as_reader(), &mut file, &checksum, cancel)?;
    use std::io::Seek;
    file.rewind()?;
    let unpacked = package.with_file_name("prebuilt");
    fs::create_dir(&unpacked)?;
    unpack(file, &unpacked, cancel)?;
    let downloaded = Manifest::read(&unpacked)?;
    if downloaded.kind != Kind::Embedded || downloaded.id != source.id {
        bail!("The downloaded package does not match the requested plugin");
    }
    validate_package(&unpacked, &downloaded)?;
    fs::remove_dir_all(package)?;
    fs::rename(unpacked, package)?;
    Ok(())
}

fn parse_checksum(text: &str, archive: &str) -> Result<String> {
    let fields: Vec<_> = text.split_whitespace().collect();
    if fields.len() != 2
        || fields[0].len() != 64
        || !fields[0].bytes().all(|b| b.is_ascii_hexdigit())
        || fields[1].trim_start_matches('*') != archive
    {
        bail!("Invalid checksum record for the prebuilt plugin");
    }
    Ok(fields[0].to_ascii_lowercase())
}

fn download(
    reader: &mut impl Read,
    writer: &mut impl Write,
    expected: &str,
    cancel: &AtomicBool,
) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check_cancelled(cancel)?;
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > MAX_DOWNLOAD {
            bail!("Prebuilt package exceeds the 1 GiB download limit");
        }
        hasher.update(&buffer[..count]);
        writer.write_all(&buffer[..count])?;
    }
    if format!("{:x}", hasher.finalize()) != expected {
        bail!("Prebuilt plugin checksum does not match; nothing was installed");
    }
    Ok(())
}

fn safe_path(path: &Path) -> Result<()> {
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir))
    {
        bail!("Prebuilt package contains an escaping path");
    }
    Ok(())
}

fn unpack(reader: impl Read, destination: &Path, cancel: &AtomicBool) -> Result<()> {
    let decoder = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(decoder);
    let mut total = 0u64;
    let mut files = 0u32;
    for entry in archive.entries()? {
        check_cancelled(cancel)?;
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        safe_path(&path)?;
        files += 1;
        total = total.checked_add(entry.size()).context("Package size overflow")?;
        if total > MAX_UNPACKED || files > 10000 {
            bail!("Prebuilt package exceeds the extraction limit");
        }
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            bail!("Prebuilt packages may only contain regular files and directories");
        }
        let target = destination.join(path);
        if kind.is_dir() {
            fs::create_dir_all(target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mode = entry.header().mode()?;
        // create_new rejects duplicate paths and file/directory collisions.
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&target)?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            check_cancelled(cancel)?;
            let count = entry.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            file.write_all(&buffer[..count])?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                target,
                fs::Permissions::from_mode(if mode & 0o111 != 0 { 0o755 } else { 0o644 }),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checksum_rejects_tampered_content_and_wrong_artifacts() {
        let content = b"prebuilt binary";
        let digest = format!("{:x}", Sha256::digest(content));
        let record = format!("{digest}  plugin.tar.gz\n");
        assert_eq!(parse_checksum(&record, "plugin.tar.gz").unwrap(), digest);
        assert!(parse_checksum(&record, "wrong.tar.gz").is_err());
        let cancel = AtomicBool::new(false);
        assert!(download(&mut &content[..], &mut Vec::new(), &digest, &cancel).is_ok());
        assert!(download(&mut &b"tampered"[..], &mut Vec::new(), &digest, &cancel).is_err());
        cancel.store(true, Ordering::Relaxed);
        assert!(download(&mut &content[..], &mut Vec::new(), &digest, &cancel).is_err());
    }
    #[test]
    fn archive_rejects_links_and_strips_privileged_permissions() {
        use flate2::{Compression, write::GzEncoder};
        for link in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let encoder = GzEncoder::new(Vec::new(), Compression::default());
            let mut archive = tar::Builder::new(encoder);
            let mut header = tar::Header::new_gnu();
            header.set_path("helper").unwrap();
            header.set_mode(0o6755);
            header.set_size(if link { 0 } else { 4 });
            if link {
                header.set_entry_type(tar::EntryType::Symlink);
                header.set_link_name("/etc/passwd").unwrap();
            }
            header.set_cksum();
            archive.append(&header, if link { &b""[..] } else { &b"data"[..] }).unwrap();
            let bytes = archive.into_inner().unwrap().finish().unwrap();
            let result = unpack(&bytes[..], dir.path(), &AtomicBool::new(false));
            if link {
                assert!(result.is_err());
            } else {
                result.unwrap();
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(dir.path().join("helper")).unwrap().permissions().mode() & 0o7777,
                    0o755
                );
            }
        }
        for path in ["../escape", "/absolute", "a/../../escape"] {
            assert!(safe_path(Path::new(path)).is_err());
        }
    }
}
