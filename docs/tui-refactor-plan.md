# TUI Refactoring Plan

> **Research Status**: Task 31.5 - Ratatui Ecosystem Research COMPLETE
> **Date**: 2025-12-25
> **Purpose**: Actionable refactoring plan based on ratatui capabilities audit

## Executive Summary

Based on comprehensive research of ratatui core widgets, ecosystem crates, and current ricecoder-tui implementation, this plan identifies:

- ✅ **5 components to KEEP** (well-integrated, working)
- ⚠️ **4 components to EVALUATE** (custom vs ecosystem comparison needed)
- ⬆️ **2 ratatui widgets to ADOPT** (add missing built-ins)
- ⛔ **2 crates to SKIP** (redundant or unsuitable)

**Total Estimated Effort**: 3-5 days (medium priority items)

---

## Phase 1: High Priority Evaluations (2-3 days)

### 1.1 tui-tree-widget vs Custom Implementation ⚠️ HIGH

**Current State**:
- **Dependency**: `tui-tree-widget = { workspace = true }` (Cargo.toml line 56)
- **Custom Code**: `src/tree_widget.rs` (100+ lines)
- **Problem**: We have BOTH dependency AND custom implementation

**Evidence**:
```rust
// Custom Implementation
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
}
```

**tui-tree-widget Features**:
- Generic identifiers (not just `String`)
- Built-in `TreeState` (vs manual HashMap)
- Optimized rendering
- Actively maintained (last update May 29, 2025)

**Action Items**:
1. ✅ **Audit** `src/tree_widget.rs` - List ALL custom features
2. ✅ **Compare** - Map custom features to tui-tree-widget API
3. ✅ **Decide**:
   - IF tui-tree-widget covers 90%+ features → **Migrate**
   - IF custom has unique requirements → **Keep custom, remove dependency**
   - IF hybrid needed → **Use tui-tree-widget + thin wrapper**
4. ✅ **Implement** chosen approach
5. ✅ **Test** file picker, project navigation

**Estimated Effort**: 4-6 hours

**Priority**: 🔴 **HIGH** (dependency waste, maintenance burden)

**Success Criteria**:
- [ ] No unused dependencies
- [ ] Tree functionality unchanged or improved
- [ ] Tests pass (file picker, navigation)

---

### 1.2 edtui vs Custom Code Editor ⚠️ HIGH

**Current State**:
- **No Dependency**: edtui not in Cargo.toml
- **Custom Code**: `src/code_editor_widget.rs` exists
- **Problem**: Unknown if edtui would reduce maintenance

**edtui Features** (from research):
- Vim-inspired keybindings (full modal editing)
- Syntax highlighting (syntect integration)
- Mouse support
- Line numbers, search, clipboard
- Multi-file editing

**Action Items**:
1. ✅ **Read** `src/code_editor_widget.rs` - Document ALL features
2. ✅ **Compare** feature matrix:
   | Feature | Custom | edtui |
   |---------|--------|-------|
   | Syntax highlighting | ? | ✅ |
   | Vim mode | ? | ✅ |
   | Line numbers | ? | ✅ |
   | Multi-file | ? | ✅ |
   | Clipboard | ? | ✅ |
3. ✅ **Prototype** edtui integration (2 hours)
4. ✅ **Decide**:
   - IF edtui covers features + reduces code → **Adopt edtui**
   - IF custom is sufficient + well-tested → **Keep custom**
   - IF significant gaps → **Keep custom + enhance**
5. ✅ **Migrate** or **Document decision**

**Estimated Effort**: 6-8 hours (includes prototyping)

**Priority**: 🔴 **HIGH** (high-value widget, potential maintenance reduction)

**Success Criteria**:
- [ ] Decision documented with rationale
- [ ] If adopted: edtui integrated, tests pass
- [ ] If kept: Custom code documented, gaps identified

---

## Phase 2: Medium Priority Evaluations (1-2 days)

### 2.1 tui-popup vs Custom popup_widget.rs ⚠️ MEDIUM

**Current State**:
- **No Dependency**: tui-popup not in Cargo.toml
- **Custom Code**: `src/popup_widget.rs` exists
- **Research**: `tui-popup` crate exists (last updated Nov 2025, built-in alternative)

**Action Items**:
1. ✅ **Audit** `src/popup_widget.rs` - List features
2. ✅ **Research** - Built-in `Clear` widget vs `tui-popup` crate vs custom
3. ✅ **Compare**:
   | Approach | Pros | Cons |
   |----------|------|------|
   | Built-in `Clear` | No dependency | Limited features |
   | `tui-popup` crate | Feature-rich | Extra dependency |
   | Custom | Tailored | Maintenance burden |
4. ✅ **Decide** based on feature needs
5. ✅ **Implement** if changing

**Estimated Effort**: 2-3 hours

**Priority**: 🟡 **MEDIUM** (nice-to-have, not critical)

**Success Criteria**:
- [ ] Popup functionality unchanged or improved
- [ ] Minimal dependencies

---

### 2.2 tui-logger vs Custom logger_widget.rs ⚠️ MEDIUM

**Current State**:
- **No Dependency**: tui-logger not in Cargo.toml
- **Custom Code**: `src/logger_widget.rs` exists
- **Research**: tui-logger actively maintained, 1M+ downloads

**tui-logger Features**:
- Log level filtering (trace, debug, info, warn, error)
- Target filtering (by module)
- Circular buffer (memory-efficient)
- Smart widget (dual-pane UI)
- Integration with `log`, `tracing`, `slog`

**Action Items**:
1. ✅ **Audit** `src/logger_widget.rs` - List features
2. ✅ **Compare** - Feature matrix vs tui-logger
3. ✅ **Check** - Integration with ricecoder logging system
4. ✅ **Decide**:
   - IF tui-logger integrates cleanly → **Adopt**
   - IF custom has ricecoder-specific features → **Keep custom**
5. ✅ **Implement** chosen approach

**Estimated Effort**: 3-4 hours

**Priority**: 🟡 **MEDIUM** (debugging feature, tui-logger is production-proven)

**Success Criteria**:
- [ ] Logging widget functional
- [ ] Integration with ricecoder-activity-log maintained

---

### 2.3 Adopt ratatui Table Widget ⬆️ MEDIUM

**Current State**:
- **Not Used**: No Table widget in ricecoder-tui
- **Opportunity**: Built-in ratatui widget, zero dependency

**Use Cases** (identified):
- Session history listing (columns: ID, Date, Provider, Messages)
- Provider comparison (columns: Name, Status, Latency, Cost)
- Configuration display (columns: Key, Value, Source)
- Metrics/statistics (columns: Metric, Value, Change)

**Action Items**:
1. ✅ **Identify** 2-3 high-value use cases
2. ✅ **Design** table layouts for each use case
3. ✅ **Implement** Table widget wrapper (`src/table_widget.rs`)
4. ✅ **Integrate** into relevant screens
5. ✅ **Test** rendering, selection, scrolling

**Estimated Effort**: 4-5 hours

**Priority**: 🟡 **MEDIUM** (nice UX improvement, common pattern)

**Success Criteria**:
- [ ] Table widget implemented and documented
- [ ] At least 2 use cases integrated
- [ ] Selection and scrolling work

---

### 2.4 Adopt ratatui Gauge Widget ⬆️ LOW

**Current State**:
- **Not Used**: No Gauge widget in ricecoder-tui
- **Opportunity**: Built-in ratatui widget, zero dependency

**Use Cases** (identified):
- Token usage meter (progress toward limit)
- Download progress (agent tasks, file operations)
- Task completion (spec execution progress)
- Operation progress (indexing, analysis)

**Action Items**:
1. ✅ **Identify** 1-2 high-value use cases
2. ✅ **Implement** Gauge widget usage
3. ✅ **Integrate** into relevant screens

**Estimated Effort**: 2-3 hours

**Priority**: 🟢 **LOW** (nice-to-have, not essential)

**Success Criteria**:
- [ ] Gauge widget integrated in at least 1 use case
- [ ] Progress updates work smoothly

---

## Phase 3: Documentation & Cleanup (0.5 days)

### 3.1 Remove Unused Dependencies ✅

**Action Items**:
1. ✅ **Identify** unused crates in Cargo.toml
2. ✅ **Remove** from dependencies
3. ✅ **Verify** build succeeds

**Estimated Effort**: 30 minutes

**Priority**: 🟢 **LOW** (cleanup)

---

### 3.2 Document Widget Capabilities ✅

**Action Items**:
1. ✅ **Create** `docs/tui-widget-catalog.md` (link to ratatui-capabilities.md)
2. ✅ **Document** all ricecoder-tui widgets and their sources
3. ✅ **Update** README with widget overview

**Estimated Effort**: 1 hour

**Priority**: 🟢 **LOW** (developer documentation)

---

## Deferred Items (Not Needed)

### ⛔ Skip tui-input

**Reason**: Redundant with ratatui-textarea

**Rationale**:
- ratatui-textarea handles multi-line → can be limited to 1 line for single-line input
- Adding tui-input increases dependency count
- Feature overlap 100%

**Alternative**: Use `TextAreaWidget` with `max_height: 1`

---

### ⛔ Skip tui-realm

**Reason**: ricecoder-tui has mature component framework

**Rationale**:
- Existing framework in `src/components/` is comprehensive:
  - Component trait, ComponentRegistry
  - Event system (EventComponent, EventContext)
  - Lifecycle management
- Migration cost massive (refactor entire TUI)
- No clear benefit vs existing architecture

**Alternative**: Continue with current component system

---

## Dependencies Summary (After Refactoring)

### Keep (Already in Use)
| Crate | Version | Purpose | Status |
|-------|---------|---------|--------|
| ratatui-textarea | ✅ | Multi-line input | KEEP |
| ratatui-image | ✅ | Image rendering | KEEP |
| ratatui-explorer | ✅ | File picker | KEEP |
| tui-scrollview | ✅ | Scrollable content | KEEP |

### Evaluate (Pending Decision)
| Crate | Version | Purpose | Action |
|-------|---------|---------|--------|
| tui-tree-widget | ✅? | Tree view | EVALUATE (vs custom) |

### Consider Adding (Based on Evaluation)
| Crate | Purpose | Condition |
|-------|---------|-----------|
| edtui | Code editor | IF custom code editor insufficient |
| tui-logger | Log widget | IF custom logger missing features |
| tui-popup | Popup dialogs | IF built-in `Clear` insufficient |

### Skip (Redundant/Unsuitable)
| Crate | Reason |
|-------|--------|
| tui-input | Redundant with ratatui-textarea |
| tui-realm | Have own component framework |

---

## Timeline & Milestones

### Week 1 (High Priority)
- **Day 1-2**: tui-tree-widget evaluation + decision
- **Day 3-4**: edtui evaluation + decision
- **Day 5**: Buffer for migration work

### Week 2 (Medium Priority)
- **Day 1**: tui-popup + tui-logger evaluations
- **Day 2**: Table widget implementation
- **Day 3**: Gauge widget implementation

### Week 3 (Cleanup)
- **Day 1**: Documentation updates
- **Day 2**: Dependency cleanup, final testing

**Total**: 2-3 weeks (spread over development schedule)

---

## Success Metrics

### Quantitative
- [ ] Dependency count reduced or justified
- [ ] Code reduction: 200+ lines removed (if migrations successful)
- [ ] Test coverage maintained (85%+)
- [ ] Build time unchanged or improved

### Qualitative
- [ ] All widgets well-documented
- [ ] Clear rationale for custom vs ecosystem choices
- [ ] No feature regressions
- [ ] Improved maintainability

---

## Next Steps (Immediate)

1. ✅ **Update** `.ai/specs/opencode-migration/tasks.md` - Mark Task 31.5 complete
2. ✅ **Create** subtasks for Phase 1 evaluations
3. ✅ **Prioritize** in development backlog
4. ✅ **Start** with tui-tree-widget evaluation (highest priority)

---

## References

- **Research Docs**: `ratatui-capabilities.md`, `tui-crate-decisions.md`
- **Source Code**: `projects/ricecoder/crates/ricecoder-tui/src/`
- **Dependencies**: `projects/ricecoder/crates/ricecoder-tui/Cargo.toml`
- **Ecosystem Research**: Background agent reports (tasks bg_7c6e935c, bg_83613411)
