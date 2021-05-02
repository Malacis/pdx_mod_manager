//! Interface and filesystem functionality concering `Game`s.

use crate::Game;
use anyhow::Result;
use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use std::{collections::HashMap, path::Path};

use super::{ui_selection, Interface};

impl Interface {
    /// Shows all games saved in configuration.
    #[async_recursion]
    pub async fn show_games(&mut self) -> Result<()> {
        if self.config.games.is_empty() {
            println!("No game found, please add one!");
            self.show_main_menu().await?;
        }

        let mut items_games = Vec::with_capacity(self.config.games.len() + 1);

        items_games.extend(
            self.config
                .games
                .iter()
                .map(|Game { title, .. }| title.as_str()),
        );
        items_games.push("Go back.");

        let selection_games = ui_selection(&items_games);

        if let Some(selection) = selection_games {
            if selection == items_games.len() - 1 {
                self.show_main_menu().await?;
            } else {
                self.selection = Some(selection);
                self.show_game_options().await?;
            }
        }
        Ok(())
    }

    /// Shows options for `Game`s.
    #[async_recursion]
    pub async fn show_game_options(&mut self) -> Result<()> {
        let game_details_items = [
            "Add mod.",
            "Delete mods.",
            "Update mods.",
            "Change game path.",
            "Change game name.",
            "Go back.",
        ];

        let selection_game_options = ui_selection(&game_details_items);

        if let Some(index) = selection_game_options {
            match index {
                0 => self.add_mod().await,
                1 => self.delete_mods().await,
                2 => self.update_all_game_mods().await,
                3 => self.change_game_path().await,
                4 => self.change_game_name().await,
                5 => {
                    self.selection = None;
                    self.show_games().await
                }
                _ => Ok(()),
            }
        } else {
            println!("User did not select anything");
            Ok(())
        }
    }

    /// Adds `Game`s.
    pub async fn add_games_manually(&mut self) -> Result<()> {
        let title: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Type in the name of the game. Name can be up to 30 characters long!")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() <= 30 {
                    Ok(())
                } else {
                    Err("Name is too long!")
                }
            })
            .interact_text()?;

        let path_mods: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Type or copy paste in the path to the mod folder.\nPlease make sure to put in the correct path! ex: C:\\Users\\Alice\\Documents\\Paradox Interactive\\Stellaris\\mod")
            .validate_with(|input: &String| -> Result<(), &str> {
                if Path::new(input).exists() {
                    Ok(())
                } else {
                    Err("This path does not exist!")
                }
            })
            .interact_text()
            ?;

        self.config.games.push(Game {
            title,
            path_mods,
            mods: HashMap::new(),
        });

        self.config.update_config_file()?;
        println!("Game added successfully!");
        self.show_main_menu().await
    }

    /// Update all mods for a selected game.
    pub async fn update_all_game_mods(&mut self) -> Result<()> {
        for (_, item_mod) in self
            .config
            .games
            .get(self.selection.expect("selection empty"))
            .expect("failed to get game")
            .mods
            .clone()
        {
            self.update_mod(item_mod.id, self.selection.expect("selection empty"))
                .await?;
        }
        self.show_game_options().await
    }

    /// Deletes a selectet game.
    pub async fn delete_game(&mut self) -> Result<()> {
        let games = &mut self.config.games;
        let mut items = vec![];
        for game in games.clone() {
            items.push(game.title);
        }

        if items.is_empty() {
            println!("You have no games configured yet!");
            return self.show_main_menu().await;
        }

        let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select with space, confirm with enter. Confirming without selecting anything will cancel.")
            .items(&items)
            .interact()?;

        if chosen.is_empty() {
            return self.show_main_menu().await;
        }

        for index in chosen {
            let _old = games.remove(index);
        }

        self.config.update_config_file()?;

        println!("Games removed!");
        self.show_main_menu().await
    }

    /// Change game of a selected game.
    pub async fn change_game_name(&mut self) -> Result<()> {
        let game = self
            .config
            .games
            .get_mut(self.selection.expect("game selection empty"))
            .expect("get game failed");
        let old_title = game.title.clone();
        let new_title: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Type in the new name of the game. Name can be up to 30 characters long!")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() <= 30 {
                    Ok(())
                } else {
                    Err("Name is too long!")
                }
            })
            .interact_text()?;

        game.title = new_title.clone();
        println!(
            "Name of {} changed to {} successfully!",
            old_title, new_title
        );
        self.show_games().await
    }

    /// Change mod path of the selected game.
    pub async fn change_game_path(&mut self) -> Result<()> {
        let new_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Type or copy paste in the new path to the mod folder.\nPlease make sure to put in the correct path! ex: C:\\Users\\Alice\\Documents\\Paradox Interactive\\Stellaris\\mod")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() <= 30 {
                    Ok(())
                } else {
                    Err("Name is too long!")
                }
            })
            .interact_text()?;

        self.config
            .games
            .get_mut(self.selection.expect("no game selection"))
            .expect("get game failed")
            .path_mods = new_path.clone();
        println!("Changed path successfully!");
        self.show_games().await
    }
}
