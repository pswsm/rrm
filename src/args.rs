use crate::utils::*;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct App {
    #[clap(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
#[clap(override_help = "\
rrm-set
Set new configuration values

USAGE:
    rrm set <OPTION> <VALUE>

OPTIONS:
    game-path    Set the path where RimWorld is installed [alias: 'path']
    pager        Set the paging software to use, like bat, more or less [alias: 'paging']
    use-pager    Set if rrm should use more to display output [values: false, true, 0, 1] [alias: 'use-paging']
")]
pub enum Options {
    #[clap(
        about = "Set if rrm should use paging software to display output [values: false, true, 0, 1]",
        visible_alias = "use-paging"
    )]
    UsePager {
        #[clap(required = true, value_parser(["true", "false", "0", "1"]))]
        value: String,
    },

    #[clap(
        about = "Set the path where RimWorld is installed",
        visible_alias = "path"
    )]
    GamePath {
        /// The path where RimWorld is installed
        #[clap(required = true)]
        value: PathBuf,
    },

    #[clap(
        about = "Set the paging software to use, like bat, more or less",
        visible_alias = "paging"
    )]
    Pager {
        #[cfg(target_os = "windows")]
        /// The path where the paging software is, for example: C:\Windows\System32\more.com
        #[clap(required = true)]
        value: PathBuf,

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        /// The name of the paging software, for example: bat, more
        #[clap(required = true)]
        value: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(visible_alias = "i", about = "Install a RimWorld Mod by name or ID")]
    Install {
        #[clap(flatten)]
        args: InstallCommandGroup,
    },

    #[clap(about = "Install everything again (except ignored by default)")]
    Pull {
        #[clap(flatten)]
        args: Pull,
        #[clap(short, long, visible_alias = "also-ignored")]
        ignored: bool,
    },

    #[clap(
        visible_alias = "ss",
        hide = true,
        about = "Search for mods in Steam",
        override_usage = "rrm search steam <MOD>"
    )]
    SearchSteam {
        #[clap(flatten)]
        args: Steam,
    },

    #[clap(
        visible_alias = "sl",
        hide = true,
        disable_version_flag = true,
        about = "Search for mods locally, where RimWorld is installed [with no flags searches by name]",
        override_usage = "rrm search local [OPTIONS] <STRING>"
    )]
    SearchLocally {
        #[clap(flatten)]
        args: Local,
    },

    #[clap(about = "Set new configuration values")]
    Set {
        #[clap(subcommand)]
        command: Options,
    },

    #[clap(visible_alias = "s", about = "Search for mods locally or in Steam")]
    Search {
        #[clap(subcommand)]
        command: Search,
    },

    #[clap(
        visible_alias = "l",
        about = LIST_DESCRIPTION
    )]
    List {
        #[clap(flatten)]
        display: DisplayOptions,
    },
    Completions {
        #[clap(value_parser(["bash", "fish", "zsh", "powershell", "elvish"]))]
        shell: String,
    },
}

#[derive(Args, Debug)]
pub struct DisplayOptions {
    /// Display the larger message
    #[clap(long)]
    pub large: bool,

    /// Force rrm to use paging software to display the output.
    #[clap(long)]
    pub pager: bool,

    /// Force rrm not to use paging software to display the output.
    #[clap(long)]
    pub no_pager: bool,
}

#[derive(Subcommand, Debug)]
pub enum Search {
    #[clap(
        visible_aliases = &["s", "ss (global)"],
        about = "Search for mods in Steam",
    )]
    Steam {
        #[clap(flatten)]
        args: Steam,
    },

    #[clap(
        visible_aliases = &["l", "sl (global)"],
        disable_version_flag = true,
        about = "Search for mods locally, where RimWorld is installed [with no flags searches by name]",
    )]
    Local {
        #[clap(flatten)]
        args: Local,
    },
}

#[derive(Args, Debug)]
#[clap(arg_required_else_help = true)]
pub struct Steam {
    #[clap(flatten)]
    pub display: DisplayOptions,

    /// The name of the RimWorld mod
    #[clap(required = true)]
    pub(crate) r#mod: String,

    /// The name of the RimWorld mod
    #[clap(short, long, required = false)]
    pub(crate) filter: Option<Option<String>>,

    /// Search by author(s) name(s)
    #[clap(short, long, requires = "filter")]
    pub(crate) author: bool,

    /// Search by version
    #[clap(short, long, requires = "filter")]
    pub(crate) version: bool,

    /// Search by Steam ID
    #[clap(short, long, requires = "filter")]
    pub(crate) steam_id: bool,

    /// Search by mod name
    #[clap(short, long, requires = "filter")]
    pub(crate) name: bool,

    /// Search by all fields
    #[clap(long, conflicts_with_all = &["authors", "version", "steam-id", "name"], requires="filter")]
    pub(crate) all: bool,
}

#[derive(Args, Debug)]
#[clap(arg_required_else_help = true)]
pub struct Local {
    #[clap(flatten)]
    pub display: DisplayOptions,

    /// The pattern to search
    #[clap(required = true)]
    pub(crate) r#string: String,

    /// Search by author(s) name(s)
    #[clap(short, long)]
    pub(crate) author: bool,

    /// Search by version
    #[clap(short, long)]
    pub(crate) version: bool,

    /// Search by Steam ID
    #[clap(short, long)]
    pub(crate) steam_id: bool,

    /// Search by mod name
    #[clap(short, long)]
    pub(crate) name: bool,

    /// Search by all fields
    #[clap(long, conflicts_with_all = &["authors", "version", "steam-id", "name"])]
    pub(crate) all: bool,
}

#[derive(Args, Debug, Clone)]
#[clap(arg_required_else_help = true)]
pub struct Pull {
    /// Automatic dependencies installation
    #[clap(long, short, visible_alias = "resolve-dependencies")]
    pub(crate) resolve: bool,

    /// Show more information about the process [alias: vvv]
    #[clap(long, visible_alias = "vvv")]
    pub(crate) verbose: bool,

    /// Show even more information. Expect a lot of output
    #[clap(long)]
    pub(crate) debug: bool,
}

#[derive(Args, Debug, Clone)]
#[clap(arg_required_else_help = false)]
pub struct InstallCommandGroup {
    /// The name of the RimWorld mod(s)
    #[clap(required = true)]
    pub(crate) rimmod: Vec<String>,

    /// The name of the RimWorld mod
    #[clap(short, long, required = false)]
    pub(crate) filter: Option<Option<String>>,

    /// Search by author(s) name(s)
    #[clap(short, long, requires = "filter")]
    pub(crate) author: bool,

    /// Search by version
    #[clap(short, long, requires = "filter")]
    pub(crate) version: bool,

    /// Search by Steam ID
    #[clap(short, long, requires = "filter")]
    pub(crate) steam_id: bool,

    /// Search by mod name
    #[clap(short, long, requires = "filter")]
    pub(crate) name: bool,

    /// Search by all fields
    #[clap(long, conflicts_with_all = &["author", "version", "steam_id", "name"], requires="filter")]
    pub(crate) all: bool,

    /// Yes to all questions
    #[clap(long, short)]
    pub(crate) yes: bool,

    /// Automatic dependencies installation
    #[clap(long, short, visible_alias = "resolve-dependencies")]
    pub(crate) resolve: bool,

    /// Show more information about the process [alias: vvv]
    #[clap(long, visible_alias = "vvv")]
    pub(crate) verbose: bool,

    /// Show even more information. Expect a lot of output
    #[clap(long)]
    pub(crate) debug: bool,
}

macro_rules! a_if {
    ($cond: expr, $add: expr) => {
        if $cond {
            $add
        } else {
            rrm_locals::FilterBy::None
        }
    };
}

macro_rules! b_if {
    ($cond: expr, $add: expr) => {
        if $cond {
            $add
        } else {
            rrm_scrap::FilterBy::None
        }
    };
}

impl Local {
    pub fn to_filter_obj(&self) -> rrm_locals::FlagSet<rrm_locals::FilterBy> {
        let mut result: rrm_locals::FlagSet<rrm_locals::FilterBy> =
            rrm_locals::FlagSet::from(rrm_locals::FilterBy::None);

        if self.all {
            return rrm_locals::FlagSet::from(rrm_locals::FilterBy::All);
        }

        result |= a_if!(self.name, rrm_locals::FilterBy::Name);
        result |= a_if!(self.author, rrm_locals::FilterBy::Author);
        result |= a_if!(self.version, rrm_locals::FilterBy::Version);
        result |= a_if!(self.steam_id, rrm_locals::FilterBy::SteamID);

        result -= rrm_locals::FilterBy::None;

        if result.is_empty() {
            result |= rrm_locals::FilterBy::Name;
        }

        result
    }
}

macro_rules! filter {
    ($s: expr) => {{
        let mut result: rrm_scrap::FlagSet<rrm_scrap::FilterBy> =
            rrm_scrap::FlagSet::from(rrm_scrap::FilterBy::None);

        if $s.all {
            return rrm_scrap::FlagSet::from(rrm_scrap::FilterBy::All);
        }

        result |= b_if!($s.name, rrm_scrap::FilterBy::Title);
        result |= b_if!($s.author, rrm_scrap::FilterBy::Author);
        result |= b_if!($s.version, rrm_scrap::FilterBy::Description);
        result |= b_if!($s.steam_id, rrm_scrap::FilterBy::SteamID);

        result -= rrm_scrap::FilterBy::None;

        if result.is_empty() {
            result |= rrm_scrap::FilterBy::Title;
        }

        result
    }};
}

impl Steam {
    pub fn to_filter_obj(&self) -> rrm_scrap::FlagSet<rrm_scrap::FilterBy> {
        filter!(self)
    }
}

impl InstallCommandGroup {
    pub fn to_filter_obj(&self) -> rrm_scrap::FlagSet<rrm_scrap::FilterBy> {
        filter!(self)
    }
}

impl App {
    pub fn load() -> App {
        App::parse()
    }
}

pub trait InstallingOptions {
    fn is_verbose(&self) -> bool;
    fn is_debug(&self) -> bool;
}

macro_rules! impl_io {
    ($s: ty) => {
        impl InstallingOptions for $s {
            fn is_verbose(&self) -> bool {
                self.verbose || self.debug
            }

            fn is_debug(&self) -> bool {
                self.debug
            }
        }
    };
}

impl_io!(Pull);
impl_io!(InstallCommandGroup);
