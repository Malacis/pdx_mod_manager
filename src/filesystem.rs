//! Filesystem functionality.

use std::{
    fs::{self, OpenOptions},
    io::{Cursor, Write},
    path::Path,
};

use crate::Config;
use anyhow::Result;
use bytes::Bytes;
use zip::ZipArchive;

pub fn write_mod(
    id: u64,
    title: String,
    mut zip: ZipArchive<Cursor<Bytes>>,
    path_mods: &str,
) -> Result<()> {
    let install_path = format!("{}/{}", path_mods.trim(), id);
    let mod_file_path = format!("{}/{}.mod", path_mods.trim(), id);
    if Path::new(&install_path).exists() {
        println!("Deleting old mod folder.");
        fs::remove_dir_all(&install_path)?;
    }
    if Path::new(&mod_file_path).exists() {
        println!("Deleting old .mod file.");
        fs::remove_file(&mod_file_path)?;
    }

    zip.extract(install_path)?;

    println!("Writing .mod file.");
    let mut mod_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(mod_file_path)?;

    mod_file.write_all(
        format!(
            "name=\"{}\"\npath=\"{}\"",
            title.trim(),
            format!("mod/{}", id)
        )
        .as_bytes(),
    )?;
    Ok(())
}

impl Config {
    /// Updates the config file.
    pub fn update_config_file(&self) -> Result<()> {
        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("config.toml")?;

        config_file.write_all(toml::to_string(&self)?.as_bytes())?;
        Ok(())
    }
}
