import { createSignal, onMount, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Terminal from "./components/Terminal";
import TabBar from "./components/TabBar";
import AIBar from "./components/AIBar";
import CommandPalette from "./components/CommandPalette";
import StatusBar from "./components/StatusBar";
import Settings from "./components/Settings";
import { getTheme, applyThemeToDOM } from "./themes/ThemeEngine";
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
  const [showSettings, setShowSettings] = createSignal(false);
  const [theme, setTheme] = createSignal<any>(null);
  const [config, setConfig] = createSignal<any>(null);
  const [loaded, setLoaded] = createSignal(false);

  onMount(async () => {
    // Load config from Rust backend
    try {
      const cfg = await invoke("get_config");
      setConfig(cfg);

      // Try loading theme from backend first
      try {
        const themeData = await invoke("get_theme", {
          name: (cfg as any).theme || "hacker-green",
        });
        setTheme(themeData);
        applyThemeToDOM(themeData);
      } catch (e) {
        // Fallback to frontend theme files
        const localTheme = getTheme((cfg as any).theme || "hacker-green");
        if (localTheme) {
          setTheme(localTheme);
          applyThemeToDOM(localTheme);
        }
      }
    } catch (e) {
      console.error("Failed to load config:", e);
      // Use default theme
      const defaultTheme = getTheme("hacker-green");
      if (defaultTheme) {
        setTheme(defaultTheme);
        applyThemeToDOM(defaultTheme);
      }
    }

    createTab();
    setLoaded(true);

    document.addEventListener("keydown", handleKeyboard);
  });

  function handleKeyboard(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key === "k") {
      e.preventDefault();
      setShowAI(!showAI());
      setShowPalette(false);
      setShowSettings(false);
    } else if (mod && e.key === "p") {
      e.preventDefault();
      setShowPalette(!showPalette());
      setShowAI(false);
      setShowSettings(false);
    } else if (mod && e.key === ",") {
      e.preventDefault();
      setShowSettings(!showSettings());
      setShowAI(false);
      setShowPalette(false);
    } else if (mod && e.key === "t") {
      e.preventDefault();
      createTab();
    } else if (mod && e.key === "w") {
      e.preventDefault();
      closeTab(activeTab());
    } else if (e.key === "Escape") {
      setShowAI(false);
      setShowPalette(false);
      setShowSettings(false);
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

  async function handleThemeChange(themeName: string) {
    try {
      const themeData = await invoke("get_theme", { name: themeName });
      setTheme(themeData);
      applyThemeToDOM(themeData);
    } catch (e) {
      const localTheme = getTheme(themeName);
      if (localTheme) {
        setTheme(localTheme);
        applyThemeToDOM(localTheme);
      }
    }

    // Reload config
    try {
      const cfg = await invoke("get_config");
      setConfig(cfg);
    } catch (e) {}
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

      <Show when={showSettings()}>
        <Settings
          onClose={() => setShowSettings(false)}
          onThemeChange={handleThemeChange}
          currentConfig={config()}
        />
      </Show>
    </div>
  );
}