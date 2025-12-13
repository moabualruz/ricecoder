use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new("id1", "Name", true);
        assert_eq!(node.id, "id1");
        assert_eq!(node.name, "Name");
        assert!(node.is_dir);
        assert!(!node.expanded);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_tree_node_children() {
        let mut node = TreeNode::new("parent", "Parent", true);
        node.add_child("child1".to_string());
        node.add_child("child2".to_string());

        assert_eq!(node.children.len(), 2);
        assert!(node.has_children());

        node.remove_child("child1");
        assert_eq!(node.children.len(), 1);
    }

    #[test]
    fn test_tree_node_expansion() {
        let mut node = TreeNode::new("id", "Name", true);
        assert!(!node.expanded);

        node.toggle_expanded();
        assert!(node.expanded);

        node.collapse();
        assert!(!node.expanded);

        node.expand();
        assert!(node.expanded);
    }

    #[test]
    fn test_tree_widget_creation() {
        let widget = TreeWidget::new("root", "Root");
        assert_eq!(widget.root_id, "root");
        assert_eq!(widget.visible_count(), 1);
    }

    #[test]
    fn test_tree_widget_add_node() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", false);
        widget.add_node("root", child);

        assert_eq!(widget.visible_count(), 1); // Still 1 because root is not expanded
    }

    #[test]
    fn test_tree_widget_expand_collapse() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", false);
        widget.add_node("root", child);

        widget.expand_node("root");
        assert_eq!(widget.visible_count(), 2);

        widget.collapse_node("root");
        assert_eq!(widget.visible_count(), 1);
    }

    #[test]
    fn test_tree_widget_selection() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", false);
        widget.add_node("root", child);

        widget.select("child1".to_string());
        assert_eq!(widget.selected(), Some("child1"));

        widget.deselect();
        assert!(widget.selected().is_none());
    }

    #[test]
    fn test_tree_widget_select_next_prev() {
        let mut widget = TreeWidget::new("root", "Root");
        let child1 = TreeNode::new("child1", "Child 1", false);
        let child2 = TreeNode::new("child2", "Child 2", false);
        widget.add_node("root", child1);
        widget.add_node("root", child2);
        widget.expand_node("root");

        widget.select_next();
        assert_eq!(widget.selected(), Some("root"));

        widget.select_next();
        assert_eq!(widget.selected(), Some("child1"));

        widget.select_prev();
        assert_eq!(widget.selected(), Some("root"));
    }

    #[test]
    fn test_tree_widget_expand_all() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", true);
        widget.add_node("root", child);

        widget.expand_all();
        assert!(widget.get_node("root").unwrap().expanded);
        assert!(widget.get_node("child1").unwrap().expanded);
    }

    #[test]
    fn test_tree_widget_collapse_all() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", true);
        widget.add_node("root", child);

        widget.expand_all();
        widget.collapse_all();
        assert!(!widget.get_node("root").unwrap().expanded);
        assert!(!widget.get_node("child1").unwrap().expanded);
    }

    #[test]
    fn test_tree_widget_remove_node() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", false);
        widget.add_node("root", child);

        widget.remove_node("child1");
        assert!(widget.get_node("child1").is_none());
    }

    #[test]
    fn test_tree_widget_clear() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", false);
        widget.add_node("root", child);
        widget.select("child1".to_string());

        widget.clear();
        assert_eq!(widget.visible_count(), 1);
        assert!(widget.selected().is_none());
    }

    #[test]
    fn test_tree_widget_display_text() {
        let mut widget = TreeWidget::new("root", "Root");
        let child = TreeNode::new("child1", "Child 1", true);
        widget.add_node("root", child);

        let text = widget.get_display_text("root");
        assert!(text.contains("Root"));

        let text = widget.get_display_text("child1");
        assert!(text.contains("Child 1"));
    }
}