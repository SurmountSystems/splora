//! Named file store for the nostr secret. This is not the popup and not a text field.

use std::fmt;
use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use nostr::key::Keys;
use nostr::nips::nip19::ToBech32;
use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "splora-named-keystore.json";

/// Failure while reading or writing the named file. The message never includes the secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStoreError {
    Io,
    Format,
    Npub,
}

/// File store. The caller chooses the path. The secret stays in that file.
pub struct SploraNamedKeyStore {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct FileBody {
    secret_hex: String,
}

impl fmt::Debug for SploraNamedKeyStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SploraNamedKeyStore")
            .field("path", &self.path)
            .finish()
    }
}

impl SploraNamedKeyStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `data_dir/splora-native/splora-named-keystore.json`, or the file name in the current directory.
    pub fn app_default_path() -> PathBuf {
        match dirs::data_dir() {
            Some(dir) => dir.join("splora-native").join(FILE_NAME),
            None => PathBuf::from(FILE_NAME),
        }
    }

    pub fn save_keys(&self, keys: &Keys) -> Result<(), KeyStoreError> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|_| KeyStoreError::Io)?;
        }
        let body = FileBody {
            secret_hex: keys.secret_key().to_secret_hex(),
        };
        let json = serde_json::to_vec(&body).map_err(|_| KeyStoreError::Format)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&self.path)
            .map_err(|_| KeyStoreError::Io)?;
        file.write_all(&json).map_err(|_| KeyStoreError::Io)?;
        Ok(())
    }

    pub fn load_keys(&self) -> Result<Keys, KeyStoreError> {
        let text = fs::read_to_string(&self.path).map_err(|_| KeyStoreError::Io)?;
        let body: FileBody = serde_json::from_str(&text).map_err(|_| KeyStoreError::Format)?;
        Keys::parse(&body.secret_hex).map_err(|_| KeyStoreError::Format)
    }

    /// Load the file, or generate a key and write it when the file is missing.
    pub fn load_or_generate(&self) -> Result<Keys, KeyStoreError> {
        if self.path.is_file() {
            self.load_keys()
        } else {
            let keys = Keys::generate();
            self.save_keys(&keys)?;
            Ok(keys)
        }
    }
}

/// Generate or load a key through the named store and return only the npub.
pub fn launch_npub() -> Result<String, KeyStoreError> {
    let store = SploraNamedKeyStore::new(SploraNamedKeyStore::app_default_path());
    let keys = store.load_or_generate()?;
    keys.public_key()
        .to_bech32()
        .map_err(|_| KeyStoreError::Npub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_store_round_trips_without_printing_the_secret() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "splora-native-keystore-{nanos}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(FILE_NAME);
        let store = SploraNamedKeyStore::new(&path);
        let keys = Keys::generate();
        store.save_keys(&keys).expect("save");
        let loaded = store.load_keys().expect("load");
        assert_eq!(loaded.public_key(), keys.public_key());
        let debug = format!("{store:?}");
        assert!(!debug.contains("nsec1"));
        assert!(!debug.contains(&keys.secret_key().to_secret_hex()));
        let raw = fs::read_to_string(&path).expect("file");
        assert!(!raw.contains("nsec1"));
        assert!(raw.contains("secret_hex"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
