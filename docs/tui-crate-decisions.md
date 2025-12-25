# TUI Crate Selection Decisions

> **Research Status**: Task 31.5 - Ratatui Ecosystem Research
> **Date**: 2025-12-25
> **Purpose**: Evaluate ecosystem crates for adoption/replacement/removal

## Decision Framework

### Evaluation Criteria

| Criterion | Weight | Description |
|-----------|--------|-------------|
| **Maintenance** | ⭐⭐⭐ | Active development, recent commits, responsive maintainers |
| **Features** | ⭐⭐⭐ | Feature completeness, extensibility, API quality |
| **Integration** | ⭐⭐ | Ease of integration, compatibility with existing code |
| **Performance** | ⭐⭐ | Runtime efficiency, memory usage, render speed |
| **Documentation** | ⭐ | API docs, examples, guides |

### Decision Categories

- ✅ **KEEP** - Already in use, working well, no action needed
- ⚠️ **EVALUATE** - Requires detailed comparison vs custom implementation
- ⬆️ **ADOPT** - Not in use, should add to dependencies
- ⛔ **SKIP** - Not needed, redundant, or inferior to alternatives

---

## 1. Crates Already in Use

### 1.1 ratatui-textarea ✅ KEEP

**Status**: ✅ **KEEP** - Core feature, well-integrated

**Evidence**:
- **Cargo.toml**: Line 55 - `ratatui-textarea = { workspace = true }`
- **Integration**: `src/textarea_widget.rs` (245 lines, custom wrapper)
- **Usage**: Multi-line input with vim mode in chat interface

**Features**:
| Feature | Status | Usage |
|---------|--------|-------|
| Multi-line editing | ✅ | Chat input |
| Vim mode (insert/normal/visual) | ✅ | Vim mode state management |
| Undo/redo history | ✅ | `set_max_histories(50)` |
| Word wrapping | ⚠️ | Not actively used |
| Selection support | ✅ | Custom Selection struct |
| Search/replace | ❌ | Not used |

**Integration Quality**: ⭐⭐⭐⭐
- Custom wrapper adds: Selection, history drafts, autocomplete hook
- Clean API: `TextAreaWidget::new()`, `text()`, `set_text()`, `handle_key()`
- No breaking changes

**Recommendation**: ✅ **KEEP** - Essential dependency, good integration

---

### 1.2 tui-tree-widget ⚠️ EVALUATE

**Status**: ⚠️ **EVALUATE** - Custom implementation vs ecosystem crate

**Evidence**:
- **Cargo.toml**: Line 56 - `tui-tree-widget = { workspace = true }`
- **Integration**: `src/tree_widget.rs` (custom TreeNode/TreeWidget, 100+ lines)
- **Usage**: File picker, project navigation

**Problem**: We have **BOTH** custom implementation AND dependency

**Custom Implementation**:
```rust
pub struct TreeNode {
    id: String,
    name: String,
    is_dir: bool,
    children: Vec<String>,
    expanded: bool,
    depth: usize,
}

pub struct TreeWidget {
    nodes: HashMap<String, TreeNode>,
    root_id: String,
    selected: Option<String>,
    visible_nodes: Vec<String>,
    // ...
}
```

**tui-tree-widget Features**:
- `TreeItem<'a, Identifier>` - Generic tree item
- `TreeState<Identifier>` - Stateful navigation
- Expand/collapse built-in
- Automatic rendering

**Comparison**:
| Feature | Custom | tui-tree-widget |
|---------|--------|-----------------|
| Generic types | ❌ String IDs | ✅ Generic `Identifier` |
| State management | Manual HashMap | ✅ Built-in TreeState |
| Rendering | Custom | ✅ Widget trait |
| Performance | Unknown | Optimized |

**Decision Factors**:
1. ✅ **Use tui-tree-widget IF**: We need generic IDs, want less maintenance
2. ✅ **Keep custom IF**: Specific requirements not met by ecosystem

**Action Required**: ⚠️ **Compare features** - Conduct detailed analysis

**Recommendation**: ⚠️ **EVALUATE** - Need feature parity analysis

---

### 1.3 ratatui-image ✅ KEEP

**Status**: ✅ **KEEP** - Unique feature, well-integrated

**Evidence**:
- **Cargo.toml**: Line 57 - `ratatui-image = { workspace = true }`
- **Integration**: `src/image_widget.rs`, `src/image_integration.rs`
- **Usage**: Image rendering in chat (screenshots, diagrams, mockups)

**Features**:
- Protocol support: Sixel, Kitty, iTerm2
- Formats: PNG, JPG, GIF, WebP
- Automatic protocol detection
- Resize and crop

**Integration Quality**: ⭐⭐⭐⭐⭐
- No alternatives available
- Critical for image support feature

**Recommendation**: ✅ **KEEP** - No alternatives, essential

---

### 1.4 ratatui-explorer ✅ KEEP

**Status**: ✅ **KEEP** - File picker functionality

**Evidence**:
- **Cargo.toml**: Line 58 - `ratatui-explorer = { workspace = true }`
- **Integration**: `src/file_picker.rs`
- **Usage**: File browser UI for file selection

**Features**:
- File system navigation
- Keyboard shortcuts
- Filtering
- Theme customization

**Integration Quality**: ⭐⭐⭐⭐
- Provides complete file picker
- No need to reinvent

**Recommendation**: ✅ **KEEP** - Essential for file picker

---

### 1.5 tui-scrollview ✅ KEEP

**Status**: ✅ **KEEP** - Chat message scrolling

**Evidence**:
- **Cargo.toml**: Line 59 - `tui-scrollview = { workspace = true }`
- **Integration**: `src/scrollview_widget.rs` (custom wrapper with ScrollState)
- **Usage**: Scrollable chat message history

**Features**:
- Vertical/horizontal scrolling
- Scroll state management
- Viewport tracking

**Integration Quality**: ⭐⭐⭐⭐
- Custom ScrollState wrapper adds resize handling
- Critical for chat UX

**Recommendation**: ✅ **KEEP** - Essential for chat display

---

## 2. Crates NOT in Use - Evaluation

### 2.1 tui-input ⛔ SKIP

**Status**: ⛔ **SKIP** - Redundant with ratatui-textarea

**Purpose**: Single-line input widget

**Features**:
- Single-line text input
- Cursor movement
- Basic editing

**Why SKIP**:
1. **Redundant**: ratatui-textarea handles multi-line → can be limited to 1 line
2. **Feature Overlap**: All tui-input features covered by ratatui-textarea
3. **Complexity**: Adding another input crate increases maintenance

**Alternative**: Use `TextAreaWidget` with `max_height: 1`

**Recommendation**: ⛔ **SKIP** - Not needed

---

### 2.2 tui-popup ⚠️ EVALUATE

**Status**: ⚠️ **EVALUATE** - Compare vs custom `popup_widget.rs`

**Purpose**: Modal dialog widgets

**Features**:
- Centered positioning
- Backdrop dimming
- Modal behavior
- Keyboard/mouse handling

**Custom Implementation**: `src/popup_widget.rs` exists

**Comparison Needed**:
| Feature | Custom | tui-popup |
|---------|--------|-----------|
| Positioning | ? | ✅ Centered |
| Backdrop | ? | ✅ Dimming |
| Types | ? | ✅ Multiple |
| API | ? | ? |

**Decision Factors**:
1. ✅ **Adopt tui-popup IF**: Better features, maintained
2. ✅ **Keep custom IF**: Specific requirements

**Action Required**: ⚠️ **Feature comparison** - Read both implementations

**Recommendation**: ⚠️ **EVALUATE** - Detailed comparison needed

---

### 2.3 tui-logger ⚠️ EVALUATE

**Status**: ⚠️ **EVALUATE** - Compare vs custom `logger_widget.rs`

**Purpose**: Log viewer widget with filtering

**Features**:
- Log level filtering
- Scrollable display
- Circular buffer
- Color coding
- Integration with `log` crate

**Custom Implementation**: `src/logger_widget.rs` exists

**Comparison Needed**:
| Feature | Custom | tui-logger |
|---------|--------|------------|
| Levels | ? | ✅ All levels |
| Filtering | ? | ✅ Built-in |
| Buffer | ? | ✅ Circular |
| API | ? | ? |

**Decision Factors**:
1. ✅ **Adopt tui-logger IF**: Feature-rich, maintained
2. ✅ **Keep custom IF**: Integration with ricecoder logging

**Action Required**: ⚠️ **Feature comparison** - Read both implementations

**Recommendation**: ⚠️ **EVALUATE** - Detailed comparison needed

---

### 2.4 edtui ⚠️ INVESTIGATE

**Status**: ⚠️ **INVESTIGATE** - Significant overlap with `code_editor_widget.rs`

**Purpose**: Full vim-inspired text editor widget

**Features**:
- Multi-file editing
- Syntax highlighting (syntect)
- Vim keybindings (modal editing)
- Line numbers
- Search/replace
- Undo/redo

**Custom Implementation**: `src/code_editor_widget.rs` exists

**Overlap Analysis**:
| Feature | Custom | edtui |
|---------|--------|-------|
| Multi-line | ✅ | ✅ |
| Syntax highlighting | ✅ (syntect) | ✅ (syntect) |
| Vim mode | ⚠️ ? | ✅ Full |
| Line numbers | ⚠️ ? | ✅ |
| Multi-file | ❌ ? | ✅ |

**Major Decision**:
- **Adopt edtui**: Replace custom editor, reduce maintenance
- **Keep custom**: Specific requirements, tight integration

**Action Required**: ⚠️ **Deep investigation** - Feature parity + migration cost

**Recommendation**: ⚠️ **INVESTIGATE** - High-impact decision

---

### 2.5 tui-realm ⛔ SKIP

**Status**: ⛔ **SKIP** - ricecoder-tui has mature component framework

**Purpose**: React/Elm-inspired component framework

**Features**:
- Component lifecycle
- Props and state
- Message passing
- Event bubbling

**Why SKIP**:
1. **Existing Framework**: ricecoder-tui has comprehensive `components/` system
2. **Migration Cost**: Massive refactoring required
3. **No Clear Benefit**: Our component system is mature

**Evidence**: `src/components/` has:
- `Component` trait
- `ComponentRegistry`
- Event system (`EventComponent`, `EventContext`)
- Lifecycle management (`lifecycle.rs`)

**Recommendation**: ⛔ **SKIP** - Not needed, redundant

---

## 3. Ratatui Built-in Widgets (NOT in Use)

### 3.1 Table ⬆️ ADOPT

**Status**: ⬆️ **ADOPT** - High value for structured data

**Purpose**: Tabular data with columns and rows

**Use Cases**:
- Session history listing
- Provider comparison
- Configuration display
- Metrics/statistics

**Features**:
- Header row
- Column width constraints
- Row selection (`TableState`)
- Styling per row/column

**Integration Effort**: ⭐ Low (built-in widget)

**Value**: ⭐⭐⭐ High (common pattern)

**Recommendation**: ⬆️ **ADOPT** - Add to ricecoder-tui

---

### 3.2 Gauge ⬆️ ADOPT

**Status**: ⬆️ **ADOPT** - Useful for progress tracking

**Purpose**: Progress bar with percentage display

**Use Cases**:
- Download progress
- Task completion
- Token usage meters
- Operation progress

**Features**:
- Ratio (0.0-1.0)
- Custom labels
- Unicode or ASCII
- Styling

**Integration Effort**: ⭐ Low (built-in widget)

**Value**: ⭐⭐ Medium (nice-to-have)

**Recommendation**: ⬆️ **ADOPT** - Add for progress indicators

---

### 3.3 BarChart, Sparkline, Canvas ⚠️ OPTIONAL

**Status**: ⚠️ **OPTIONAL** - Low priority

**Purpose**: Data visualization

**Use Cases**:
- Metrics visualization
- Statistics display
- Custom diagrams

**Integration Effort**: ⭐ Low (built-in)

**Value**: ⭐ Low (rare use case)

**Recommendation**: ⚠️ **DEFER** - Add only if needed

---

## 4. Summary of Decisions

### Immediate Actions

| Crate | Decision | Priority | Action |
|-------|----------|----------|--------|
| ratatui-textarea | ✅ KEEP | N/A | No action |
| ratatui-image | ✅ KEEP | N/A | No action |
| ratatui-explorer | ✅ KEEP | N/A | No action |
| tui-scrollview | ✅ KEEP | N/A | No action |
| tui-tree-widget | ⚠️ EVALUATE | 🔴 HIGH | Feature comparison |
| tui-popup | ⚠️ EVALUATE | 🟡 MEDIUM | Feature comparison |
| tui-logger | ⚠️ EVALUATE | 🟡 MEDIUM | Feature comparison |
| edtui | ⚠️ INVESTIGATE | 🔴 HIGH | Deep analysis |
| tui-input | ⛔ SKIP | N/A | No action |
| tui-realm | ⛔ SKIP | N/A | No action |
| Table widget | ⬆️ ADOPT | 🟡 MEDIUM | Implement usage |
| Gauge widget | ⬆️ ADOPT | 🟢 LOW | Implement usage |

### Research Tasks

1. **🔴 HIGH Priority**:
   - ⚠️ **tui-tree-widget vs custom**: Feature parity analysis
   - ⚠️ **edtui vs custom editor**: Migration cost vs benefit

2. **🟡 MEDIUM Priority**:
   - ⚠️ **tui-popup vs custom**: Feature comparison
   - ⚠️ **tui-logger vs custom**: Feature comparison
   - ⬆️ **Table widget**: Identify use cases and implement
   - ⬆️ **Gauge widget**: Identify use cases and implement

3. **🟢 LOW Priority**:
   - ⚠️ **BarChart/Sparkline/Canvas**: Defer until needed

---

## 5. Next Steps (Task 31.5 Continuation)

1. ✅ **Research complete** - Crate decisions documented
2. 🔄 **Awaiting** - Background agent final reports
3. ⏭️ **Next** - Create `tui-refactor-plan.md` with actionable items
4. ⏭️ **Next** - Update `tasks.md` marking Task 31.5 complete

---

## References

- **Cargo.toml**: Current dependencies verified
- **Source Analysis**: All `src/*.rs` files reviewed
- **Context7**: Ratatui docs and widget catalog
- **GitHub Examples**: Usage patterns from grep.app
