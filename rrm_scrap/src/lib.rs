pub use flagset::*;
use fuzzy_matcher::FuzzyMatcher;
use lazy_regex::*;
use reqwest::Response;
use rrm_locals::{DisplayType, InfoString};
use std::io::Stdout;
use std::io::Write;
use std::ops::Deref;
use std::process::{exit, Stdio};

#[cfg(test)]
mod test;

/// Capitalizes the first character of a string
fn capitalize(s: &str) -> String {
    let (ccc, bbb) = s.split_at(1);
    ccc.to_uppercase() + bbb
}

async fn get_contents(url: String) -> String {
    let resp: Response = reqwest::get(url).await.unwrap_or_else(|err| {
        eprintln!("{}", capitalize(&err.to_string()));
        exit(1);
    });

    let resp: String = resp.text().await.unwrap_or_else(|err| {
        eprintln!("{}", capitalize(&err.to_string()));
        exit(1);
    });

    resp
}

pub async fn look_for_mod(mod_name: &str) -> (Vec<ModSteamInfo>, usize) {
    use scraper::{Html, Selector};

    let contents: String = get_contents(
        r#"https://steamcommunity.com/workshop/browse/?appid=294100&searchtext="URL""#
            .replace("URL", mod_name),
    )
    .await;

    let contents: Html = Html::parse_document(&contents);
    let script: Selector =
        Selector::parse("#profileBlock > div > div.workshopBrowseItems > script").unwrap();

    let mut mods_steam_info: Vec<ModSteamInfo> = contents
        .select(&script)
        .map(|elem| single_decode_element(elem.inner_html()))
        .collect();

    let author: Selector = Selector::parse("#profileBlock > div > div.workshopBrowseItems > div > div.workshopItemAuthorName.ellipsis > a").unwrap();

    // What this size for?
    let mut size: usize = 0;

    for (i, element) in contents.select(&author).enumerate() {
        mods_steam_info[i].author = element.inner_html();
        if mods_steam_info[i].title.len() > size {
            size = mods_steam_info[i].title.len();
        }
    }

    (mods_steam_info, size)
}

/// From a <script> tag from steam's workshop page, gets the relevant info about said mod
fn single_decode_element(element_contents: String) -> ModSteamInfo {
    let re = regex::Regex::new(r"\{.{1,}\}").unwrap();
    let mm = re.find(element_contents.trim()).unwrap().as_str();

    serde_json::from_str::<ModSteamInfoRaw>(mm).unwrap().into()
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ModSteamInfo {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub author: String,
}

/// Raw struct from the script string
#[derive(serde::Serialize, serde::Deserialize)]
struct ModSteamInfoRaw {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default = "default_author")]
    pub author: String,
}

fn default_author() -> String {
    "".to_string()
}

impl From<ModSteamInfoRaw> for ModSteamInfo {
    fn from(value: ModSteamInfoRaw) -> Self {
        ModSteamInfo {
            id: value.id.parse().unwrap(),
            title: value.title,
            description: value.description,
            author: value.author,
        }
    }
}

impl ModSteamInfo {
    /// Generates the headers of the table with the {size} spacing
    fn gen_headers(size: usize) -> String {
        "".to_string()
            .add_s(format!("{:>15}", "Steam ID"))
            .add_s(format!("   {:<size$}", "Name", size = size))
            .add_s(format!("   {:<20}", "Uploader"))
            .add_s(format!("\n{:>15}", "--------"))
            .add_s(format!("   {:<size$}", "--------", size = size))
            .add_s(format!("   {:<20}", "--------"))
    }

    pub fn gen_large(&self) -> String {
        "".to_string()
            .add_s(format!("Name     : {}\n", self.title))
            .add_s(format!("Steam ID : {}\n", self.id))
            .add_s(format!("Author   : {}\n", self.author))
            .add_s(format!("Description: {}\n", self.description))
    }

    pub fn gen_short(&self, biggest_name: usize) -> String {
        "".to_string()
            .add_s(format!("{:>15}", self.id))
            .add_s(format!("   {:<size$}", self.title, size = biggest_name))
            .add_s(format!("   {:<20}", self.author))
    }

    pub fn display(&self, form: &DisplayType, biggest_name: usize) {
        let mut f: Stdout = std::io::stdout();

        if let DisplayType::Long = form {
            writeln!(f, "{}", self.gen_large()).unwrap()
        } else {
            writeln!(f, "{}", self.gen_short(biggest_name)).unwrap()
        }
    }

    pub fn gen_display(&self, form: &DisplayType, biggest_name: usize) -> String {
        let mut result = String::new();

        if let DisplayType::Long = form {
            result.push_str(&self.gen_large());
        } else {
            result.push_str(&self.gen_short(biggest_name));
        }

        result.push('\n');

        result
    }
}

#[derive(Default)]
pub struct SteamMods {
    pub mods: Vec<ModSteamInfo>,
    pub biggest_name_size: usize,
    pub display_type: Option<DisplayType>,
}

impl SteamMods {
    pub fn new() -> Self {
        SteamMods::default()
    }

    pub async fn search(m: &str) -> Self {
        let (mods, biggest_name_size) = look_for_mod(m).await;

        SteamMods {
            mods,
            biggest_name_size,
            display_type: None,
        }
    }

    pub fn with_display(self, t: DisplayType) -> Self {
        let mut s = self;
        s.display_type = Some(t);
        s
    }

    pub fn with_raw_display(self, t: Option<DisplayType>) -> Self {
        let mut s = self;
        s.display_type = t;
        s
    }

    pub fn gen_display(&self) -> String {
        let mut result = "".to_string();

        if let Some(typ) = self.display_type {
            if let DisplayType::Short = typ {
                result.push_str(&ModSteamInfo::gen_headers(self.biggest_name_size));
                result.push('\n')
            }

            self.mods.iter().for_each(|m| {
                result.push_str(&m.gen_display(&typ, self.biggest_name_size));
            });
        } else {
            result.push_str(&ModSteamInfo::gen_headers(self.biggest_name_size));
            result = result.replace("       Steam ID", "             Steam ID");
            result = result.replace('\n', "\n      ");
            result.push('\n');

            self.mods.iter().enumerate().for_each(|(i, m)| {
                result.push_str(&format!(
                    " {:<4} {}",
                    i,
                    &m.gen_display(&DisplayType::Short, self.biggest_name_size)
                ));
            });
        }

        result
    }

    pub fn more_display(&self, with_pager: &str) {
        let output = self.gen_display();

        let mut more = std::process::Command::new(with_pager)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let more_stdin = more.stdin.as_mut().unwrap();
        more_stdin
            .write_all(output.as_bytes())
            .unwrap_or_else(|err| {
                eprintln!(
                    "Something went wrong while writing contents to `more`.\n\
            Error: {err}"
                )
            });

        more.wait().unwrap();
    }

    pub fn display(&self) {
        print!("{}", self.gen_display())
    }

    pub fn display_numbered(&self) {
        print!("{}", self.gen_display())
    }
}

impl Deref for SteamMods {
    type Target = Vec<ModSteamInfo>;

    fn deref(&self) -> &Self::Target {
        &self.mods
    }
}

flags! {
    pub enum FilterBy: u8 {
        SteamID     = 0b00001,
        Title       = 0b00010,
        Description = 0b00100,
        Author      = 0b01000,
        None        = 0b10000,
        All = (FilterBy::SteamID | FilterBy::Title | FilterBy::Description | FilterBy::Author).bits(),
    }
}

pub trait Filtrable<T: flagset::Flags>: Sized {
    fn filter_by(&self, filter: FlagSet<T>, value: &str) -> Self;
}

impl Filtrable<FilterBy> for SteamMods {
    fn filter_by(&self, filter: FlagSet<FilterBy>, value: &str) -> Self {
        use FilterBy::*;

        let mut filtered = SteamMods::new();
        let mods: Vec<ModSteamInfo> = self.mods.clone();

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        filtered.display_type = self.display_type;

        mods.into_iter().for_each(|m| {
            let result = {
                (if filter.contains(All) || filter.contains(Title) {
                    matcher.fuzzy_match(&m.title, value).is_some()
                } else {
                    false
                }) || (if filter.contains(Author) || filter.contains(All) {
                    matcher.fuzzy_match(&m.author, value).is_some()
                } else {
                    false
                }) || (if filter.contains(Description) || filter.contains(All) {
                    matcher.fuzzy_match(&m.author, value).is_some()
                } else {
                    false
                }) || (if filter.contains(SteamID) || filter.contains(All) {
                    matcher.fuzzy_match(&m.id.to_string(), value).is_some()
                } else {
                    false
                })
            };

            if result {
                if m.title.len() > filtered.biggest_name_size {
                    filtered.biggest_name_size = m.title.len();
                }

                filtered.mods.push(m);
            };
        });

        filtered
    }
}
