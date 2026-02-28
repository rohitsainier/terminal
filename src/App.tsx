import { createSignal, onMount, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Terminal from "./components/Terminal";
import TabBar from "./components/TabBar";
import AIBar from "./components/AIBar";
import CommandPalette from "./components/CommandPalette";
import StatusBar from "./components/StatusBar";
import "./styles/global.css";
import "./styles/terminal.css";
import "./styles/effects.css";

interface Tab {
  id: string;
  title: string;
  cwd: string;
}

export default function App() {
  const [tabs, setTabs] = createSignal<Tab[]>([]);
  const [activeTab, setActiveTab] = createSignal("");
  const [showAI, setShowAI] = createSignal(false);
  const [showPalette, setShowPalette] = createSignal(false);
  const [theme, setTheme] = createSignal<any>(null);
  const [config, setConfig] = createSignal<any>(null);
  const [loaded, setLoaded] = createSignal(false);

  onMount(async () => {
    // Load config and theme
    try {
      const cfg = await invoke("get_config");
      setConfig(cfg);

      const themeData = await invoke("get_theme", {
        name: (cfg as any).theme || "hacker-green",
      });
      setTheme(themeData);
      applyTheme(themeData);
    } catch (e) {
      console.error("Failed to load config:", e);
      // Use default theme even if config fails
      setTheme({
        background: "#0a0e14",
        foreground: "#00ff41",
        cursor: "#00ff41",
        cursorAccent: "#0a0e14",
        selection: "#00ff4133",
        border: "#00ff4122",
        accent: "#00ff41",
        accentDim: "#00ff4166",
        panelBackground: "#0d1117",
        tabActive: "#00ff4120",
        statusBar: "#050808",
        ansi: {
          black: "#0a0e14",
          red: "#ff3333",
          green: "#00ff41",
          yellow: "#ffff00",
          blue: "#00d4ff",
          magenta: "#ff00ff",
          cyan: "#00ffff",
          white: "#b3b3b3",
          brightBlack: "#555555",
          brightRed: "#ff6666",
          brightGreen: "#66ff66",
          brightYellow: "#ffff66",
          brightBlue: "#66d4ff",
          brightMagenta: "#ff66ff",
          brightCyan: "#66ffff",
          brightWhite: "#ffffff",
        },
      });
    }

    // Create first tab
    createTab();
    setLoaded(true);

    // Keyboard shortcuts
    document.addEventListener("keydown", handleKeyboard);
  });

  function handleKeyboard(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key === "k") {
      e.preventDefault();
      setShowAI(!showAI());
      setShowPalette(false);
    } else if (mod && e.key === "p") {
      e.preventDefault();
      setShowPalette(!showPalette());
      setShowAI(false);
    } else if (mod && e.key === "t") {
      e.preventDefault();
      createTab();
    } else if (mod && e.key === "w") {
      e.preventDefault();
      closeTab(activeTab());
    } else if (e.key === "Escape") {
      setShowAI(false);
      setShowPalette(false);
    }
  }

  function createTab() {
    const id = crypto.randomUUID();
    const newTab: Tab = { id, title: "flux", cwd: "~" };
    setTabs((prev) => [...prev, newTab]);
    setActiveTab(id);
  }

  function closeTab(id: string) {
    invoke("close_session", { id }).catch(() => {});
    const remaining = tabs().filter((t) => t.id !== id);
    setTabs(remaining);
    if (activeTab() === id && remaining.length > 0) {
      setActiveTab(remaining[remaining.length - 1].id);
    }
    if (remaining.length === 0) {
      createTab();
    }
  }

  function applyTheme(t: any) {
    if (!t) return;
    const root = document.documentElement;
    root.style.setProperty("--bg", t.background || "#0a0e14");
    root.style.setProperty("--fg", t.foreground || "#00ff41");
    root.style.setProperty("--accent", t.accent || "#00ff41");
    root.style.setProperty("--accent-dim", t.accentDim || "#00ff4166");
    root.style.setProperty("--panel-bg", t.panelBackground || "#0d1117");
    root.style.setProperty("--tab-active", t.tabActive || "#00ff4120");
    root.style.setProperty("--status-bg", t.statusBar || "#050808");
    root.style.setProperty("--border", t.border || "#00ff4122");
    root.style.setProperty("--selection", t.selection || "#00ff4133");
    root.style.setProperty("--glow-color", t.effects?.glowColor || t.accent || "#00ff41");
  }

  return (
    <div class="app">
      <TabBar
        tabs={tabs()}
        activeTab={activeTab()}
        onSelect={setActiveTab}
        onClose={closeTab}
        onCreate={createTab}
      />

      <div class="terminal-container">
        <Show when={loaded()}>
          <For each={tabs()}>
            {(tab) => (
              <div
                class="terminal-pane"
                style={{
                  display: tab.id === activeTab() ? "block" : "none",
                }}
              >
                <Terminal
                  sessionId={tab.id}
                  theme={theme()}
                  config={config()}
                />
              </div>
            )}
          </For>
        </Show>
      </div>

      <StatusBar
        activeTab={tabs().find((t) => t.id === activeTab())}
        theme={theme()?.name || "hacker-green"}
      />

      <Show when={showAI()}>
        <AIBar
          sessionId={activeTab()}
          onClose={() => setShowAI(false)}
        />
      </Show>

      <Show when={showPalette()}>
        <CommandPalette onClose={() => setShowPalette(false)} />
      </Show>
    </div>
  );
}