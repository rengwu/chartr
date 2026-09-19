//! The persisted list of folders chartr calls spaces.
//!
//! This is a model below GPUI: folder picking belongs to the window, while
//! validation and persistence are testable without one. Its registry lives
//! under the `chartr` configuration namespace.

use std::{
    ffi::OsString,
    fmt, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub const SPACES_FILE: &str = "spaces.toml";

const HEADER: &str = "\
# chartr's registered spaces, in sidebar order. Every folder here is one the
# operator added; nothing authoritative lives in this file, so deleting it costs
# re-adding the folders and nothing else.
";

pub fn spaces_file() -> Result<PathBuf, Error> {
    Ok(config_root()?.join(SPACES_FILE))
}

pub(crate) fn config_root() -> Result<PathBuf, Error> {
    config_root_from(std::env::var_os("XDG_CONFIG_HOME"), std::env::home_dir())
}

fn config_root_from(xdg: Option<OsString>, home: Option<PathBuf>) -> Result<PathBuf, Error> {
    if let Some(xdg) = xdg.filter(|xdg| Path::new(xdg).is_absolute()) {
        return Ok(PathBuf::from(xdg).join("chartr"));
    }
    home.filter(|home| !home.as_os_str().is_empty())
        .map(|home| home.join(".config/chartr"))
        .ok_or(Error::NoConfigRoot)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Space {
    path: PathBuf,
    name: String,
    // Unknown keys belong to older or newer chartr versions. Carry them
    // through so this rewrite never eats another version's state.
    extra: toml::Table,
}

impl Space {
    fn new(path: PathBuf, name: Option<String>, extra: toml::Table) -> Self {
        let name =
            name.filter(|name| !name.trim().is_empty()).unwrap_or_else(|| display_name(&path));
        Self { path, name, extra }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

#[derive(Debug, Clone)]
pub struct Registry {
    file: PathBuf,
    spaces: Vec<Space>,
    extra: toml::Table,
}

impl Registry {
    pub fn load(file: impl Into<PathBuf>) -> Result<Self, Error> {
        let file = file.into();
        let text = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(Self { file, spaces: Vec::new(), extra: toml::Table::new() });
            }
            Err(source) => return Err(Error::io(file, "reading", source)),
        };

        let document: Document = toml::from_str(&text)
            .map_err(|source| Error::Malformed { path: file.clone(), source })?;
        Ok(Self { spaces: seat(&file, document.spaces)?, extra: document.extra, file })
    }

    pub fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    /// Register a folder, appending it to file/sidebar order.
    ///
    /// Re-registering is selection, not duplication, and therefore performs no
    /// write. A failed write rolls the in-memory row back.
    pub fn register(&mut self, path: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let path = absolute(path.as_ref())?;
        recordable(&path)?;
        let metadata = fs::metadata(&path)
            .map_err(|source| Error::NotAFolder { path: path.clone(), source: Some(source) })?;
        if !metadata.is_dir() {
            return Err(Error::NotAFolder { path, source: None });
        }
        if self.spaces.iter().any(|space| same_path(&space.path, &path)) {
            return Ok(path);
        }

        self.spaces.push(Space::new(path.clone(), None, toml::Table::new()));
        if let Err(error) = self.save() {
            self.spaces.pop();
            return Err(error);
        }
        Ok(path)
    }

    /// Forget a registered folder without touching the folder itself.
    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<bool, Error> {
        let Some(index) =
            self.spaces.iter().position(|space| same_path(space.path(), path.as_ref()))
        else {
            return Ok(false);
        };
        let removed = self.spaces.remove(index);
        if let Err(error) = self.save() {
            self.spaces.insert(index, removed);
            return Err(error);
        }
        Ok(true)
    }

    pub fn rename(&mut self, path: impl AsRef<Path>, name: String) -> Result<(), Error> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::BadName);
        }
        let Some(index) =
            self.spaces.iter().position(|space| same_path(space.path(), path.as_ref()))
        else {
            return Ok(());
        };
        let old = std::mem::replace(&mut self.spaces[index].name, name.to_owned());
        if let Err(error) = self.save() {
            self.spaces[index].name = old;
            return Err(error);
        }
        Ok(())
    }

    /// Replaces the registered-space order with a complete path permutation.
    ///
    /// The candidate is validated before the in-memory registry changes. A
    /// failed write restores the previous order, so the sidebar can reject a
    /// drop without ever presenting an arrangement the next launch would lose.
    pub fn reorder(&mut self, paths: &[PathBuf]) -> Result<bool, Error> {
        if paths.len() != self.spaces.len() {
            return Err(Error::BadReorder);
        }
        let mut remaining = self.spaces.clone();
        let mut candidate = Vec::with_capacity(remaining.len());
        for path in paths {
            let Some(index) = remaining.iter().position(|space| same_path(space.path(), path))
            else {
                return Err(Error::BadReorder);
            };
            candidate.push(remaining.remove(index));
        }
        if !remaining.is_empty() {
            return Err(Error::BadReorder);
        }
        if candidate == self.spaces {
            return Ok(false);
        }

        let previous = std::mem::replace(&mut self.spaces, candidate);
        if let Err(error) = self.save() {
            self.spaces = previous;
            return Err(error);
        }
        Ok(true)
    }

    pub fn relocate(
        &mut self,
        old_path: impl AsRef<Path>,
        new_path: impl AsRef<Path>,
    ) -> Result<PathBuf, Error> {
        let new_path = absolute(new_path.as_ref())?;
        recordable(&new_path)?;
        if !new_path.is_dir() {
            return Err(Error::NotAFolder { path: new_path, source: None });
        }
        let Some(index) =
            self.spaces.iter().position(|space| same_path(space.path(), old_path.as_ref()))
        else {
            return Ok(new_path);
        };
        if self
            .spaces
            .iter()
            .enumerate()
            .any(|(candidate, space)| candidate != index && same_path(space.path(), &new_path))
        {
            return Err(Error::DuplicateFolder(new_path));
        }
        let old = std::mem::replace(&mut self.spaces[index].path, new_path.clone());
        if let Err(error) = self.save() {
            self.spaces[index].path = old;
            return Err(error);
        }
        Ok(new_path)
    }

    fn save(&self) -> Result<(), Error> {
        let parent = self.file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .map_err(|source| Error::io(parent.to_path_buf(), "creating", source))?;

        let document = Document {
            spaces: self
                .spaces
                .iter()
                .map(|space| {
                    Ok(Record {
                        path: recordable(&space.path)?.to_owned(),
                        name: (space.name != display_name(&space.path)).then(|| space.name.clone()),
                        extra: space.extra.clone(),
                    })
                })
                .collect::<Result<_, Error>>()?,
            extra: self.extra.clone(),
        };
        let body = toml::to_string(&document).map_err(Error::Encode)?;

        // The old implementation stages beside the destination and atomically
        // persists it. Keep that exact transaction boundary here.
        let mut staged = tempfile::NamedTempFile::new_in(parent)
            .map_err(|source| Error::io(self.file.clone(), "staging", source))?;
        staged
            .write_all(format!("{HEADER}\n{body}").as_bytes())
            .and_then(|_| staged.flush())
            .map_err(|source| Error::io(self.file.clone(), "staging", source))?;
        staged
            .persist(&self.file)
            .map_err(|error| Error::io(self.file.clone(), "replacing", error.error))?;
        Ok(())
    }
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    resolved(a) == resolved(b)
}

fn resolved(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_owned())
}

fn absolute(path: &Path) -> Result<PathBuf, Error> {
    std::path::absolute(path)
        .map_err(|source| Error::NotAFolder { path: path.to_path_buf(), source: Some(source) })
}

fn recordable(path: &Path) -> Result<&str, Error> {
    path.to_str().ok_or_else(|| Error::NotUnicode { path: path.to_path_buf() })
}

/// Load file order, with the one migration supported by the old registry:
/// legacy integer `order` keys sort first and are then removed.
fn seat(file: &Path, records: Vec<Record>) -> Result<Vec<Space>, Error> {
    let mut records: Vec<(i64, Record)> = records
        .into_iter()
        .map(|mut record| {
            let order = match record.extra.remove("order") {
                Some(toml::Value::Integer(order)) => order,
                Some(other) => {
                    record.extra.insert("order".into(), other);
                    i64::MAX
                }
                None => i64::MAX,
            };
            (order, record)
        })
        .collect();
    records.sort_by_key(|(order, _)| *order);

    let mut spaces: Vec<Space> = Vec::with_capacity(records.len());
    for (_, record) in records {
        let path = PathBuf::from(&record.path);
        if !path.is_absolute() {
            return Err(Error::NotAbsolute { file: file.to_path_buf(), path: record.path });
        }
        if spaces.iter().all(|space| !same_path(&space.path, &path)) {
            spaces.push(Space::new(path, record.name, record.extra));
        }
    }
    Ok(spaces)
}

#[derive(Debug, Deserialize, Serialize)]
struct Document {
    #[serde(default, rename = "space")]
    spaces: Vec<Record>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(flatten)]
    extra: toml::Table,
}

#[derive(Debug)]
pub enum Error {
    Io { path: PathBuf, action: &'static str, source: io::Error },
    Malformed { path: PathBuf, source: toml::de::Error },
    NotAbsolute { file: PathBuf, path: String },
    Encode(toml::ser::Error),
    NotAFolder { path: PathBuf, source: Option<io::Error> },
    NotUnicode { path: PathBuf },
    NoConfigRoot,
    BadName,
    DuplicateFolder(PathBuf),
    BadReorder,
}

impl Error {
    fn io(path: PathBuf, action: &'static str, source: io::Error) -> Self {
        Self::Io { path, action, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, action, source } => {
                write!(f, "{action} {}: {source}", path.display())
            }
            Self::Malformed { path, .. } => {
                write!(f, "{} is not a space registry chartr can read", path.display())
            }
            Self::NotAbsolute { file, path } => write!(
                f,
                "{} names the relative path {path:?}; every space is an absolute path",
                file.display()
            ),
            Self::Encode(source) => write!(f, "encoding the space registry: {source}"),
            Self::NotAFolder { path, source: Some(source) } => {
                write!(f, "{} is not a folder chartr can register: {source}", path.display())
            }
            Self::NotAFolder { path, source: None } => {
                write!(f, "{} is a file, not a folder", path.display())
            }
            Self::NotUnicode { path } => write!(
                f,
                "{} is not a name the registry file can hold: it is not Unicode",
                path.display()
            ),
            Self::NoConfigRoot => write!(
                f,
                "neither XDG_CONFIG_HOME nor a home directory is set, so there is nowhere for {SPACES_FILE} to live"
            ),
            Self::BadName => write!(f, "a space name cannot be empty"),
            Self::DuplicateFolder(path) => {
                write!(f, "{} is already registered as another space", path.display())
            }
            Self::BadReorder => {
                write!(f, "a space reorder must name every registered folder exactly once")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Malformed { source, .. } => Some(source),
            Self::Encode(source) => Some(source),
            Self::NotAFolder { source, .. } => {
                source.as_ref().map(|source| source as &(dyn std::error::Error + 'static))
            }
            Self::NotAbsolute { .. }
            | Self::NotUnicode { .. }
            | Self::NoConfigRoot
            | Self::BadName
            | Self::DuplicateFolder(_)
            | Self::BadReorder => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_root_uses_the_chartr_namespace() {
        assert_eq!(
            config_root_from(Some("/xdg".into()), Some("/home/op".into())).unwrap(),
            PathBuf::from("/xdg/chartr")
        );
        assert_eq!(
            config_root_from(None, Some("/home/op".into())).unwrap(),
            PathBuf::from("/home/op/.config/chartr")
        );
    }

    #[test]
    fn registration_round_trips_and_deduplicates() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("project");
        fs::create_dir(&folder).unwrap();
        let file = temp.path().join("spaces.toml");
        let mut registry = Registry::load(&file).unwrap();

        registry.register(&folder).unwrap();
        registry.register(&folder).unwrap();

        let loaded = Registry::load(file).unwrap();
        assert_eq!(loaded.spaces().len(), 1);
        assert_eq!(loaded.spaces()[0].name(), "project");
    }

    #[test]
    fn removing_a_space_only_changes_the_registry() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("project");
        fs::create_dir(&folder).unwrap();
        let file = temp.path().join("spaces.toml");
        let mut registry = Registry::load(&file).unwrap();
        registry.register(&folder).unwrap();

        assert!(registry.remove(&folder).unwrap());
        assert!(folder.is_dir(), "the project folder is not registry data");
        assert!(Registry::load(file).unwrap().spaces().is_empty());
    }

    #[test]
    fn legacy_order_is_honoured_once() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("spaces.toml");
        fs::write(
            &file,
            "[[space]]\npath = \"/second\"\norder = 2\n\n[[space]]\npath = \"/first\"\norder = 1\n",
        )
        .unwrap();
        let registry = Registry::load(file).unwrap();
        assert_eq!(registry.spaces()[0].name(), "first");
        assert_eq!(registry.spaces()[1].name(), "second");
    }

    #[test]
    fn keys_owned_by_other_chartr_versions_survive_a_write() {
        let temp = tempfile::tempdir().unwrap();
        let existing = temp.path().join("existing");
        let added = temp.path().join("added");
        fs::create_dir(&existing).unwrap();
        fs::create_dir(&added).unwrap();
        let file = temp.path().join("spaces.toml");
        fs::write(
            &file,
            format!(
                "future_top = \"kept\"\n\n[[space]]\npath = {:?}\nfuture_row = 42\n",
                existing.to_string_lossy()
            ),
        )
        .unwrap();

        let mut registry = Registry::load(&file).unwrap();
        registry.register(added).unwrap();
        let written = fs::read_to_string(file).unwrap();

        assert!(written.contains("future_top = \"kept\""));
        assert!(written.contains("future_row = 42"));
    }

    #[test]
    fn reordered_spaces_survive_relaunch_in_file_order() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        let third = temp.path().join("third");
        for folder in [&first, &second, &third] {
            fs::create_dir(folder).unwrap();
        }
        let file = temp.path().join("spaces.toml");
        let mut registry = Registry::load(&file).unwrap();
        for folder in [&first, &second, &third] {
            registry.register(folder).unwrap();
        }

        assert!(registry.reorder(&[third.clone(), first.clone(), second.clone()]).unwrap());
        let relaunched = Registry::load(file).unwrap();
        let order: Vec<_> = relaunched.spaces().iter().map(|space| space.path()).collect();
        assert_eq!(order, vec![&third, &first, &second]);
    }

    #[test]
    fn invalid_reorders_are_rejected_without_mutating_the_registry() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let mut registry = Registry::load(temp.path().join("spaces.toml")).unwrap();
        registry.register(&first).unwrap();
        registry.register(&second).unwrap();

        assert!(matches!(registry.reorder(std::slice::from_ref(&first)), Err(Error::BadReorder)));
        assert!(matches!(
            registry.reorder(&[first.clone(), first.clone()]),
            Err(Error::BadReorder)
        ));
        assert_eq!(registry.spaces()[0].path(), &first);
        assert_eq!(registry.spaces()[1].path(), &second);
    }

    #[test]
    fn no_op_avoids_io_and_a_failed_write_rolls_back_memory() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let config = temp.path().join("config");
        let file = config.join("spaces.toml");
        let mut registry = Registry::load(&file).unwrap();
        registry.register(&first).unwrap();
        registry.register(&second).unwrap();

        // Leave the loaded registry pointing at a path whose parent is now a
        // plain file. This reliably fails staging on every platform without
        // relying on permission behavior under a privileged test runner.
        fs::rename(&config, temp.path().join("moved-config")).unwrap();
        fs::write(&config, "not a directory").unwrap();

        assert!(!registry.reorder(&[first.clone(), second.clone()]).unwrap());
        assert!(registry.reorder(&[second, first.clone()]).is_err());
        assert_eq!(registry.spaces()[0].path(), &first);
    }
}
