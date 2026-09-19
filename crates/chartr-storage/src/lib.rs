//! Atomic file replacement for application-owned settings and plugin data.
//! Validation, conflict detection, and project-path authorization stay with callers.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// A fully written candidate beside its destination. Dropping it aborts the write.
/// Existing permissions are retained; new files use tempfile's private permissions.
pub struct StagedWrite {
    path: PathBuf,
    file: tempfile::NamedTempFile,
}

impl StagedWrite {
    /// Does not create directories: project writers can require a validated parent.
    pub fn new(path: &Path, bytes: &[u8]) -> io::Result<Self> {
        let mut file = tempfile::NamedTempFile::new_in(parent(path))?;
        match fs::metadata(path) {
            Ok(metadata) => file.as_file().set_permissions(metadata.permissions())?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        file.write_all(bytes)?;
        // All callers get the same durability policy before the atomic rename.
        file.as_file().sync_all()?;
        Ok(Self { path: path.into(), file })
    }

    pub fn replace(self) -> io::Result<()> {
        self.file.persist(self.path).map_err(|error| error.error)?;
        Ok(())
    }

    /// Publish a new file without overwriting a concurrently created destination.
    pub fn create_new(self) -> io::Result<()> {
        self.file.persist_noclobber(self.path).map_err(|error| error.error)?;
        Ok(())
    }
}

fn parent(path: &Path) -> &Path {
    path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."))
}

/// Create private-data directories, sync the candidate, then replace the destination.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    fs::create_dir_all(parent(path))?;
    StagedWrite::new(path, bytes)?.replace()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abandoned_and_failed_writes_preserve_the_previous_data() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        write_atomic(&path, b"old").unwrap();
        drop(StagedWrite::new(&path, b"abandoned").unwrap());
        assert_eq!(fs::read(&path).unwrap(), b"old");
        let staged = StagedWrite::new(&path, b"new").unwrap();
        assert!(staged.create_new().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"old");
        let staged = StagedWrite::new(&path, b"new").unwrap();
        fs::rename(&path, temp.path().join("backup")).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(staged.replace().is_err());
        assert_eq!(fs::read(temp.path().join("backup")).unwrap(), b"old");
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn readers_observe_complete_versions_and_new_file_creation_never_clobbers() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("new/data");
        write_atomic(&path, b"old").unwrap();
        let mut old_reader = fs::File::open(&path).unwrap();
        write_atomic(&path, b"replacement").unwrap();
        let mut old = Vec::new();
        std::io::Read::read_to_end(&mut old_reader, &mut old).unwrap();
        assert_eq!(old, b"old");
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        let new_path = temp.path().join("new/created");
        let staged = StagedWrite::new(&new_path, b"ours").unwrap();
        fs::write(&new_path, b"theirs").unwrap();
        assert!(staged.create_new().is_err());
        assert_eq!(fs::read(new_path).unwrap(), b"theirs");
    }

    #[cfg(unix)]
    #[test]
    fn replacement_preserves_existing_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        write_atomic(&path, b"new").unwrap();
        assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o640);
    }
}
