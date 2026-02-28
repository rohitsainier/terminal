import { createSignal, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  onClose: () => void;
}

interface PaletteItem {
  id: string;
  icon: string;
  label: string;
  shortcut?: string;
  action: () => void;
}

export default function CommandPalette(props: Props) {
  const [search, setSearch] = createSignal("");
  const [selectedIndex, setSelectedIndex] = createSignal(0);

  const items: PaletteItem[] = [
    {
      id: "new-tab",
      icon: "➕",
      label: "New Tab",
      shortcut: "⌘T",
      action: () => props.onClose(),
    },
    {
      id: "split-v",
      icon: "▐",
      label: "Split Pane Vertical",
      shortcut: "⌘D",
      action: () => props.onClose(),
    },
    {
      id: "split-h",
      icon: "▄",
      label: "Split Pane Horizontal",
      shortcut: "⌘⇧D",
      action: () => props.onClose(),
    },
    {
      id: "ai-bar",
      icon: "⚡",
      label: "AI Command Bar",
      shortcut: "⌘K",
      action: () => props.onClose(),
    },
    {
      id: "snippets",
      icon: "📋",
      label: "Snippet Library",
      shortcut: "⌘⇧L",
      action: () => props.onClose(),
    },
    {
      id: "theme-hacker",
      icon: "🟢",
      label: "Theme: Hacker Green",
      action: () => {
        invoke("set_config", { config: { theme: "hacker-green" } });
        props.onClose();
      },
    },
    {
      id: "theme-cyberpunk",
      icon: "🟣",
      label: "Theme: Cyberpunk",
      action: () => {
        invoke("set_config", { config: { theme: "cyberpunk" } });
        props.onClose();
      },
    },
    {
      id: "theme-matrix",
      icon: "⬛",
      label: "Theme: Matrix",
      action: () => {
        invoke("set_config", { config: { theme: "matrix" } });
        props.onClose();
      },
    },
    {
      id: "theme-tron",
      icon: "🔵",
      label: "Theme: Tron",
      action: () => {
        invoke("set_config", { config: { theme: "tron" } });
        props.onClose();
      },
    },
    {
      id: "theme-ghost",
      icon: "👻",
      label: "Theme: Ghost Protocol",
      action: () => {
        invoke("set_config", { config: { theme: "ghost-protocol" } });
        props.onClose();
      },
    },
    {
      id: "toggle-crt",
      icon: "📺",
      label: "Toggle CRT Scanlines",
      action: () => props.onClose(),
    },
    {
      id: "toggle-glow",
      icon: "✨",
      label: "Toggle Text Glow",
      action: () => props.onClose(),
    },
    {
      id: "toggle-matrix",
      icon: "🌧️",
      label: "Toggle Matrix Rain",
      action: () => props.onClose(),
    },
    {
      id: "settings",
      icon: "⚙️",
      label: "Settings",
      shortcut: "⌘,",
      action: () => props.onClose(),
    },
  ];

  const filtered = createMemo(() => {
    const q = search().toLowerCase();
    if (!q) return items;
    return items.filter(
      (item) =>
        item.label.toLowerCase().includes(q) ||
        item.id.toLowerCase().includes(q)
    );
  });

  function handleKeyDown(e: KeyboardEvent) {
    const list = filtered();
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, list.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (list[selectedIndex()]) {
        list[selectedIndex()].action();
      }
    }
  }

  return (
    <div class="palette-overlay" onClick={() => props.onClose()}>
      <div class="palette" onClick={(e) => e.stopPropagation()}>
        <input
          class="palette-input"
          type="text"
          placeholder="Search commands..."
          value={search()}
          onInput={(e) => {
            setSearch(e.currentTarget.value);
            setSelectedIndex(0);
          }}
          onKeyDown={handleKeyDown}
          autofocus
        />
        <div class="palette-items">
          {filtered().map((item, index) => (
            <div
              class={`palette-item ${
                index === selectedIndex() ? "selected" : ""
              }`}
              onClick={() => item.action()}
              onMouseEnter={() => setSelectedIndex(index)}
            >
              <span class="palette-item-icon">{item.icon}</span>
              <span>{item.label}</span>
              {item.shortcut && (
                <span class="palette-item-shortcut">{item.shortcut}</span>
              )}
            </div>
          ))}
          {filtered().length === 0 && (
            <div class="palette-item" style={{ opacity: "0.4" }}>
              No results found
            </div>
          )}
        </div>
      </div>
    </div>
  );
}