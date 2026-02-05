# Rivulet TUI Keyboard Shortcuts

## Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down (next item) |
| `k` / `↑` | Move up (previous item) |
| `n` / `PageDown` | Jump down 10 items |
| `p` / `PageUp` | Jump up 10 items |
| `Tab` | Switch to next pane (Feeds → Items → Preview) |
| `Shift+Tab` | Switch to previous pane |

## Actions

| Key | Action |
|-----|--------|
| `Enter` | Select feed (load its items) |
| `r` | Toggle read/unread status |
| `s` | Toggle star/unstar |
| `o` | Open item link in browser (marks as read) |
| `R` | Refresh all feeds |

## View

| Key | Action |
|-----|--------|
| `m` | Toggle maximize mode (fullscreen preview) |

## General

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+C` | Quit |

## Panes

```
┌─────────────────────────────────────┐
│   FEEDS PANE                        │  ← List of subscribed feeds
├─────────────────────────────────────┤
│   ITEMS PANE                        │  ← Articles from selected feed
├─────────────────────────────────────┤
│   PREVIEW PANE                      │  ← Article content
├─────────────────────────────────────┤
│   STATUS BAR                        │  ← Shortcuts hint
└─────────────────────────────────────┘
```

## Item Markers

| Marker | Meaning |
|--------|---------|
| `●` | Unread item |
| `★` | Starred item |
| ` ` | Read item (no marker) |

## CLI Options

```bash
# Set number of parallel workers for fetching
rivulet --workers 20 import feeds.opml
rivulet -w 15 update

# Default is 10 workers
rivulet import feeds.opml
```
