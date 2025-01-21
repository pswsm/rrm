use crate::args::{Local, Steam};
use crate::search_in_steam;
use crate::utils::*;

#[macro_export]
macro_rules! display_search {
    ($m: expr, $args: expr, $i: expr) => {
        if !$m.is_empty() {
            if $args.display.pager || $i.use_pager && !$args.display.no_pager {
                $m.more_display(&$i.with_pager)
            } else {
                $m.display()
            }
        } else {
            println!("No results found")
        }
    };
}

pub fn search_locally(installer: Installer, args: Local) {
    use rrm_locals::Filtrable;

    let d_type = rrm_locals::DisplayType::from(args.display.large);
    let mods = GameMods::from(&installer.rimworld_path).with_display(d_type);

    let filtered = mods.filter_by(args.to_filter_obj(), &args.string);

    display_search!(filtered, args, installer);
}

pub async fn search_steam(i: Installer, args: Steam) {
    use rrm_scrap::Filtrable;

    let mods = SteamMods::search(&args.r#mod)
        .await
        .with_display(rrm_locals::DisplayType::from(args.display.large));

    let mods = search_in_steam!(args, mods);

    display_search!(mods, args, i);
}
