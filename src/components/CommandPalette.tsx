import { createSignal, createMemo } from "solid-js";

interface Props {
  onClose: () => void;
  onAction: (actionId: string) => void;
}

interface PaletteItem {
  id: string;
  icon: string;
  label: string;
  shortcut?: string;
}

export default function CommandPalette(props: Props) {
  const [search, setSearch] = createSignal("");
  const [selectedIndex, setSelectedIndex] = createSignal(0);

  const items: PaletteItem[] = [
    { id: "new-tab", icon: "➕", label: "New Tab", shortcut: "⌘T" },
    { id: "ai-bar", icon: "⚡", label: "AI Command Bar", shortcut: "⌘K" },
    { id: "snippets", icon: "📋", label: "Snippet Library", shortcut: "⌘⇧L" },
    { id: "toggle-sidebar", icon: "📂", label: "Toggle Sidebar", shortcut: "⌘B" },
    { id: "theme-hacker-green", icon: "🟢", label: "Theme: Hacker Green" },
    { id: "theme-cyberpunk", icon: "🟣", label: "Theme: Cyberpunk" },
    { id: "theme-matrix", icon: "⬛", label: "Theme: Matrix" },
    { id: "theme-tron", icon: "🔵", label: "Theme: Tron" },
    { id: "theme-ghost-protocol", icon: "👻", label: "Theme: Ghost Protocol" },
    { id: "theme-midnight", icon: "🌙", label: "Theme: Midnight" },
    { id: "toggle-crt", icon: "📺", label: "Toggle CRT Scanlines" },
    { id: "toggle-glow", icon: "✨", label: "Toggle Text Glow" },
    { id: "toggle-matrix", icon: "🌧️", label: "Toggle Matrix Rain" },
    { id: "toggle-particles", icon: "🎆", label: "Toggle Keystroke Particles" },
    { id: "toggle-hologram", icon: "🔮", label: "Toggle Hologram Effect" },
    { id: "settings", icon: "⚙️", label: "Settings", shortcut: "⌘," },
  ];

  const filtered = createMemo(() => {
    const q = search().toLowerCase();
    if (!q) return items;
    return items.filter(
      (item) =>
        item.label.toLowerCase().includes(q) || item.id.toLowerCase().includes(q)
    );
  });

  function execute(item: PaletteItem) {
    props.onAction(item.id);
    props.onClose();
  }

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
      if (list[selectedIndex()]) execute(list[selectedIndex()]);
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
              class={`palette-item ${index === selectedIndex() ? "selected" : ""}`}
              onClick={() => execute(item)}
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