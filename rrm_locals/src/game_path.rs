use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GamePath(PathBuf);

impl GamePath {
    pub fn new() -> Self {
        Self(PathBuf::new())
    }

    fn create(path: &PathBuf) -> Result<Self, String> {
        let expanded_path: PathBuf = match path.starts_with("~") {
            true => path
                .display()
                .to_string()
                .replacen("~", &std::env::var("HOME").unwrap(), 1)
                .into(),
            false => path.to_owned(),
        };

        if !expanded_path.is_dir() {
            return Err(format!(
                "Path \"{}\" does not exist",
                expanded_path.display()
            ));
        };

        Ok(GamePath(expanded_path))
    }

    pub fn get_mod_path(&self) -> PathBuf {
        self.0.join("game/Mods")
    }
}

impl TryFrom<PathBuf> for GamePath {
    type Error = String;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        GamePath::create(&path)
    }
}

impl GamePath {
    pub fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl std::fmt::Display for GamePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
