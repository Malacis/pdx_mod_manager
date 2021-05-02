//! Interface functionality.

mod games;
mod mods;

use anyhow::Result;
use async_recursion::async_recursion;
use std::{
    fs::{self, File},
    io::Write,
};

use crate::{remote::Remote, Config};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};

/// Uses the dialoguer crate to give the user a selection.
fn ui_selection(items: &[&str]) -> Option<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .expect("dialouger error")
}

/// Parent struct for the whole program.
pub struct Interface {
    /// Holds the saved configuration of the program.
    pub config: Config,
    /// The currently selected `Game`.
    pub selection: Option<usize>,
    /// Holds the current `reqwest::Client`.
    pub remote: Remote,
}

impl Interface {
    /// Instanciates a new `Interface` struct.
    pub fn new() -> Result<Self> {
        let config = if let Ok(config) = fs::read_to_string("config.toml") {
            config
        } else {
            let mut file = File::create("config.toml")?;
            let new_config = String::from("games = []");
            file.write_all(b"games = []")?;
            new_config
        };

        Ok(Self {
            config: toml::from_str::<Config>(&config)?,
            selection: None,
            remote: Remote::new(),
        })
    }

    /// Shows the main menu.
    #[async_recursion]
    pub async fn show_main_menu(&mut self) -> Result<()> {
        let items_options = [
            "Show games.",
            "Add new game manually.",
            "Update all mods.",
            "Delete game. This just deletes the configuration for this program, not the actual game.",
            "Exit.",
        ];

        let selection_options = ui_selection(&items_options);

        if let Some(index) = selection_options {
            match index {
                0 => self.show_games().await,
                1 => self.add_games_manually().await,
                2 => self.update_all_mods().await,
                3 => self.delete_game().await,
                _ => Ok(()),
            }
        } else {
            println!("User did not select anything");
            Ok(())
        }
    }

    /// updates all mods.
    pub async fn update_all_mods(&mut self) -> Result<()> {
        for (i, game) in self.config.games.clone().into_iter().enumerate() {
            for (_, item_mod) in game.mods.clone() {
                self.update_mod(item_mod.id, i).await?;
            }
        }
        self.show_main_menu().await
    }
}
