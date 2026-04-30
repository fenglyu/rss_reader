# Plan — Enter-progression bug fix + Left/Right arrow pane navigation

## Context

User reports two coupled UX issues with the Reader tab navigation:

1. **Bug** — In the Reader tab, the expected workflow is:
   - Enter on a feed in the Feeds pane → focus moves to Items pane (with items loaded)
   - Enter on an item in the Items pane → focus moves to Preview pane

   Today, only the first half works. `Action::Select` in `src/tui/mod.rs:128-135` matches `(Reader, Feeds)` only; there is **no arm** for `(Reader, Items) → Preview` or `(Latest, Items) → Preview`. Enter on an item does nothing — the workflow stalls. The user described this as "nothing pops up in the feed panel."

2. **Feature** — User wants a simpler single-keypress alternative to the existing `Ctrl+W h/l` chord: bind `Left`/`Right` arrow keys to "move focus one pane left / right" with the same load-on-first-step semantics (Right from Feeds with no feed loaded should load and land on Items, not skip to Preview).

Both arrow keys are currently unbound (defaults use `Up`/`Down` for vertical list movement and `Tab` / `Shift+Tab` for pane cycling).

## Recommended approach

### A. Bug fix — extend `Action::Select`

Replace the single-arm match in `src/tui/mod.rs:128-135` with a 2-D match on `(active_tab, active_pane)`:

```rust
Action::Select => match (tui_app.active_tab, tui_app.active_pane) {
    (AppTab::Reader, ActivePane::Feeds) => {
        if load_items_for_highlighted_feed(&mut tui_app, &ctx)? {
            tui_app.active_pane = ActivePane::Items;
        }
    }
    (_, ActivePane::Items) => {
        if tui_app.selected_item_for_active_tab().is_some() {
            tui_app.active_pane = ActivePane::Preview;
        }
    }
    _ => {}
}
```

The `(_, Items)` arm covers both Reader and Latest tabs symmetrically — Latest doesn't have a Feeds pane but does have Items+Preview, and the same workflow makes sense there.

### B. Feature — `Left`/`Right` arrow keys for directional focus

The existing `Ctrl+W h/l` chord already does exactly what the user wants:

1. Sync items for the highlighted feed (so `loaded_feed.is_some()` reflects current state)
2. Compute target via `focus_left_for_tab` / `focus_right_for_tab`
3. Call `focus_pane(tui_app, ctx, target)`

The plan is to **extract this into a single helper** and bind it to both the chord arms and new `Action::FocusLeft`/`Action::FocusRight` actions.

#### B1. New Actions (`src/tui/event.rs`)

Add two variants to `Action`:

```rust
FocusLeft,
FocusRight,
```

#### B2. Keybinding fields (`src/config/keybindings.rs`)

Add to `KeybindingConfig`:

```rust
pub pane_left: Vec<String>,
pub pane_right: Vec<String>,
```

Defaults:

```rust
pane_left: vec!["Left".to_string()],
pane_right: vec!["Right".to_string()],
```

Add to `get_action`:

```rust
} else if self.matches_key(key, &self.pane_left) {
    Action::FocusLeft
} else if self.matches_key(key, &self.pane_right) {
    Action::FocusRight
```

Extend the `test_keybinding_config_get_action` test with two new assertions for `Left` → `FocusLeft`, `Right` → `FocusRight`.

#### B3. Shared helper (`src/tui/mod.rs`)

Extract from chord handler `h`/`l` arms:

```rust
fn focus_pane_directional(
    tui_app: &mut TuiApp,
    ctx: &AppContext,
    target_fn: fn(&TuiApp) -> ActivePane,
) -> Result<bool> {
    if tui_app.active_tab == AppTab::Reader {
        load_items_for_highlighted_feed(tui_app, ctx)?;
    }
    let target = target_fn(tui_app);
    if target == tui_app.active_pane {
        Ok(false)
    } else {
        focus_pane(tui_app, ctx, target)?;
        Ok(true)
    }
}
```

Refactor chord `h`/`l` arms to use it. Add new dispatch arms:

```rust
Action::FocusLeft => {
    focus_pane_directional(&mut tui_app, &ctx, focus_left_for_tab)?;
}
Action::FocusRight => {
    focus_pane_directional(&mut tui_app, &ctx, focus_right_for_tab)?;
}
```

(No status-bar message; arrow keys are silent navigation. The chord arms can keep their "Already at leftmost/rightmost" status messages by branching on the bool return.)

#### B4. Sample config + bundled default

- `src/config/mod.rs` `default_config_content()` — add lines under `[keybindings]`:
  ```
  pane_left = ["Left"]
  pane_right = ["Right"]
  ```
- `config.sample.toml` — same.

### C. Documentation

- `README.md` — add two rows to the "Pane & tab navigation" table:
  - `←` / `Left` — focus pane to the left
  - `→` / `Right` — focus pane to the right
  Note that `Enter` now also advances Items → Preview.
- `SHORTCUTS.md` — same additions.

## Files to modify

| File | Change |
|---|---|
| `src/tui/event.rs` | + `FocusLeft`, `FocusRight` variants |
| `src/config/keybindings.rs` | + 2 fields, + 2 defaults, + 2 dispatch branches, + test asserts |
| `src/config/mod.rs` | + 2 lines in `default_config_content()` |
| `config.sample.toml` | + 2 lines |
| `src/tui/mod.rs` | extend `Action::Select`, extract `focus_pane_directional`, refactor chord arms, + 2 new dispatch arms |
| `src/tui/mod.rs::tests` | + 3 tests (see verification) |
| `README.md` | + 2 rows in keybindings table; note Enter on item |
| `SHORTCUTS.md` | + 2 rows |

## Functions to reuse (no rework needed)

- `load_items_for_highlighted_feed` (`src/tui/mod.rs`) — already idempotent; safe to call before any focus shift in Reader.
- `focus_left_for_tab` / `focus_right_for_tab` (`src/tui/mod.rs`) — already correct after the previous refactor (use `loaded_feed.is_some()`).
- `focus_pane` (`src/tui/mod.rs`) — already calls `load_items_for_highlighted_feed` for non-Feeds targets; the directional helper relies on this.
- `selected_item_for_active_tab` (`src/tui/app.rs`) — used by the Items → Preview branch.

## Verification

### Unit tests (added to `src/tui/mod.rs::tests`)

1. **`select_advances_feeds_to_items`** — Reader+Feeds, no feed yet loaded; `Action::Select` flow; assert `active_pane == Items`, `loaded_feed_id() == Some(first_feed_id)`, `loaded_items().len() > 0`.
2. **`select_advances_items_to_preview`** — Reader+Items with items loaded; `Action::Select` flow; assert `active_pane == Preview`. Repeat for Latest tab.
3. **`right_arrow_walks_feeds_to_preview_loading_items_on_first_step`** — start at Reader+Feeds with `loaded_feed = None`; call `focus_pane_directional(focus_right_for_tab)` once → expect `Items` (with items loaded), again → expect `Preview`. Then `focus_left_for_tab` walks back: Preview → Items → Feeds.

(All three are pure state-machine tests; no TUI harness needed. Pattern: `AppContext::in_memory()` + `add_feed_with_items` helper already exists in the test module.)

### Manual smoke test

```bash
cargo run -- tui
```

1. Add a couple of feeds (`rivulet add <URL>` first, then `update`).
2. Press `]` → Reader tab opens.
3. Press `Enter` on a feed → Items pane should populate and gain focus.
4. Press `Enter` on an item → Preview pane should gain focus and show the item.
5. Press `←` → focus moves back to Items.
6. Press `←` again → focus moves to Feeds.
7. Press `→` from Feeds with a different feed highlighted → items load and focus lands on Items.
8. `Tab` and `Ctrl+W h/l` should behave the same as before (no regressions).

### CI

```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

## Risk / out-of-scope

- The user described the bug as "nothing pops up in the feed panel" which is ambiguous. My read: they meant the Items pane stays empty when they Enter on a feed. The trace shows that Enter-on-feed actually does load items today — but it's possible the user pressed Enter on an item (which does nothing) and described that as "feed panel broken." Adding the Items → Preview arm fixes the latter unambiguously. If the Enter-on-feed path is actually broken in some edge case I missed, the new test `select_advances_feeds_to_items` will surface it.
- No changes to the existing Tab / Shift+Tab cycling, no changes to the Ctrl+W chord behavior, no changes to LoadedFeed structure.
