use ricecoder_help::*;

#[test]
fn test_help_item_creation() {
    let item = HelpItem::new("Test Title", "Test content");
    assert_eq!(item.title, "Test Title");
    assert_eq!(item.content, "Test content");
    assert!(item.keywords.is_empty());
}

#[test]
fn test_help_item_with_keywords() {
    let item = HelpItem::new("Test", "Content")
        .with_keywords(vec!["keyword1".to_string(), "keyword2".to_string()]);
    assert_eq!(item.keywords.len(), 2);
}

#[test]
fn test_help_item_matches() {
    let item = HelpItem::new("Test Title", "Some content")
        .with_keywords(vec!["test".to_string(), "example".to_string()]);
    
    assert!(item.matches("title"));
    assert!(item.matches("Title"));
    assert!(item.matches("content"));
    assert!(item.matches("test"));
    assert!(item.matches("example"));
    assert!(!item.matches("nonexistent"));
}

#[test]
fn test_help_category_creation() {
    let category = HelpCategory::new("Test Category")
        .with_description("Test description")
        .add_item("Item 1", "Content 1")
        .add_item("Item 2", "Content 2");
    
    assert_eq!(category.name, "Test Category");
    assert_eq!(category.description, "Test description");
    assert_eq!(category.items.len(), 2);
}

#[test]
fn test_help_category_search() {
    let category = HelpCategory::new("Test")
        .add_item("First Item", "First content")
        .add_item("Second Item", "Second content");
    
    let results = category.search("first");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "First Item");
}

#[test]
fn test_help_content_creation() {
    let content = HelpContent::new()
        .add_category(HelpCategory::new("Category 1"))
        .add_shortcut("Ctrl+Q", "Quit");
    
    assert_eq!(content.categories.len(), 1);
    assert_eq!(content.shortcuts.len(), 1);
}

#[test]
fn test_help_content_search() {
    let content = HelpContent::new()
        .add_category(
            HelpCategory::new("Commands")
                .add_item("help", "Show help")
                .add_item("exit", "Exit application")
        );
    
    let results = content.search("help");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.title, "help");
}

#[test]
fn test_default_ricecoder_help() {
    let help = HelpContent::default_ricecoder_help();
    
    // Should have multiple categories
    assert!(help.categories.len() >= 6);
    
    // Should have shortcuts
    assert!(!help.shortcuts.is_empty());
    
    // Should have Getting Started category
    assert!(help.get_category("Getting Started").is_some());
    
    // Should have Commands category
    assert!(help.get_category("Commands").is_some());
}

#[test]
fn test_help_system_get_topic() {
    let help = HelpContent::default_ricecoder_help();

    // Test getting existing topic
    let topic = help.get_topic("Welcome to RiceCoder");
    assert!(topic.is_some());
    assert_eq!(topic.unwrap().title, "Welcome to RiceCoder");

    // Test getting non-existent topic
    let topic = help.get_topic("Non-existent Topic");
    assert!(topic.is_none());
}

#[test]
fn test_help_system_search_topics() {
    let help = HelpContent::default_ricecoder_help();

    // Test searching for existing content
    let results = help.search_topics("help");
    assert!(!results.is_empty());

    // Should find the help command
    let help_command = results.iter().find(|item| item.title == "/help");
    assert!(help_command.is_some());

    // Test searching for non-existent content
    let results = help.search_topics("nonexistentquery");
    assert!(results.is_empty());
}