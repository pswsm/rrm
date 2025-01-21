#[derive(Debug, PartialEq, Eq)]
pub struct UnknownPackageId;
#[derive(Debug, PartialEq, Eq)]
pub struct KnownPackageId(PackageId);

#[derive(Debug, PartialEq, Eq)]
pub struct PackageId {
    author: String,
    name_path: Vec<String>,
}

impl PackageId {
    fn parse(from: &str) -> Self {
        let (author, name_paths) = from.split_once('.').unwrap();
        let name_path = name_paths
            .split('.')
            .map(|path_part| path_part.to_string())
            .collect();
        Self {
            author: author.to_string(),
            name_path,
        }
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.author, self.name_path.join("."))
    }
}

#[cfg(test)]
mod package_id_tests {
    use super::PackageId;

    #[test]
    fn parse_test() {
        let name_path = vec!["VanillaFactionsExpanded".to_string(), "Core".to_string()];
        let parse_target = PackageId {
            author: "OskarPotocki".to_string(),
            name_path,
        };

        let parsed = PackageId::parse("OskarPotocki.VanillaFactionsExpanded.Core");
        assert_eq!(parsed, parse_target);
    }

    #[test]
    fn display_test() {
        let name_path = vec!["VanillaFactionsExpanded".to_string(), "Core".to_string()];
        let package_id = PackageId {
            author: "OskarPotocki".to_string(),
            name_path,
        };

        assert_eq!(
            package_id.to_string(),
            "OskarPotocki.VanillaFactionsExpanded.Core"
        );
    }
}

pub struct RimworldMod<P> {
    id: P,
    steam_id: usize,
    name: String,
    author: String,
    description: String,
    versions: Vec<String>,
}

impl From<rrm_scrap::ModSteamInfo> for RimworldMod<UnknownPackageId> {
    fn from(value: rrm_scrap::ModSteamInfo) -> Self {
        RimworldMod {
            id: UnknownPackageId,
            steam_id: value.id,
            name: value.title,
            author: value.author,
            description: value.description,
            versions: Vec::new(),
        }
    }
}

impl RimworldMod<UnknownPackageId> {
    fn add_id(self, id: PackageId) -> RimworldMod<KnownPackageId> {
        RimworldMod {
            id: KnownPackageId(id),
            steam_id: self.steam_id,
            name: self.name,
            author: self.author,
            description: self.description,
            versions: self.versions,
        }
    }
}
