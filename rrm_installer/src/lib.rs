use directories::UserDirs;
use include_dir::{include_dir, Dir};
use rrm_locals::GamePath;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::unix::fs::PermissionsExt;

#[cfg(target_os = "windows")]
static DEFAULT_PAGING_SOFTWARE: &str = r"C:\Windows\System32\more.com";

#[cfg(target_os = "macos")]
static PROJECT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/steamcmd/macos");

#[cfg(target_os = "windows")]
static PROJECT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/steamcmd/windows");

#[cfg(target_os = "linux")]
static PROJECT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/steamcmd/linux");

#[cfg(any(target_os = "linux", target_os = "macos"))]
const DEFAULT_PAGING_SOFTWARE: &str = r"less";

const CONFIG_FILE_NAME: &str = "config.json";

pub enum InstallerErrorCode {
    ConfigFileNotAvailable,
    ConfigParseError,
    WriteConfigError,
}

impl std::fmt::Display for InstallerErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            Self::ConfigFileNotAvailable => "ConfigNotAvailable".to_string(),
            Self::ConfigParseError => "ConfigParseError".to_string(),
            Self::WriteConfigError => "WriteConfigError".to_string(),
        };
        write!(f, "{}", x)
    }
}

pub struct InstallerError {
    pub code: InstallerErrorCode,
    pub message: String,
}

impl InstallerError {
    fn new(code: InstallerErrorCode, message: String) -> Self {
        Self { code, message }
    }
}

impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Installer {
    pub rimworld_path: GamePath,
    pub steamcmd_path: PathBuf,
    pub use_pager: bool,
    pub with_pager: String,
}

impl Installer {
    pub fn try_from_config() -> Result<Self, InstallerError> {
        println!("Reading from config...");
        let cfg_file: std::fs::File = match Self::get_config_file() {
            Ok(f) => f,
            Err(r) => {
                return Err(InstallerError::new(
                    InstallerErrorCode::ConfigFileNotAvailable,
                    r,
                ))
            }
        };

        let reader = std::io::BufReader::new(cfg_file);

        match serde_json::de::from_reader(reader) {
            Ok(installer) => Ok(installer),
            Err(whatever) => Err(InstallerError::new(
                InstallerErrorCode::ConfigParseError,
                whatever.to_string(),
            )),
        }
    }

    pub fn new_empty() -> Self {
        Installer {
            rimworld_path: GamePath::new(),
            steamcmd_path: PathBuf::new(),
            use_pager: false,
            with_pager: String::new(),
        }
    }

    pub fn with_rimworld_path(mut self, rimworld_path: GamePath) {
        self.rimworld_path = rimworld_path;
    }

    pub fn with_steamcmd_path(mut self, steamcmd_path: PathBuf) {
        self.steamcmd_path = steamcmd_path;
    }

    pub fn with_pager(mut self, pager: String) {
        self.with_pager = pager;
    }

    pub fn enable_pager(mut self) {
        self.use_pager = true;
    }

    pub fn disable_pager(mut self) {
        self.use_pager = false;
    }

    fn get_config_path() -> PathBuf {
        let config_dir = match std::env::var("XDG_CONFIG_HOME") {
            Err(_) => PathBuf::from("/etc/rrm"),
            Ok(shtr) => PathBuf::from(shtr),
        };
        config_dir.join("rrm")
    }

    fn get_config_file() -> Result<std::fs::File, String> {
        let config_path: PathBuf = Self::get_config_path();
        match std::fs::File::open(config_path.join(CONFIG_FILE_NAME)) {
            Ok(file) => Ok(file),
            Err(_reason) => Ok(Self::create_config_file()?),
        }
    }

    fn create_config_file() -> Result<std::fs::File, String> {
        let config_path: PathBuf = Self::get_config_path();
        match std::fs::File::create(config_path.join(CONFIG_FILE_NAME)) {
            Ok(f) => Ok(f),
            Err(_whatever) => {
                let msg = format!(
                    "Could not create configuration at {}",
                    config_path.display()
                );
                Err(msg)
            }
        }
    }

    pub fn init(game_path: PathBuf) -> Self {
        let config_dir = get_or_create_config_dir();
        let config_file = config_dir.join("config");
        if !config_file.exists() {
            std::fs::File::create(&config_file).unwrap();

            let steamcmd_path = config_dir.join("steamcmd");
            std::fs::create_dir(&steamcmd_path).unwrap_or_else(|err| {
                if !steamcmd_path.is_dir() {
                    panic!("{}", err);
                }
            });

            // TODO: Check if it's already installed first
            // TODO: Install from https://steamcdn-a.akamaihd.net/client/installer/steamcmd
            PROJECT_DIR.extract(steamcmd_path.as_path()).unwrap();

            #[cfg(any(target_os = "macos", target_os = "linux"))]
            set_permissions_for_steamcmd(steamcmd_path.as_path());

            println!("Installing steamcmd...");
            run_steam_command("", &config_dir, 1);
            println!("Done!");
        }

        std::env::set_current_dir(&config_dir).unwrap();

        if !game_path.exists() {
            return Installer {
                rimworld_path: GamePath::new(),
                steamcmd_path: PathBuf::new(),
                with_pager: DEFAULT_PAGING_SOFTWARE.to_string(),
                use_pager: true,
            };
        };

        let path = match GamePath::try_from(game_path) {
            Ok(p) => p,
            Err(_) => std::process::exit(1),
        };

        Installer {
            rimworld_path: path,
            steamcmd_path: PathBuf::new(),
            with_pager: DEFAULT_PAGING_SOFTWARE.to_string(),
            use_pager: true,
        }
    }

    pub fn try_write_config(&self) -> Result<(), InstallerError> {
        let json = match serde_json::to_string_pretty(self) {
            Ok(json) => json,
            Err(reason) => {
                return Err(InstallerError::new(
                    InstallerErrorCode::WriteConfigError,
                    reason.to_string(),
                ))
            }
        };

        let mut config_file: std::fs::File = match Self::get_config_file() {
            Ok(file) => file,
            Err(reason) => {
                return Err(InstallerError::new(
                    InstallerErrorCode::WriteConfigError,
                    reason,
                ))
            }
        };

        match config_file.write_all(json.as_bytes()) {
            Ok(_) => Ok(()),
            Err(reason) => Err(InstallerError::new(
                InstallerErrorCode::WriteConfigError,
                reason.to_string(),
            )),
        }
    }

    pub fn install_sync(&self, c: Vec<rrm_scrap::ModSteamInfo>) -> (bool, String) {
        let to_install = Self::gen_install_string(&c);
        let a: String = run_steam_command(&to_install, &get_or_create_config_dir(), 1);
        (a.contains("Success. Downloaded item"), a)
    }

    pub fn gen_install_string(c: &[rrm_scrap::ModSteamInfo]) -> String {
        "+workshop_download_item 294100 ".to_string()
            + &c.iter()
                .map(|rimmod| rimmod.id.to_string())
                .collect::<Vec<String>>()
                .join(" +workshop_download_item 294100 ")
    }

    pub fn get_steamcmd_path(&self) -> PathBuf {
        get_steamcmd_path(&get_or_create_config_dir())
    }

    pub fn run_steam_command(&self, c: &str, count: usize) -> String {
        run_steam_command(c, &get_or_create_config_dir(), count)
    }
}

pub fn get_or_create_config_dir() -> PathBuf {
    if let Some(path) = env_var_config("XDG_CONFIG_HOME")
        .or_else(|| env_var_config("RRM_CONFIG_HOME"))
        .or_else(|| env_var_config("CONFIG_HOME"))
    {
        return path;
    }

    let config_dir = UserDirs::new().unwrap().home_dir().join(".config");
    if config_dir.exists() {
        let config_dir = config_dir.join("rrm");
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).unwrap();
        }
        return config_dir.to_path_buf();
    }

    let config_dir = UserDirs::new().unwrap().home_dir().join(".rrm");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap();
    }

    println!("{:?}", &config_dir);
    config_dir.to_path_buf()
}

fn env_var_config(var: &'static str) -> Option<PathBuf> {
    std::env::var(var).ok().map(|env_config_dir| {
        let env_config_dir = PathBuf::from(env_config_dir).join("rrm");
        if !env_config_dir.exists() {
            std::fs::create_dir_all(&env_config_dir).unwrap();
        }
        env_config_dir
    })
}

pub fn run_steam_command(c: &str, config_path: &Path, count: usize) -> String {
    let steam = get_steamcmd_path(config_path);

    #[cfg(target_os = "windows")]
    let out = std::process::Command::new(steam.as_path().to_str().unwrap())
        .args("+login anonymous {} +quit".replace("{}", c).split(" "))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .unwrap_or_else(|error| {
            eprintln!("Could not execute steamcmd successfully.\nError: {}", error);
            exit(1);
        });

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let out = std::process::Command::new("env")
        .args(
            r#"HOME=PATH [] +login anonymous {} +quit"#
                .replace(
                    "PATH",
                    config_path.as_os_str().to_str().unwrap(),
                )
                .replace("[]", steam.as_path().to_str().unwrap())
                .replace("{}", c)
                .split(' '),
        )
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .unwrap_or_else(|error| {
            eprintln!("Could not execute steamcmd successfully.\nError: {}", error);
            exit(1);
        });

    let out = String::from_utf8(out.clone().stdout).unwrap();

    if c.contains("+workshop_download_item 294100")
        && out.contains("Connecting anonymously to Steam Public...OK")
        && out.contains("Waiting for client config...OK")
        && out.contains("Waiting for user info...OK")
    {
        out
    } else if c.contains("+workshop_download_item 294100") {
        run_steam_command(c, config_path, count + 1)
    } else if count == 5 {
        "Error: Failed to install".to_string()
    } else {
        run_steam_command(c, config_path, count + 1)
    }
}

pub fn get_steamcmd_path(config_path: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    return config_path.join("steamcmd").join("steamcmd");

    #[cfg(target_os = "linux")]
    return config_path.join("steamcmd").join("steamcmd.sh");

    #[cfg(target_os = "windows")]
    return config_path.join("steamcmd").join("steamcmd.exe");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn set_permissions_for_steamcmd(path: &Path) {
    let files = path.read_dir().unwrap();

    for file in files {
        let file = file.unwrap();

        if !file.file_type().unwrap().is_dir() {
            let mut perms = std::fs::metadata(file.path()).unwrap().permissions();
            perms.set_mode(0o744);
            std::fs::set_permissions(file.path(), perms).unwrap();
        } else {
            set_permissions_for_steamcmd(&file.path());
        }
    }
}
