import { Show } from "solid-js";

interface Props {
  visible: boolean;
  onClose: () => void;
}

export default function ShortcutHelp(props: Props) {
  if (!props.visible) return null;

  const sections = [
    {
      title: "General",
      icon: "⚡",
      shortcuts: [
        { keys: "⌘/Ctrl + K", action: "AI Command Bar" },
        { keys: "⌘/Ctrl + P", action: "Command Palette" },
        { keys: "⌘/Ctrl + ,", action: "Settings" },
        { keys: "⌘/Ctrl + B", action: "Toggle Sidebar" },
        { keys: "Escape", action: "Close any overlay" },
      ],
    },
    {
      title: "Tabs",
      icon: "📑",
      shortcuts: [
        { keys: "⌘/Ctrl + T", action: "New Tab" },
        { keys: "⌘/Ctrl + W", action: "Close Tab" },
      ],
    },
    {
      title: "Terminal",
      icon: "🖥️",
      shortcuts: [
        { keys: "Ctrl + Space", action: "Autocomplete" },
        { keys: "↑ / ↓", action: "Navigate suggestions" },
        { keys: "Enter / Tab", action: "Accept suggestion" },
        { keys: "Ctrl + C", action: "Cancel command" },
        { keys: "Ctrl + L", action: "Clear screen" },
        { keys: "Ctrl + U", action: "Clear line" },
        { keys: "Ctrl + A", action: "Jump to line start" },
        { keys: "Ctrl + E", action: "Jump to line end" },
      ],
    },
    {
      title: "Tools",
      icon: "🛠️",
      shortcuts: [
        { keys: "⌘/Ctrl + Shift + L", action: "Snippet Library" },
        { keys: "⌘/Ctrl + M", action: "MCP Servers Panel" },
        { keys: "⌘/Ctrl + Shift + C", action: "AI + MCP Chat" },
        { keys: "⌘/Ctrl + Shift + N", action: "NETOPS Dashboard" },
        { keys: "⌘/Ctrl + Shift + B", action: "BharatLink P2P Share" },
      ],
    },
    {
      title: "AI Bar",
      icon: "🤖",
      shortcuts: [
        { keys: "Enter", action: "Submit query" },
        { keys: "▶ Run", action: "Execute AI command" },
        { keys: "📋 Copy", action: "Copy to clipboard" },
        { keys: "💡 Explain", action: "Get explanation" },
      ],
    },
    {
      title: "Command Palette",
      icon: "🔍",
      shortcuts: [
        { keys: "↑ / ↓", action: "Navigate items" },
        { keys: "Enter", action: "Execute selected" },
        { keys: "Type to filter", action: "Search commands" },
      ],
    },
  ];

  return (
    <div class="shortcut-overlay" onClick={() => props.onClose()}>
      <div class="shortcut-panel" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="shortcut-header">
          <div class="shortcut-header-left">
            <span class="shortcut-header-icon">⌨️</span>
            <span>Keyboard Shortcuts</span>
          </div>
          <span class="shortcut-close" onClick={() => props.onClose()}>
            ×
          </span>
        </div>

        {/* Grid */}
        <div class="shortcut-grid">
          {sections.map((section) => (
            <div class="shortcut-section">
              <div class="shortcut-section-title">
                <span>{section.icon}</span>
                <span>{section.title}</span>
              </div>
              <div class="shortcut-list">
                {section.shortcuts.map((s) => (
                  <div class="shortcut-row">
                    <span class="shortcut-keys">
                      {s.keys.split(" + ").map((key, i) => (
                        <>
                          {i > 0 && <span class="shortcut-plus">+</span>}
                          <kbd class="shortcut-kbd">{key.trim()}</kbd>
                        </>
                      ))}
                    </span>
                    <span class="shortcut-action">{s.action}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div class="shortcut-footer">
          <span>Press <kbd class="shortcut-kbd">?</kbd> or click <kbd class="shortcut-kbd">ℹ️</kbd> in status bar to toggle</span>
        </div>
      </div>
    </div>
  );
}