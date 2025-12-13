use ricecoder_tui::*;
use std::sync::Arc;
use tokio::sync::RwLock;

mod tests {
    use super::*;

    #[test]
    fn test_virtual_list_creation() {
        let items = vec![1, 2, 3, 4, 5];
        let render_fn = |item: &i32, _index: usize| VirtualNode::Text(item.to_string());
        let list = VirtualList::new(items, 3, render_fn);

        assert_eq!(list.scroll.total_items, 5);
        assert_eq!(list.scroll.visible_items, 3);
    }

    #[test]
    fn test_virtual_list_visible_items() {
        let items = vec![1, 2, 3, 4, 5];
        let render_fn = |item: &i32, _index: usize| VirtualNode::Text(item.to_string());
        let list = VirtualList::new(items, 3, render_fn);

        let visible = list.visible_items();
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn test_virtual_renderer_diffing() {
        let mut renderer = VirtualRenderer::new();

        let old_tree = VirtualNode::Text("old".to_string());
        let new_tree = VirtualNode::Text("new".to_string());

        let operations = renderer.render(new_tree);
        assert!(!operations.is_empty());
    }

    #[test]
    fn test_lazy_loader() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let loaded = Arc::new(RwLock::new(Vec::new()));

        let loaded_clone = Arc::clone(&loaded);
        let mut loader = LazyLoader::new(items, 3, move |batch| {
            let mut loaded = loaded_clone.blocking_write();
            loaded.extend(batch);
        });

        // Load first batch
        futures::executor::block_on(loader.load_next_batch());
        assert_eq!(loader.loaded_items().len(), 3);
        assert_eq!(loader.progress(), 0.3);
    }

    #[test]
    fn test_circular_buffer() {
        let mut buffer = memory_efficient::CircularBuffer::new(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4); // Should evict 1

        let items: Vec<_> = buffer.iter().cloned().collect();
        assert_eq!(items, vec![2, 3, 4]);
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = memory_efficient::LruCache::new(3);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.put("d", 4); // Should evict "a"

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"d"), Some(&4)); // "d" should be most recent
    }
}