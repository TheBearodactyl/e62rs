//! blacklist manager ui
use std::sync::Arc;

use demand::MultiSelect;

use {
    demand::{DemandOption, Select},
    hashbrown::HashSet,
};

use {color_eyre::eyre::Context, demand::Input};

use {color_eyre::eyre::Result, demand::Confirm};

use crate::{
    config::blacklist::{add_to_blacklist, clear_blacklist, remove_from_blacklist},
    getopt,
    ui::{E6Ui, autocomplete::TagAutocompleter, menus::BlacklistManager, themes::ROSE_PINE},
};

impl E6Ui {
    /// show info about the blacklist
    pub fn show_blacklist_info(&self) -> Result<()> {
        let blacklist = getopt!(blacklist);

        if blacklist.is_empty() {
            println!("blacklist is empty.");
            return Ok(());
        }

        println!("Current blacklisted tags ({} total):", blacklist.len());
        for (i, tag) in blacklist.iter().enumerate() {
            println!("  {}. {}", i + 1, tag);
        }
        println!(
            "\nNote: Posts with these tags will be filtered out unless explicitly \
             searched for."
        );

        Ok(())
    }

    /// show the blacklist manager ui
    pub async fn manage_blacklist(&self) -> Result<()> {
        loop {
            let blacklist_action = BlacklistManager::select("Blacklist Settings:")
                .run()
                .wrap_err("Failed to display blacklist menu")?;

            let should_continue = match blacklist_action {
                BlacklistManager::ShowCurrent => {
                    self.show_blacklist_info()?;
                    self.prompt_continue()?
                }
                BlacklistManager::AddTag => {
                    self.add_tag_to_blacklist().await?;
                    self.prompt_continue()?
                }
                BlacklistManager::RemoveTag => {
                    self.remove_tag_from_blacklist().await?;
                    self.prompt_continue()?
                }
                BlacklistManager::Clear => {
                    self.clear_blacklist().await?;
                    self.prompt_continue()?
                }
                BlacklistManager::ImportFromSearch => {
                    self.import_tags_to_blacklist().await?;
                    self.prompt_continue()?
                }
                BlacklistManager::Back => break,
            };

            if !should_continue {
                break;
            }
        }

        Ok(())
    }

    /// ask whether to continue managing the blacklist
    fn prompt_continue(&self) -> Result<bool> {
        Confirm::new("Continue managing blacklist?")
            .affirmative("Yes")
            .negative("No")
            .theme(&ROSE_PINE)
            .run()
            .wrap_err("Failed to get user input")
    }

    /// add a tag to the blacklist
    async fn add_tag_to_blacklist(&self) -> Result<()> {
        let tag_db = Arc::clone(&self.tag_db);
        let completer = TagAutocompleter::new(tag_db);

        let tag = Input::new("Enter tag to add to blacklist:")
            .autocomplete(completer)
            .run()
            .wrap_err("Failed to get tag input")?;

        let tag = tag.trim();

        if tag.is_empty() {
            println!("Tag cannot be empty.");
            return Ok(());
        }

        let tag = tag.to_string();
        let blacklist = getopt!(blacklist);

        if blacklist.contains(&tag) {
            println!("Tag '{}' is already in the blacklist.", tag);
            return Ok(());
        }

        if !self.tag_db.exists(&tag) && !self.prompt_add_unknown_tag(&tag).await? {
            return Ok(());
        }

        self.add_validated_tag_to_blacklist(tag).await
    }

    /// ask whether to add a tag not in the tags database
    async fn prompt_add_unknown_tag(&self, tag: &str) -> Result<bool> {
        let use_anyway = Confirm::new(format!(
            "Tag '{}' not found in database. Add to blacklist anyway?",
            tag
        ))
        .affirmative("Yes")
        .negative("No")
        .theme(&ROSE_PINE)
        .run()
        .wrap_err("Failed to get user confirmation")?;

        if use_anyway {
            return Ok(true);
        }

        let suggestions = self.tag_db.search(tag, 5);
        if suggestions.is_empty() {
            return Ok(false);
        }

        let opts: Vec<_> = suggestions.iter().map(DemandOption::new).collect();
        let selected = Select::new("Did you mean one of these tags?")
            .options(opts)
            .description("Select a tag or press ESC to cancel")
            .run()
            .wrap_err("Failed to display tag suggestions")?;

        if !selected.is_empty() {
            self.add_validated_tag_to_blacklist(selected.clone())
                .await?;
        }

        Ok(false)
    }

    /// add a validated tag to the blacklist
    async fn add_validated_tag_to_blacklist(&self, tag: String) -> Result<()> {
        add_to_blacklist(tag.clone())
            .wrap_err_with(|| format!("Failed to add '{}' to blacklist", tag))?;

        println!(
            "Successfully added '{}' to blacklist and saved configuration.",
            tag
        );
        Ok(())
    }

    /// remove a tag from the blacklist
    async fn remove_tag_from_blacklist(&self) -> Result<()> {
        let blacklist = getopt!(blacklist);

        if blacklist.is_empty() {
            println!("Blacklist is empty. Nothing to remove.");
            return Ok(());
        }

        let opts: Vec<_> = blacklist.iter().map(DemandOption::new).collect();
        let tag_to_remove = Select::new("Select tag to remove from blacklist:")
            .options(opts)
            .description("Use arrow keys to navigate, Enter to select, Esc to cancel")
            .run()
            .wrap_err("Failed to display tag selection")?;

        if tag_to_remove.is_empty() {
            return Ok(());
        }

        let confirm = Confirm::new(format!("Remove '{}' from blacklist?", tag_to_remove))
            .affirmative("Yes")
            .negative("No")
            .theme(&ROSE_PINE)
            .run()
            .wrap_err("Failed to get user confirmation")?;

        if !confirm {
            return Ok(());
        }

        match remove_from_blacklist(tag_to_remove.clone().as_str()) {
            Ok(true) => {
                println!(
                    "Successfully removed '{}' from blacklist and saved \
                     configuration.",
                    tag_to_remove
                );
            }
            Ok(false) => {
                println!("Tag '{}' was not found in blacklist.", tag_to_remove);
            }
            Err(e) => {
                return Err(e).wrap_err_with(|| {
                    format!("failed to remove '{}' from blacklist", tag_to_remove)
                });
            }
        }

        Ok(())
    }

    /// clear all tags from the blacklist
    async fn clear_blacklist(&self) -> Result<()> {
        let blacklist = getopt!(blacklist);
        let blacklist_count = blacklist.len();

        if blacklist_count == 0 {
            println!("Blacklist is already empty.");
            return Ok(());
        }

        let confirm = Confirm::new(format!(
            "Clear all {} tags from blacklist? This cannot be undone.",
            blacklist_count
        ))
        .affirmative("Yes")
        .negative("No")
        .theme(&ROSE_PINE)
        .run()
        .wrap_err("Failed to get user confirmation")?;

        if !confirm {
            return Ok(());
        }

        clear_blacklist().wrap_err("Failed to clear blacklist")?;

        println!("Successfully cleared blacklist and saved configuration.");
        Ok(())
    }

    /// import tags from a search to the blacklist
    async fn import_tags_to_blacklist(&self) -> Result<()> {
        let blacklist = getopt!(blacklist);

        println!(
            "This will allow you to search for posts and add their tags to the \
             blacklist."
        );

        let (include_tags, _, exclude_tags) = self
            .collect_tags()
            .wrap_err("Failed to collect search tags")?;

        if include_tags.is_empty() && exclude_tags.is_empty() {
            println!("No search tags provided.");
            return Ok(());
        }

        let mut search_tags = include_tags.clone();
        search_tags.extend(exclude_tags.iter().map(|tag| format!("-{}", tag)));

        let results = self
            .client
            .search_posts(&search_tags, Some(10), None)
            .await
            .wrap_err("Failed to search posts")?;

        if results.posts.is_empty() {
            println!("No posts found for the given search.");
            return Ok(());
        }

        let mut all_tags = HashSet::new();
        for post in &results.posts {
            all_tags.extend(post.tags.general.iter().cloned());
            all_tags.extend(post.tags.artist.iter().cloned());
            all_tags.extend(post.tags.character.iter().cloned());
            all_tags.extend(post.tags.species.iter().cloned());
            all_tags.extend(post.tags.copyright.iter().cloned());
            all_tags.extend(post.tags.meta.iter().cloned());
            all_tags.extend(post.tags.lore.iter().cloned());
        }

        for search_tag in &include_tags {
            all_tags.remove(search_tag);
        }

        let mut sorted_tags: Vec<String> = all_tags.into_iter().collect();
        sorted_tags.sort();

        if sorted_tags.is_empty() {
            println!("No additional tags found to blacklist.");
            return Ok(());
        }

        let selected_tags = MultiSelect::new(format!(
            "Select tags to add to blacklist ({} available):",
            sorted_tags.len()
        ))
        .description("Space to select/deselect, Enter to confirm, Esc to cancel")
        .options(sorted_tags.iter().map(DemandOption::new).collect())
        .run()
        .wrap_err("Failed to display tag selection")?;

        if selected_tags.is_empty() {
            println!("No tags selected.");
            return Ok(());
        }

        let confirm = Confirm::new(format!(
            "Add {} selected tags to blacklist?",
            selected_tags.len()
        ))
        .affirmative("Yes")
        .negative("No")
        .theme(&ROSE_PINE)
        .run()
        .wrap_err("Failed to get user confirmation")?;

        if !confirm {
            return Ok(());
        }

        let mut added_count = 0;
        let mut already_exists = 0;
        let mut errors = Vec::new();

        for tag in selected_tags {
            if blacklist.contains(tag) {
                already_exists += 1;
                continue;
            }

            if let Err(e) = add_to_blacklist(tag.clone()) {
                errors.push((tag, e));
            } else {
                added_count += 1;
            }
        }

        println!("Added {} new tags to blacklist.", added_count);
        if already_exists > 0 {
            println!("{} tags were already in the blacklist.", already_exists);
        }
        if !errors.is_empty() {
            println!("Failed to add {} tags:", errors.len());
            for (tag, err) in errors {
                println!("  - '{}': {}", tag, err);
            }
        }
        println!("Configuration saved.");

        Ok(())
    }
}
