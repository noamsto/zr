use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const DB_VERSION: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dir<'a> {
    pub path: Cow<'a, str>,
    pub rank: f64,
    pub last_accessed: i64,
}

#[derive(Debug)]
pub struct Database {
    pub dirs: Vec<Dir<'static>>,
    path: PathBuf,
}

pub struct RelocatedEntry {
    pub old_path: String,
    pub new_path: String,
    pub rank: f64,
}

impl Database {
    pub fn open() -> io::Result<Self> {
        let path = db_path()?;
        let bytes = fs::read(&path)?;
        let dirs = decode(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Database { dirs, path })
    }

    pub fn save(&self) -> io::Result<()> {
        let bytes = encode(&self.dirs).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let tmp = self.path.with_extension("zo.tmp");
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn matching_paths(&self, dir: &str) -> Vec<&Dir<'static>> {
        let norm = dir.trim_end_matches('/');
        let prefix = format!("{norm}/");
        self.dirs
            .iter()
            .filter(|d| d.path.as_ref() == norm || d.path.starts_with(&prefix))
            .collect()
    }

    pub fn relocate_paths(&mut self, old: &str, new: &str) -> Vec<RelocatedEntry> {
        let old_norm = old.trim_end_matches('/');
        let new_norm = new.trim_end_matches('/');
        let child_prefix = format!("{old_norm}/");

        let mut relocated = Vec::new();
        for dir in &mut self.dirs {
            let new_path = if dir.path.as_ref() == old_norm {
                new_norm.to_string()
            } else if dir.path.starts_with(&child_prefix) {
                format!("{new_norm}{}", &dir.path[old_norm.len()..])
            } else {
                continue;
            };

            let old_path = dir.path.to_string();
            let rank = dir.rank;
            dir.path = Cow::Owned(new_path.clone());
            relocated.push(RelocatedEntry {
                old_path,
                new_path,
                rank,
            });
        }
        relocated
    }

    #[allow(dead_code)]
    pub fn paths(&self) -> Vec<&str> {
        self.dirs.iter().map(|d| d.path.as_ref()).collect()
    }
}

fn db_path() -> io::Result<PathBuf> {
    if let Ok(dir) = std::env::var("_ZO_DATA_DIR") {
        return Ok(PathBuf::from(dir).join("db.zo"));
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir).join("zoxide/db.zo"));
    }
    let home = dirs_next().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home dir"))?;
    Ok(home.join(".local/share/zoxide/db.zo"))
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

macro_rules! bc {
    () => {{
        use bincode::Options;
        bincode::options().with_fixint_encoding()
    }};
}

fn decode(bytes: &[u8]) -> Result<Vec<Dir<'static>>, Box<bincode::ErrorKind>> {
    use bincode::Options;

    let version: u32 = bc!().allow_trailing_bytes().deserialize(bytes)?;
    if version != DB_VERSION {
        return Err(Box::new(bincode::ErrorKind::Custom(format!(
            "unsupported db version {version} (expected {DB_VERSION})"
        ))));
    }

    // Skip past the version (4 bytes for fixint u32).
    let dirs: Vec<Dir<'static>> = bc!().allow_trailing_bytes().deserialize(&bytes[4..])?;
    Ok(dirs)
}

fn encode(dirs: &[Dir<'static>]) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
    use bincode::Options;

    let mut buf = bc!().serialize(&DB_VERSION)?;
    buf.extend(bc!().serialize(dirs)?);
    Ok(buf)
}
