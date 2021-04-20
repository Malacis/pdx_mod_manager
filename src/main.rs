//! This program is for people who own the gog version of paradox games but still want to use mods from steam.
//!
//! This is still a very early version, use at your own risk.
//!
//! Todos:
//! - clean up documentation and rethink the whole struct and program structure
//! - add actual errorhandling
//! - add funcionality for users to cancel operations
//! - let the program find mods and games on its own and configure them automatically
//! - add progress bars for operations
//! - improve interface

mod interface;
mod remote;

use std::collections::HashMap;

use anyhow::Result;
use interface::Interface;
use serde::{Deserialize, Serialize};

/// This struct saves the configuration for this program and is used for toml deserialization and serilization.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// All configured `Game`s.
    games: Vec<Game>,
}

/// Configured games.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Game {
    /// Name of the game.
    title: String,
    /// Path where mods are installed for this game.
    path_mods: String,
    /// Configured `Mod`s.
    mods: HashMap<String, Mod>,
}

/// Configured mods.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Mod {
    /// Name of the mod.
    title: String,
    /// Id of the mod.
    id: u64,
    /// Time of the games last update in unix time.
    time_updated: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut interface = Interface::new()?;

    interface.show_main_menu().await?;

    Ok(())
}
