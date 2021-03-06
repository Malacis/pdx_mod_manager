//! Interface and filesystem functionality concering `Mod`s.

use anyhow::Result;
use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect};
use std::{fs, io::Cursor};

use crate::{filesystem::write_mod, Mod};

use super::Interface;

impl Interface {
    /// Function to add mods.
    #[async_recursion]
    pub async fn add_mod(&mut self) -> Result<()> {
        let game = self
            .config
            .games
            .get_mut(self.selection.expect("game selection is none"))
            .expect("get game failed");
        let item_id = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Type in the id of the mod!")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.find(char::is_numeric).is_some() {
                    Ok(())
                } else {
                    println!("{}", input);
                    Err("Please only type in numbers!")
                }
            })
            .interact_text()
            .expect("dialoguer error")
            .parse::<u64>()
            .expect("could not parse string to u64");

        let (item_title, item_time_updated) = self.remote.get_item_info(item_id).await?;
        let proceed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to download and install {} for {}?",
                item_title, game.title
            ))
            .interact()?;

        if !proceed {
            return self.show_game_options().await;
        }

        let file = self.remote.download_item(item_id).await?;
        println!("Download finished!");
        println!("### Installing ###");
        let zip = zip::ZipArchive::new(Cursor::new(file))?;
        write_mod(item_id, item_title.clone(), zip, &game.path_mods)?;

        println!("Updating config file.");
        if let Some(item) = game.mods.get_mut(&item_id.to_string()) {
            item.time_updated = item_time_updated;
        } else {
            let _old = game.mods.insert(
                item_id.to_string(),
                Mod {
                    id: item_id,
                    title: item_title,
                    time_updated: item_time_updated,
                },
            );
        }

        self.config.update_config_file()?;
        println!("Mod installed!.");

        self.show_game_options().await
    }

    /// Deletes mods.
    #[async_recursion]
    pub async fn delete_mods(&mut self) -> Result<()> {
        let game = self
            .config
            .games
            .get_mut(self.selection.expect("game selection is none"))
            .expect("get game failed");

        let mods = &mut game.mods;
        let mut items = vec![];
        let mut keys = vec![];
        for (key, modif) in mods.clone() {
            items.push(modif.title.clone());
            keys.push(key);
        }

        if items.is_empty() {
            println!("You have no mods installed for that game!");
            return self.show_game_options().await;
        }

        let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select with space, confirm with enter. Confirming without selecting anything will cancel.")
            .items(&items)
            .interact()?;

        if chosen.is_empty() {
            return self.show_game_options().await;
        }

        for index in chosen {
            fs::remove_dir_all(format!(
                "{}/{}",
                game.path_mods,
                keys.get(index).expect("could not find key")
            ))?;
            fs::remove_file(format!(
                "{}/{}{}",
                game.path_mods,
                keys.get(index).expect("could not find key"),
                ".mod"
            ))?;
            let _old = mods
                .remove(keys.get(index).expect("could not find key"))
                .expect("could not remove mod from config");
        }

        self.config.update_config_file()?;

        println!("Mods removed!");
        self.show_game_options().await
    }

    /// updates mod
    pub async fn update_mod(&mut self, item_id: u64, game_selection: usize) -> Result<()> {
        let game = self
            .config
            .games
            .get_mut(game_selection)
            .expect("get game failed");

        let modif = game
            .mods
            .get_mut(&item_id.to_string())
            .expect("get mod failed");

        let (item_title, item_time_updated) = self.remote.get_item_info(item_id).await?;

        if modif.time_updated >= item_time_updated {
            println!(
                "Mod {} for {} is already up to date!",
                modif.title, game.title
            );
            return Ok(());
        }

        println!("Updating mod {} for {}!", modif.title, game.title);

        let file = self.remote.download_item(item_id).await?;

        let zip = zip::ZipArchive::new(Cursor::new(file))?;

        write_mod(item_id, item_title.clone(), zip, &game.path_mods)?;

        println!("Updating config file.");
        modif.time_updated = item_time_updated;

        println!("Mod updated!.");

        self.config.update_config_file()?;
        Ok(())
    }
}
