# Ratatui Capabilities Reference

> **Research Status**: Task 31.5 - Ratatui Ecosystem Research
> **Date**: 2025-12-25
> **Sources**: Context7 docs, GitHub examples, Cargo.toml analysis

## Overview

Ratatui is a Rust library for building rich TUI applications. This document catalogs ALL built-in widgets, layout capabilities, styling systems, and ecosystem crates to inform ricecoder-tui architecture decisions.

---

## 1. Built-in Widgets (ratatui core)

### 1.1 Text Display Widgets

#### **Block**
- **Purpose**: Container with borders, title, and styling
- **Use Cases**: Widget wrapping, visual organization, UI structure
- **Key Methods**:
  - `Block::bordered()` - Create with all borders
  - `Block::title()` - Set title (top, bottom, left, right)
  - `Block::borders()` - Configure borders (ALL, NONE, LEFT, RIGHT, TOP, BOTTOM)
  - `Block::border_style()` - Custom border styling
  - `Block::padding()` - Internal padding (horizontal, vertical, uniform)
  - `Block::inner()` - Get drawable area inside block
- **Configuration**:
  - TitlePosition: Top, Bottom
  - Alignment: Left, Center, Right
  - BorderType: Plain, Rounded, Double, Thick
- **Current Usage**: Used throughout ricecoder-tui

#### **Paragraph**
- **Purpose**: Multi-line text rendering with wrapping and scrolling
- **Use Cases**: Messages, documentation, logs, markdown display
- **Key Methods**:
  - `Paragraph::new(text)` - Create from Text
  - `Paragraph::wrap()` - Enable word wrapping (Wrap::Trim)
  - `Paragraph::scroll()` - Scroll position (row, col)
  - `Paragraph::alignment()` - Text alignment
  - `Paragraph::block()` - Wrap with block
- **Configuration**:
  - Wrap modes: None, Trim (trim whitespace)
  - Alignment: Left, Center, Right, Justified
- **Current Usage**: Used in ricecoder-tui chat display

### 1.2 Interactive Widgets

#### **List**
- **Purpose**: Selectable list with keyboard navigation
- **Use Cases**: File lists, menus, options, item selection
- **Key Methods**:
  - `List::new(items)` - Create from ListItem collection
  - `List::highlight_style()` - Selected item style
  - `List::highlight_symbol()` - Prefix for selected item (e.g., ">>")
  - `List::repeat_highlight_symbol()` - Repeat symbol across row
  - `List::direction()` - TopToBottom or BottomToTop
- **State Management**: `ListState` (selected index, offset)
- **Current Usage**: Used in ricecoder-tui components/list.rs

#### **Table**
- **Purpose**: Tabular data with columns and rows
- **Use Cases**: Data grids, structured content, comparisons
- **Key Methods**:
  - `Table::new(rows, widths)` - Create with Row collection
  - `Table::header()` - Set header row
  - `Table::widths()` - Column width constraints
  - `Table::column_spacing()` - Space between columns
  - `Table::highlight_style()` - Selected row style
  - `Table::highlight_symbol()` - Row selection indicator
- **State Management**: `TableState` (selected row, offset)
- **Current Usage**: **NOT USED** - Consider adding

#### **Tabs**
- **Purpose**: Tab navigation interface
- **Use Cases**: Multi-page UIs, section switching
- **Key Methods**:
  - `Tabs::new(titles)` - Create from title list
  - `Tabs::select()` - Set active tab index
  - `Tabs::highlight_style()` - Active tab style
  - `Tabs::divider()` - Separator between tabs
- **Configuration**:
  - Style: Default, highlighted
  - Divider: Custom string (e.g., " | ")
- **Current Usage**: Used in ricecoder-tui components/tabs.rs

### 1.3 Data Visualization Widgets

#### **Gauge**
- **Purpose**: Progress bar with percentage display
- **Use Cases**: Download progress, task completion, metrics
- **Key Methods**:
  - `Gauge::default().ratio(f64)` - Set progress (0.0-1.0)
  - `Gauge::label()` - Custom label text
  - `Gauge::gauge_style()` - Filled portion style
  - `Gauge::use_unicode()` - Enable unicode block chars
- **Configuration**:
  - Unicode vs ASCII rendering
  - Custom label formatting
- **Current Usage**: **NOT USED** - Consider for progress tracking

#### **BarChart**
- **Purpose**: Categorical bar chart
- **Use Cases**: Metrics comparison, statistics
- **Key Methods**:
  - `BarChart::default().data()`
  - `BarChart::bar_width()`, `bar_gap()`, `group_gap()`
  - `BarChart::max()` - Y-axis maximum
  - `BarChart::value_style()`, `label_style()`
- **Current Usage**: **NOT USED**

#### **Sparkline**
- **Purpose**: Inline mini chart
- **Use Cases**: Trend visualization, compact metrics
- **Current Usage**: **NOT USED**

### 1.4 Drawing Widgets

#### **Canvas**
- **Purpose**: Custom drawing with shapes and primitives
- **Use Cases**: Diagrams, graphs, custom visualizations
- **Key Methods**:
  - `Canvas::default().paint()` - Custom render function
  - Drawing primitives: Line, Rectangle, Circle, Points
- **Current Usage**: **NOT USED**

#### **Clear**
- **Purpose**: Clear specific area of terminal
- **Use Cases**: Partial screen updates, widget removal
- **Current Usage**: **NOT USED** - handled by ratatui internally

---

## 2. Layout System

### 2.1 Layout Builder

```rust
// Vertical split
let [header, body, footer] = Layout::vertical([
    Constraint::Length(3),      // Fixed 3 lines
    Constraint::Fill(1),         // Take remaining space
    Constraint::Length(1),       // Fixed 1 line
]).areas(frame.area());

// Horizontal split
let [sidebar, main] = Layout::horizontal([
    Constraint::Percentage(20),  // 20% width
    Constraint::Percentage(80),  // 80% width
]).areas(body);
```

### 2.2 Constraint Types

| Constraint | Description | Use Case |
|------------|-------------|----------|
| `Length(n)` | Fixed size in cells | Headers, footers, borders |
| `Percentage(n)` | Percentage of available space | Proportional splits |
| `Ratio(a, b)` | Ratio of available space | Complex proportions |
| `Min(n)` | Minimum size | Ensure minimum widget size |
| `Max(n)` | Maximum size | Cap widget growth |
| `Fill(n)` | Fill available space | Main content areas |

### 2.3 Flex Modes

- **Flex::Start** (default): Align areas to start
- **Flex::Center**: Center areas
- **Flex::End**: Align to end
- **Flex::SpaceEvenly**: Even spacing
- **Flex::SpaceBetween**: Space between areas
- **Flex::Legacy**: Old stretch behavior (deprecated)

### 2.4 Layout Macros

```rust
use ratatui_macros::{vertical, horizontal, constraints};

let [top, main, bottom] = vertical![==1, *=1, >=3].areas(area);
let [left, main, right] = horizontal![>=20, *=1, >=20].areas(main);
```

**Current Usage**: ricecoder-tui uses manual Layout API, not macros

---

## 3. Styling System

### 3.1 Style Builder

```rust
use ratatui::style::{Style, Color, Modifier};

let style = Style::new()
    .fg(Color::Cyan)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
```

### 3.2 Color Support

**Named Colors**: Black, Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, White

**RGB Colors**: `Color::Rgb(r, g, b)`

**Indexed Colors**: `Color::Indexed(n)` (0-255)

**Current Usage**: ricecoder-tui has comprehensive theme system in `style.rs` and `theme.rs`

### 3.3 Modifiers

- `BOLD`, `DIM`, `ITALIC`, `UNDERLINED`
- `SLOW_BLINK`, `RAPID_BLINK`
- `REVERSED`, `HIDDEN`, `CROSSED_OUT`

### 3.4 Stylize Trait (Convenience)

```rust
use ratatui::style::Stylize;

let text = "Hello".cyan().bold();
let block = Block::bordered().title("Title".green());
```

**Current Usage**: Used throughout ricecoder-tui

---

## 4. Event Handling

### 4.1 crossterm Integration

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

if event::poll(Duration::from_millis(100))? {
    match event::read()? {
        Event::Key(key) => handle_key(key),
        Event::Mouse(mouse) => handle_mouse(mouse),
        Event::Resize(w, h) => handle_resize(w, h),
        _ => {}
    }
}
```

**Current Usage**: ricecoder-tui has event system in `event.rs` and `event_dispatcher.rs`

### 4.2 Input Abstraction

**Common Pattern**: Convert `crossterm::event::KeyEvent` to application-specific actions

**Current Usage**: ricecoder-tui uses `KeyEvent` directly plus custom `InputEvent` enum

---

## 5. Rendering Lifecycle

### 5.1 Frame-based Rendering

```rust
terminal.draw(|frame| {
    frame.render_widget(widget, area);
    frame.render_stateful_widget(list, area, &mut state);
})?;
```

### 5.2 Widget vs WidgetRef

- **Widget trait**: Consumes self during rendering
- **WidgetRef trait**: Renders by reference (deprecated in favor of `impl Widget for &Type`)

**Current Usage**: ricecoder-tui uses Widget trait

### 5.3 StatefulWidget

For widgets with mutable state (List, Table, Tree, etc.)

```rust
impl StatefulWidget for MyWidget {
    type State = MyState;
    
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render with state mutation
    }
}
```

**Current Usage**: Used in ricecoder-tui for stateful components

---

## 6. Ecosystem Crates (Already in Use)

### 6.1 ratatui-textarea ✅

**Status**: **ALREADY IN USE** (Cargo.toml line 55)

**Purpose**: Multi-line text input with vim mode support

**Features**:
- Multi-line editing
- Vim mode (normal, insert, visual)
- Undo/redo history
- Word wrapping
- Syntax highlighting support
- Search and replace

**Current Integration**: `src/textarea_widget.rs` (245 lines)
- Custom wrapper around `TextArea`
- Vim mode state management
- Selection support
- History draft preservation

**Recommendation**: ✅ **KEEP** - Core feature, well-integrated

### 6.2 tui-tree-widget ✅

**Status**: **ALREADY IN USE** (Cargo.toml line 56)

**Purpose**: Tree view for hierarchical data

**Features**:
- Expand/collapse nodes
- Keyboard navigation
- Custom item rendering
- State management (TreeState)

**Current Integration**: `src/tree_widget.rs` (custom implementation, 100+ lines)
- Custom TreeNode and TreeWidget
- HashMap-based storage
- Visible node tracking

**Recommendation**: ⚠️ **EVALUATE** - Custom implementation vs tui-tree-widget features

### 6.3 ratatui-image ✅

**Status**: **ALREADY IN USE** (Cargo.toml line 57)

**Purpose**: Image rendering in terminals

**Features**:
- Protocol support: Sixel, Kitty, iTerm2
- Image formats: PNG, JPG, GIF
- Automatic protocol detection
- Resize and crop

**Current Integration**: `src/image_widget.rs` and `src/image_integration.rs`

**Recommendation**: ✅ **KEEP** - Unique feature

### 6.4 ratatui-explorer ✅

**Status**: **ALREADY IN USE** (Cargo.toml line 58)

**Purpose**: File system explorer widget

**Features**:
- File browser UI
- Keyboard navigation
- File filtering
- Theme customization

**Current Integration**: Used in `src/file_picker.rs`

**Recommendation**: ✅ **KEEP** - Provides file picker functionality

### 6.5 tui-scrollview ✅

**Status**: **ALREADY IN USE** (Cargo.toml line 59)

**Purpose**: Scrollable content container

**Features**:
- Vertical and horizontal scrolling
- Scroll state management
- Viewport tracking

**Current Integration**: `src/scrollview_widget.rs` (custom wrapper)

**Recommendation**: ✅ **KEEP** - Essential for chat display

---

## 7. Ecosystem Crates (NOT in Use - Evaluation Needed)

### 7.1 tui-input ❌

**Status**: NOT USED

**Purpose**: Single-line input widget

**Reason to Skip**: We already have ratatui-textarea for multi-line input. Single-line can be handled with textarea limited to 1 line.

**Recommendation**: ⛔ **SKIP** - Redundant with ratatui-textarea

### 7.2 tui-popup ❌

**Status**: NOT USED

**Purpose**: Popup dialog widgets

**Features**:
- Modal dialogs
- Centered positioning
- Backdrop dimming

**Current Alternative**: ricecoder-tui has `popup_widget.rs` (custom implementation)

**Recommendation**: ✅ **EVALUATE** - Compare custom vs tui-popup features

### 7.3 tui-logger ❌

**Status**: NOT USED

**Purpose**: Log viewer widget with filtering

**Features**:
- Log level filtering
- Scrollable log display
- Circular buffer

**Current Alternative**: ricecoder-tui has `logger_widget.rs` (custom implementation)

**Recommendation**: ✅ **EVALUATE** - Feature comparison needed

### 7.4 edtui ❌

**Status**: NOT USED

**Purpose**: Full text editor widget (vim-inspired)

**Features**:
- Multi-file editing
- Syntax highlighting
- Vim keybindings
- Modal editing

**Current Alternative**: ricecoder-tui has `code_editor_widget.rs` (custom implementation)

**Recommendation**: ⚠️ **INVESTIGATE** - Significant overlap, feature comparison needed

### 7.5 tui-realm ❌

**Status**: NOT USED

**Purpose**: Component framework for ratatui (React/Elm inspired)

**Features**:
- Component lifecycle
- Props and state
- Message passing
- Event system

**Current Alternative**: ricecoder-tui has comprehensive component system in `components/`

**Recommendation**: ⛔ **SKIP** - ricecoder-tui has mature component architecture

---

## 8. Performance Considerations

### 8.1 Rendering Optimization

**Built-in Features**:
- Diff-based rendering (only changed cells)
- Double buffering
- Viewport culling

**Best Practices**:
- Cache computed layouts
- Use `Clear` widget sparingly
- Minimize full redraws

**Current Usage**: ricecoder-tui has `render_pipeline.rs` for optimization

### 8.2 State Management

**Stateful Widgets**: Separate state from widget for efficient re-rendering

**Pattern**:
```rust
struct MyWidget { /* immutable config */ }
struct MyState { /* mutable state */ }

impl StatefulWidget for MyWidget {
    type State = MyState;
    // render with &mut state
}
```

**Current Usage**: Used in ricecoder-tui components

---

## 9. Gaps & Recommendations

### 9.1 Missing ratatui Built-ins

| Widget | Status | Recommendation |
|--------|--------|----------------|
| Table | Not used | ✅ ADD for structured data display |
| Gauge | Not used | ✅ ADD for progress indicators |
| BarChart | Not used | ⚠️ OPTIONAL - low priority |
| Sparkline | Not used | ⚠️ OPTIONAL - low priority |
| Canvas | Not used | ⚠️ OPTIONAL - advanced use cases |

### 9.2 Ecosystem Crate Decisions

| Crate | Current | Recommendation |
|-------|---------|----------------|
| ratatui-textarea | ✅ In use | ✅ KEEP |
| tui-tree-widget | ✅ In use, custom wrapper | ⚠️ EVALUATE vs custom |
| ratatui-image | ✅ In use | ✅ KEEP |
| ratatui-explorer | ✅ In use | ✅ KEEP |
| tui-scrollview | ✅ In use | ✅ KEEP |
| tui-input | ❌ Not used | ⛔ SKIP (redundant) |
| tui-popup | ❌ Not used | ✅ EVALUATE |
| tui-logger | ❌ Not used | ✅ EVALUATE |
| edtui | ❌ Not used | ⚠️ INVESTIGATE |
| tui-realm | ❌ Not used | ⛔ SKIP (have own framework) |

### 9.3 Custom vs Ecosystem Trade-offs

**Custom Implementations Found**:
1. `tree_widget.rs` - Custom tree (vs tui-tree-widget)
2. `popup_widget.rs` - Custom popup (vs tui-popup)
3. `logger_widget.rs` - Custom logger (vs tui-logger)
4. `code_editor_widget.rs` - Custom editor (vs edtui)

**Decision Criteria**:
- ✅ **Use ecosystem** if: Maintained, feature-rich, good API
- ✅ **Keep custom** if: Tight integration, specific requirements, performance
- ⚠️ **Evaluate** if: Significant feature overlap

---

## 10. Next Steps (Task 31.5 Continuation)

1. ✅ **Research complete** - Core ratatui documented
2. 🔄 **Awaiting** - Background agent research on ecosystem crates
3. ⏭️ **Next** - Audit ricecoder-tui components against ratatui capabilities
4. ⏭️ **Next** - Create `tui-crate-decisions.md` with evaluation criteria
5. ⏭️ **Next** - Create `tui-refactor-plan.md` with actionable items

---

## References

- **Context7 Docs**: `/ratatui/ratatui` (widgets, layout snippets)
- **Cargo.toml**: `ricecoder-tui/Cargo.toml` (current dependencies)
- **Source Code**: `ricecoder-tui/src/` (current implementations)
- **GitHub Examples**: grep.app search results (usage patterns)
