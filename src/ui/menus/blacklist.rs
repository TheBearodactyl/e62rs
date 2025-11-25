use {
    crate::{
        config::options::E62Rs,
        ui::{E6Ui, ROSE_PINE, autocomplete::TagAutocompleter, menus::BlacklistManager},
    },
    color_eyre::eyre::Result,
    demand::{Confirm, DemandOption, Input, MultiSelect, Select},
    std::sync::Arc,
};

impl E6Ui {
    pub fn show_blacklist_info(&self) -> Result<()> {
        let config = E62Rs::get()?;
        let blacklist = config.blacklist;

        if blacklist.is_empty() {
            println!("Blacklist is empty.");
        } else {
            println!("Current blacklisted tags ({} total):", blacklist.len());
            for (i, tag) in blacklist.iter().enumerate() {
                println!("  {}. {}", i + 1, tag);
            }
            println!(
                "\nNote: Posts with these tags will be filtered out unless explicitly searched \
                 for."
            );
        }

        Ok(())
    }

    pub async fn manage_blacklist(&self) -> Result<()> {
        loop {
            let blacklist_action = BlacklistManager::select("Blacklist Settings:").run()?;

            match blacklist_action {
                BlacklistManager::ShowCurrent => {
                    self.show_blacklist_info()?;
                }
                BlacklistManager::AddTag => {
                    self.add_tag_to_blacklist().await?;
                }
                BlacklistManager::RemoveTag => {
                    self.remove_tag_from_blacklist().await?;
                }
                BlacklistManager::Clear => {
                    self.clear_blacklist().await?;
                }
                BlacklistManager::ImportFromSearch => {
                    self.import_tags_to_blacklist().await?;
                }
                BlacklistManager::Back => break,
            }

            if matches!(blacklist_action, BlacklistManager::Back) {
                break;
            }

            if !Confirm::new("Continue managing blacklist?")
                .affirmative("Yes")
                .negative("No")
                .theme(&ROSE_PINE)
                .run()?
            {
                break;
            }
        }
        Ok(())
    }

    fn autocomplete_blacklist_tags(&self, input: &str) -> Result<Vec<String>, String> {
        let suggestions = self.tag_db.autocomplete(input, 10);

        Ok(suggestions)
    }

    async fn add_tag_to_blacklist(&self) -> Result<()> {
        let tag_db = Arc::clone(&self.tag_db);
        let completer = TagAutocompleter::new(tag_db);
        let tag = Input::new("Enter tag to add to blacklist:")
            .autocomplete(completer)
            .run()?;

        let tag = tag.trim().to_string();
        if tag.is_empty() {
            println!("Tag cannot be empty.");
            return Ok(());
        }

        if !self.tag_db.exists(&tag) {
            let use_anyway = Confirm::new(format!(
                "Tag '{}' not found in database. Add to blacklist anyway?",
                tag
            ))
            .affirmative("Yes")
            .negative("No")
            .theme(&ROSE_PINE)
            .run()?;

            if !use_anyway {
                let suggestions = self.tag_db.search(&tag, 5);
                if !suggestions.is_empty() {
                    let opts = suggestions
                        .iter()
                        .map(DemandOption::new)
                        .collect::<Vec<_>>();
                    let selected = Select::new("Did you mean one of these tags?")
                        .options(opts)
                        .description("Select a tag or press ESC to cancel")
                        .run()?;

                    if !selected.is_empty() {
                        return self.add_validated_tag_to_blacklist(selected.clone()).await;
                    }
                }
                return Ok(());
            }
        }

        self.add_validated_tag_to_blacklist(tag).await
    }

    async fn add_validated_tag_to_blacklist(&self, tag: String) -> Result<()> {
        let mut config = E62Rs::get().unwrap_or_default();
        let blacklist = &config.blacklist;

        if blacklist.contains(&tag) {
            println!("Tag '{}' is already in the blacklist.", tag);
            return Ok(());
        }

        match config.add_to_blacklist(tag.clone()) {
            Ok(()) => {
                println!(
                    "Successfully added '{}' to blacklist and saved configuration.",
                    tag
                );
            }
            Err(e) => {
                println!("Failed to add tag to blacklist: {}", e);
            }
        }

        Ok(())
    }

    async fn remove_tag_from_blacklist(&self) -> Result<()> {
        let config = E62Rs::get().unwrap_or_default();
        let blacklist = &config.blacklist;

        let blacklisted = blacklist.iter().map(DemandOption::new).collect::<Vec<_>>();
        let tag_to_remove = Select::new("Select tag to remove from blacklist:")
            .options(blacklisted)
            .description("Use arrow keys to navigate, Enter to select, Esc to cancel")
            .run()?;

        if !tag_to_remove.is_empty() {
            let confirm = Confirm::new(format!("Remove '{}' from blacklist?", tag_to_remove))
                .affirmative("Yes")
                .negative("No")
                .theme(&ROSE_PINE)
                .run()?;

            if confirm {
                let mut config = E62Rs::get().unwrap_or_default();
                match config.remove_from_blacklist(tag_to_remove) {
                    Ok(true) => {
                        println!(
                            "Successfully removed '{}' from blacklist and saved configuration.",
                            tag_to_remove
                        );
                    }
                    Ok(false) => {
                        println!("Tag '{}' was not found in blacklist.", tag_to_remove);
                    }
                    Err(e) => {
                        println!("Failed to remove tag from blacklist: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn clear_blacklist(&self) -> Result<()> {
        let config = E62Rs::get()?;
        let blacklist_count = config.blacklist.len();

        if blacklist_count == 0 {
            println!("Blacklist is already empty.");
            return Ok(());
        }

        let confirm = Confirm::new(format!(
            "Clear all {} tags from blacklist? This cannot be undone.",
            blacklist_count
        ))
        .run()?;

        if confirm {
            let mut config = E62Rs::get().unwrap_or_default();
            match config.clear_blacklist() {
                Ok(()) => {
                    println!("Successfully cleared blacklist and saved configuration.");
                }
                Err(e) => {
                    println!("Failed to clear blacklist: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn import_tags_to_blacklist(&self) -> Result<()> {
        let blacklist = E62Rs::get()?.blacklist;
        println!("This will allow you to search for posts and add their tags to the blacklist.");

        let (include_tags, _, exclude_tags) = self.collect_tags()?;
        if include_tags.is_empty() && exclude_tags.is_empty() {
            println!("No search tags provided.");
            return Ok(());
        }

        let mut search_tags = include_tags.clone();
        for exclude_tag in exclude_tags {
            search_tags.push(format!("-{}", exclude_tag));
        }

        let results = self
            .client
            .search_posts(search_tags, Some(10), None)
            .await?;

        if results.posts.is_empty() {
            println!("No posts found for the given search.");
            return Ok(());
        }

        let mut all_tags: std::collections::HashSet<String> = std::collections::HashSet::new();
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
        .options(
            sorted_tags
                .iter()
                .map(DemandOption::new)
                .collect::<Vec<_>>(),
        )
        .run()?;

        if selected_tags.is_empty() {
            println!("No tags selected.");
            return Ok(());
        }

        let confirm = Confirm::new(format!(
            "Add {} selected tags to blacklist?",
            selected_tags.len()
        ))
        .run()?;

        if confirm {
            let mut config = E62Rs::get()?;
            let mut added_count = 0;
            let mut already_exists = 0;

            for tag in selected_tags {
                if blacklist.contains(tag) {
                    already_exists += 1;
                    continue;
                }

                if let Err(e) = config.add_to_blacklist(tag.clone()) {
                    println!("Failed to add '{}': {}", tag, e);
                } else {
                    added_count += 1;
                }
            }

            println!("Added {} new tags to blacklist.", added_count);
            if already_exists > 0 {
                println!("{} tags were already in the blacklist.", already_exists);
            }
            println!("Configuration saved.");
        }

        Ok(())
    }
}
