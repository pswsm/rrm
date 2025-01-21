use crate::args::DisplayOptions;
use crate::utils::*;
use rrm_installer::Installer;

pub fn list(i: Installer, d: DisplayOptions) {
    let mods: GameMods = GameMods::from(&i.rimworld_path).with_display(DisplayType::from(d.large));

    if !mods.is_empty() {
        if d.pager || i.use_pager && !d.no_pager {
            mods.more_display(&i.with_pager);
        } else {
            mods.display();
        }
    } else {
        println!("No results found")
    }
}
