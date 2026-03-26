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

fn is_match(path: &str, dir: &str) -> bool {
    path == dir || (path.starts_with(dir) && path.as_bytes().get(dir.len()) == Some(&b'/'))
}

/// Caller must ensure `is_match(path, old_prefix)` is true.
pub fn rewrite_path(path: &str, old_prefix: &str, new_prefix: &str) -> String {
    if path == old_prefix {
        new_prefix.to_string()
    } else {
        format!("{new_prefix}{}", &path[old_prefix.len()..])
    }
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
        self.dirs
            .iter()
            .filter(|d| is_match(&d.path, norm))
            .collect()
    }

    pub fn relocate_paths(&mut self, old: &str, new: &str) -> Vec<RelocatedEntry> {
        let old_norm = old.trim_end_matches('/');
        let new_norm = new.trim_end_matches('/');

        let mut relocated = Vec::new();
        for dir in &mut self.dirs {
            if !is_match(&dir.path, old_norm) {
                continue;
            }

            let new_path = rewrite_path(&dir.path, old_norm, new_norm);
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
}

fn db_path() -> io::Result<PathBuf> {
    if let Ok(dir) = std::env::var("_ZO_DATA_DIR") {
        return Ok(PathBuf::from(dir).join("db.zo"));
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir).join("zoxide/db.zo"));
    }
    let home = home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home dir"))?;
    Ok(home.join(".local/share/zoxide/db.zo"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

macro_rules! bc {
    () => {
        bincode::options().with_fixint_encoding()
    };
}

fn decode(bytes: &[u8]) -> Result<Vec<Dir<'static>>, Box<bincode::ErrorKind>> {
    use bincode::Options;

    if bytes.len() < 4 {
        return Err(Box::new(bincode::ErrorKind::Custom(
            "database too small".into(),
        )));
    }

    let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if version != DB_VERSION {
        return Err(Box::new(bincode::ErrorKind::Custom(format!(
            "unsupported db version {version} (expected {DB_VERSION})"
        ))));
    }

    let dirs: Vec<Dir<'static>> = bc!().allow_trailing_bytes().deserialize(&bytes[4..])?;
    Ok(dirs)
}

fn encode(dirs: &[Dir<'static>]) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
    use bincode::Options;

    let dir_bytes = bc!().serialize(dirs)?;
    let mut buf = Vec::with_capacity(4 + dir_bytes.len());
    buf.extend_from_slice(&DB_VERSION.to_le_bytes());
    buf.extend(dir_bytes);
    Ok(buf)
}
