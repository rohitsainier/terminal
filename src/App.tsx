import { createSignal, onMount, onCleanup, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Terminal from "./components/Terminal";
import TabBar from "./components/TabBar";
import AIBar from "./components/AIBar";
import CommandPalette from "./components/CommandPalette";
import StatusBar from "./components/StatusBar";
import Settings from "./components/Settings";
import Sidebar from "./components/Sidebar";
import SnippetLibrary from "./components/SnippetLibrary";
import CRTEffect from "./effects/CRTEffect";
import GlowEffect from "./effects/GlowEffect";
import MatrixRain from "./effects/MatrixRain";
import ParticleEngine from "./effects/ParticleEngine";
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
  const [showSidebar, setShowSidebar] = createSignal(false);
  const [showSnippets, setShowSnippets] = createSignal(false);
  const [theme, setTheme] = createSignal<any>(null);
  const [config, setConfig] = createSignal<any>(null);
  const [loaded, setLoaded] = createSignal(false);

  onMount(async () => {
    try {
      const cfg = await invoke("get_config");
      setConfig(cfg);

      try {
        const themeData = await invoke("get_theme", {
          name: (cfg as any).theme || "hacker-green",
        });
        setTheme(themeData);
        applyThemeToDOM(themeData);
      } catch (_) {
        const localTheme = getTheme((cfg as any).theme || "hacker-green");
        if (localTheme) {
          setTheme(localTheme);
          applyThemeToDOM(localTheme);
        }
      }
    } catch (_) {
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

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyboard);
  });

  function handleKeyboard(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key === "k") {
      e.preventDefault();
      closeAllOverlays();
      setShowAI(!showAI());
    } else if (mod && e.key === "p") {
      e.preventDefault();
      closeAllOverlays();
      setShowPalette(!showPalette());
    } else if (mod && e.key === ",") {
      e.preventDefault();
      closeAllOverlays();
      setShowSettings(!showSettings());
    } else if (mod && e.key === "t") {
      e.preventDefault();
      createTab();
    } else if (mod && e.key === "w") {
      e.preventDefault();
      closeTab(activeTab());
    } else if (mod && e.key === "b") {
      e.preventDefault();
      setShowSidebar((s) => !s);
    } else if (mod && e.shiftKey && e.key === "L") {
      e.preventDefault();
      closeAllOverlays();
      setShowSnippets(!showSnippets());
    } else if (e.key === "Escape") {
      closeAllOverlays();
    }
  }

  function closeAllOverlays() {
    setShowAI(false);
    setShowPalette(false);
    setShowSettings(false);
    setShowSnippets(false);
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
    } catch (_) {
      const localTheme = getTheme(themeName);
      if (localTheme) {
        setTheme(localTheme);
        applyThemeToDOM(localTheme);
      }
    }
    try {
      const cfg = await invoke("get_config");
      setConfig(cfg);
    } catch (_) {}
  }

  // ── Toggle an effect, save config, re-render ──
  async function toggleEffect(effect: string) {
    const cfg = config();
    if (!cfg) return;

    const fx = { ...(cfg.effects || {}) };
    switch (effect) {
      case "crt":
        fx.crt_scanlines = !fx.crt_scanlines;
        break;
      case "glow":
        fx.glow = !fx.glow;
        break;
      case "matrix":
        fx.matrix_rain = !fx.matrix_rain;
        break;
      case "particles":
        fx.particles_on_keystroke = !fx.particles_on_keystroke;
        break;
    }

    const newConfig = { ...cfg, effects: fx };
    setConfig(newConfig);
    try {
      await invoke("set_config", { config: newConfig });
    } catch (e) {
      console.error("Failed to save config:", e);
    }
  }

  // ── Command Palette action router ──
  function handlePaletteAction(actionId: string) {
    if (actionId === "new-tab") {
      createTab();
    } else if (actionId === "ai-bar") {
      setShowAI(true);
    } else if (actionId === "snippets") {
      setShowSnippets(true);
    } else if (actionId === "settings") {
      setShowSettings(true);
    } else if (actionId === "toggle-sidebar") {
      setShowSidebar((s) => !s);
    } else if (actionId === "toggle-crt") {
      toggleEffect("crt");
    } else if (actionId === "toggle-glow") {
      toggleEffect("glow");
    } else if (actionId === "toggle-matrix") {
      toggleEffect("matrix");
    } else if (actionId === "toggle-particles") {
      toggleEffect("particles");
    } else if (actionId.startsWith("theme-")) {
      const name = actionId.replace("theme-", "");
      handleThemeChange(name);
    }
  }

  // Convenience accessors for effects config
  const fx = () => config()?.effects || {};
  const themeAccent = () =>
    theme()?.accent || theme()?.effects?.glowColor || "#00ff41";

  return (
    <div class="app">
      {/* ── Visual Effects ── */}
      <Show when={fx().matrix_rain}>
        <MatrixRain
          color={theme()?.effects?.particleColor || themeAccent()}
          opacity={fx().matrix_rain_opacity || 0.05}
        />
      </Show>

      <CRTEffect
        enabled={fx().crt_scanlines ?? false}
        curvature={fx().crt_curvature ?? false}
        flicker={fx().screen_flicker ?? false}
      />

      <GlowEffect
        color={theme()?.effects?.glowColor || themeAccent()}
        intensity={fx().glow_intensity || 0.3}
        enabled={fx().glow ?? false}
      />

      <ParticleEngine
        color={theme()?.effects?.particleColor || themeAccent()}
        enabled={fx().particles_on_keystroke ?? false}
      />

      {/* ── Chrome ── */}
      <TabBar
        tabs={tabs()}
        activeTab={activeTab()}
        onSelect={setActiveTab}
        onClose={closeTab}
        onCreate={createTab}
      />

      <div style={{ display: "flex", flex: "1", overflow: "hidden" }}>
        {/* ── Sidebar ── */}
        <Show when={showSidebar()}>
          <Sidebar
            visible={true}
            onClose={() => setShowSidebar(false)}
          />
        </Show>

        {/* ── Terminal Panes ── */}
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
      </div>

      <StatusBar
        activeTab={tabs().find((t) => t.id === activeTab())}
        theme={theme()?.name || config()?.theme || "hacker-green"}
      />

      {/* ── Overlays ── */}
      <Show when={showAI()}>
        <AIBar sessionId={activeTab()} onClose={() => setShowAI(false)} />
      </Show>

      <Show when={showPalette()}>
        <CommandPalette
          onClose={() => setShowPalette(false)}
          onAction={handlePaletteAction}
        />
      </Show>

      <Show when={showSettings()}>
        <Settings
          onClose={() => {
            setShowSettings(false);
            // Reload config so effects/font changes take effect
            invoke("get_config")
              .then((cfg) => setConfig(cfg))
              .catch(() => {});
          }}
          onThemeChange={handleThemeChange}
          currentConfig={config()}
        />
      </Show>

      <Show when={showSnippets()}>
        <SnippetLibrary
          sessionId={activeTab()}
          onClose={() => setShowSnippets(false)}
        />
      </Show>
    </div>
  );
}