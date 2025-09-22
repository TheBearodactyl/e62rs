use crate::Cfg;
use anyhow::Result;

impl Cfg {
    pub fn add_to_blacklist(&mut self, tag: String) -> Result<()> {
        let blacklist = self.blacklist.get_or_insert_with(Vec::new);

        if !blacklist.contains(&tag) {
            blacklist.push(tag);
            blacklist.sort();

            self.save_to_file("e62rs.toml")?;
        }

        Ok(())
    }

    pub fn remove_from_blacklist(&mut self, tag: &str) -> Result<bool> {
        if let Some(blacklist) = &mut self.blacklist
            && let Some(pos) = blacklist.iter().position(|x| x == tag)
        {
            blacklist.remove(pos);
            self.save_to_file("e62rs.toml")?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn clear_blacklist(&mut self) -> Result<()> {
        if let Some(blacklist) = &mut self.blacklist {
            blacklist.clear();
            self.save_to_file("e62rs.toml")?;
        }

        Ok(())
    }
}
