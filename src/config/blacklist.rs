use color_eyre::eyre::Result;

use crate::config::options::E62Rs;

impl E62Rs {
    pub fn add_to_blacklist(&mut self, tag: String) -> Result<()> {
        let blacklist = &mut self.blacklist;

        if !blacklist.contains(&tag) {
            blacklist.push(tag);
            blacklist.sort();
            self.save_to_file("e62rs.toml")?;
        }

        Ok(())
    }

    pub fn remove_from_blacklist(&mut self, tag: &str) -> Result<bool> {
        let blacklist = &mut self.blacklist;
        if let Some(pos) = blacklist.iter().position(|x| x == tag) {
            blacklist.remove(pos);
            self.save_to_file("e62rs.toml")?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn clear_blacklist(&mut self) -> Result<()> {
        let blacklist = &mut self.blacklist;
        blacklist.clear();
        self.save_to_file("e62rs.toml")?;
        Ok(())
    }
}
