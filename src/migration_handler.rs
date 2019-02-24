use crate::errors::*;
use crate::utils::*;
use std::fs;

pub fn migrate_old_subscriptions() -> Result<()> {
    let path = get_podcast_dir()?;
    let mut old_path = path.clone();
    old_path.push(".subscriptions");
    if old_path.exists() {
        println!("Migrating old subscriptions file...");
        let new_path = get_sub_file()?;
        fs::rename(&old_path, &new_path)
            .chain_err(|| format!("Unable to move {:?} to {:?}", &old_path, &new_path))?;
    }
    Ok(())
}
