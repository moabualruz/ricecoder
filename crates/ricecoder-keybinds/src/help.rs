//! Help system for displaying keybinds

use crate::models::Keybind;

/// Pagination information
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub current_page: usize,
    pub total_pages: usize,
}

/// Keybind help system
pub struct KeybindHelp;

impl KeybindHelp {
    /// Display all keybinds
    pub fn display_all(keybinds: &[&Keybind]) -> String {
        let mut output = String::from("# All Keybinds\n\n");

        if keybinds.is_empty() {
            output.push_str("No keybinds configured.\n");
            return output;
        }

        // Group by category
        let mut categories: std::collections::HashMap<String, Vec<&Keybind>> =
            std::collections::HashMap::new();

        for keybind in keybinds {
            categories
                .entry(keybind.category.clone())
                .or_default()
                .push(keybind);
        }

        // Sort categories
        let mut sorted_categories: Vec<_> = categories.keys().collect();
        sorted_categories.sort();

        for category in sorted_categories {
            output.push_str(&format!("## {}\n\n", category));

            let mut keybinds_in_category = categories[category].clone();
            keybinds_in_category.sort_by(|a, b| a.action_id.cmp(&b.action_id));

            for keybind in keybinds_in_category {
                output.push_str(&format!(
                    "- **{}**: `{}` - {}\n",
                    keybind.action_id, keybind.key, keybind.description
                ));
            }

            output.push('\n');
        }

        output
    }

    /// Display keybinds by category
    pub fn display_by_category(keybinds: &[&Keybind], category: &str) -> String {
        let filtered: Vec<&Keybind> = keybinds
            .iter()
            .filter(|kb| kb.category == category)
            .copied()
            .collect();

        if filtered.is_empty() {
            return format!("No keybinds found in category: {}\n", category);
        }

        let mut output = format!("# {} Keybinds\n\n", category);

        for keybind in filtered {
            output.push_str(&format!(
                "- **{}**: `{}` - {}\n",
                keybind.action_id, keybind.key, keybind.description
            ));
        }

        output
    }

    /// Search keybinds by query
    pub fn search<'a>(keybinds: &[&'a Keybind], query: &str) -> Vec<&'a Keybind> {
        let query_lower = query.to_lowercase();

        keybinds
            .iter()
            .filter(|kb| {
                kb.action_id.to_lowercase().contains(&query_lower)
                    || kb.key.to_lowercase().contains(&query_lower)
                    || kb.description.to_lowercase().contains(&query_lower)
            })
            .copied()
            .collect()
    }

    /// Paginate keybinds
    pub fn paginate<'a>(keybinds: &[&'a Keybind], page: usize, page_size: usize) -> Page<&'a Keybind> {
        if page_size == 0 {
            return Page {
                items: Vec::new(),
                current_page: 0,
                total_pages: 0,
            };
        }

        let total_pages = keybinds.len().div_ceil(page_size);

        if page == 0 || page > total_pages {
            return Page {
                items: Vec::new(),
                current_page: page,
                total_pages,
            };
        }

        let start = (page - 1) * page_size;
        let end = std::cmp::min(start + page_size, keybinds.len());

        Page {
            items: keybinds[start..end].to_vec(),
            current_page: page,
            total_pages,
        }
    }
}


