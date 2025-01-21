pub use crate::log;
use regex::Regex;
pub use rrm_installer::Installer;
pub use rrm_locals::{DisplayType, GameMods};
pub use rrm_scrap::SteamMods;
pub use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
pub const RW_DEFAULT_PATH: [&str; 2] = [
    r"/Applications/RimWorld.app/",
    r"~/Library/Application Support/Steam/steamapps/common/RimWorld/RimWorldMac.app/",
];

// looks like rust does not expand `~` to $HOME
// TODO: Fix
#[cfg(target_os = "linux")]
pub const RW_DEFAULT_PATH: [&str; 4] = [
    r"~/GOG Games/RimWorld/",
    r"~/.steam/steam/SteamApps/common/",
    r"~/.local/share/Steam/steamapps/common/RimWorld/",
    r"~/Games/RimWorld/",
];
#[cfg(target_os = "windows")]
pub const RW_DEFAULT_PATH: [&str; 2] = [
    r"C:\Program Files (x86)\Steam\steamapps\common\RimWorld",
    r"C:\Program Files\Steam\steamapps\common\RimWorld",
];

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const LIST_DESCRIPTION: &str = "List installed Mods in Path/To/RimWorld/Mods/";
#[cfg(target_os = "windows")]
pub const LIST_DESCRIPTION: &str = r#"List installed Mods in C:\Path\To\RimWorld\Mods"#;

#[macro_export]
macro_rules! printf {
    ( $($t:tt)* ) => {
        {
            use std::io::Write;
            let mut h = std::io::stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}

#[macro_export]
macro_rules! search_in_steam {
    ($args: expr, $mods: expr) => {{
        if $args.filter.is_some() {
            let value = if $args.filter.as_ref().unwrap().is_some() {
                $args.filter.as_ref().unwrap().clone().unwrap()
            } else {
                $args.r#mod.clone()
            };

            $mods.filter_by($args.to_filter_obj(), &value)
        } else {
            $mods
        }
    }};
}

pub fn extract_id(m: &str, reg: &Regex) -> Option<String> {
    if let Some(caps) = reg.captures(m) {
        if caps.len() == 0 {
            println!("Invalid id: {m}");
            return None;
        }

        Some(caps["id"].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::extract_id;
    use regex::Regex;

    #[test]
    fn test_extract_id() {
        let re = Regex::new(r"[a-zA-Z/:.]+\?id=(?P<id>\d+).*").unwrap();
        let ids = r"
        
https://steamcommunity.com/sharedfiles/filedetails/?id=432423434&?4234=
https://steamcommunity.com/sharedfiles/filedetails/?id=4234532
https://steamcommunity.com/sharedfiles/filedetails/?3423423



";
        let ids: Vec<&str> = ids.split('\n').collect();

        for id in ids {
            println!(
                "{}",
                extract_id(id, &re).unwrap_or_else(|| "Invalid".to_string())
            );
        }
    }
}
