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
import HologramEffect from "./effects/HologramEffect";
import { useTheme } from "./hooks/useTheme";
import ShortcutHelp from "./components/ShortcutHelp";
import MCPPanel from "./components/MCPPanel";
import MCPChat from "./components/MCPChat";
import NetopsDashboard from "./components/netops";
import BharatLinkDashboard from "./components/bharatlink";
import type { AppConfig, Tab } from "./types";
import "./styles/global.css";
import "./styles/terminal.css";
import "./styles/effects.css";
import "./styles/netops.css";
import "./styles/bharatlink.css";

export default function App() {
  const [tabs, setTabs] = createSignal<Tab[]>([]);
  const [activeTab, setActiveTab] = createSignal("");
  const [showAI, setShowAI] = createSignal(false);
  const [showPalette, setShowPalette] = createSignal(false);
  const [showSettings, setShowSettings] = createSignal(false);
  const [showSidebar, setShowSidebar] = createSignal(false);
  const [showSnippets, setShowSnippets] = createSignal(false);
  const [config, setConfig] = createSignal<AppConfig | null>(null);
  const [loaded, setLoaded] = createSignal(false);
  const [showShortcuts, setShowShortcuts] = createSignal(false);
  const [showMCP, setShowMCP] = createSignal(false);
  const [showMCPChat, setShowMCPChat] = createSignal(false);
  const [showNetops, setShowNetops] = createSignal(false);
  const [showBharatLink, setShowBharatLink] = createSignal(false);
  const [blNodeRunning, setBlNodeRunning] = createSignal(false);
  const [blPeerCount, setBlPeerCount] = createSignal(0);

  // ── Theme via hook ──
  const theme = useTheme();

  onMount(async () => {
    // Load config
    try {
      const cfg = (await invoke("get_config")) as AppConfig;
      setConfig(cfg);
      await theme.loadTheme(cfg.theme || "hacker-green");
    } catch (_) {
      await theme.loadTheme("hacker-green");
    }

    createTab();
    setLoaded(true);
    document.addEventListener("keydown", handleKeyboard);

    // Auto-start BharatLink node in background
    try {
      await invoke("bharatlink_start");
      setBlNodeRunning(true);
      // Start polling peer count every 10s
      const pollPeers = async () => {
        try {
          const peers = (await invoke("bharatlink_get_peers")) as any[];
          setBlPeerCount(peers.length);
        } catch (_) {}
      };
      pollPeers();
      const peerInterval = setInterval(pollPeers, 10000);
      // Listen for node status events
      const { listen } = await import("@tauri-apps/api/event");
      await listen("bharatlink-node-status", (e: any) => {
        const running = e.payload?.is_running ?? false;
        setBlNodeRunning(running);
        if (running && e.payload?.discovered_peers != null) {
          setBlPeerCount(e.payload.discovered_peers);
        }
        if (!running) setBlPeerCount(0);
      });
      await listen("bharatlink-peer-discovered", () => {
        pollPeers();
      });
      await listen("bharatlink-peer-lost", () => {
        pollPeers();
      });
    } catch (e) {
      console.warn("[BharatLink] Auto-start failed:", e);
    }
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyboard);
    // Save history on app close
    invoke("save_history").catch(() => {});
  });

  function handleKeyboard(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key === "k") {
      e.preventDefault();
      closeAllOverlays();
      setShowAI((v) => !v);
    } else if (mod && e.key === "p") {
      e.preventDefault();
      closeAllOverlays();
      setShowPalette((v) => !v);
    } else if (mod && e.key === ",") {
      e.preventDefault();
      closeAllOverlays();
      setShowSettings((v) => !v);
    } else if (mod && e.key === "t") {
      e.preventDefault();
      createTab();
    } else if (mod && e.key === "w") {
      e.preventDefault();
      closeTab(activeTab());
    } else if (mod && !e.shiftKey && e.key === "b") {
      e.preventDefault();
      setShowSidebar((s) => !s);
    } else if (mod && e.shiftKey && (e.key === "L" || e.key === "l")) {
      e.preventDefault();
      closeAllOverlays();
      setShowSnippets((v) => !v);
    } else if (e.key === "Escape") {
      closeAllOverlays();
      setShowShortcuts(false);
    } else if (e.key === "?" && !e.metaKey && !e.ctrlKey && !showAI() && !showPalette() && !showSettings() && !showSnippets()) {
      // Only trigger if no overlay is open and not typing in an input
      const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
      if (tag !== "input" && tag !== "textarea" && tag !== "select") {
        e.preventDefault();
        setShowShortcuts((v) => !v);
      }
    } else if (mod && e.key === "m") {
      e.preventDefault();
      closeAllOverlays();
      setShowMCP((v) => !v);
    } else if (mod && e.shiftKey && (e.key === "C" || e.key === "c") && !e.altKey) {
      e.preventDefault();
      closeAllOverlays();
      setShowMCPChat((v) => !v);
    } else if (mod && e.shiftKey && (e.key === "N" || e.key === "n")) {
      e.preventDefault();
      closeAllOverlays();
      setShowNetops((v) => !v);
    } else if (mod && e.shiftKey && (e.key === "B" || e.key === "b")) {
      e.preventDefault();
      closeAllOverlays();
      setShowBharatLink((v) => !v);
    }
  }

  function closeAllOverlays() {
    setShowAI(false);
    setShowPalette(false);
    setShowSettings(false);
    setShowSnippets(false);
    setShowShortcuts(false);
    setShowMCP(false);
    setShowMCPChat(false);
    setShowNetops(false);
    setShowBharatLink(false);
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
    if (remaining.length === 0) createTab();
  }

  async function handleThemeChange(themeName: string) {
    await theme.loadTheme(themeName);
    try {
      const cfg = (await invoke("get_config")) as AppConfig;
      setConfig(cfg);
    } catch (_) {}
  }

  // ── Write to active terminal ──
  function writeToActiveSession(data: string) {
    invoke("write_to_session", { id: activeTab(), data }).catch(() => {});
  }

  // ── SSH connect ──
  function handleSSHConnect(sshId: string) {
    invoke("connect_ssh", { id: sshId, sessionId: activeTab() }).catch(
      (e: any) => console.error("SSH connect error:", e)
    );
    setShowSidebar(false);
  }

  // ── Toggle effects ──
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
      case "hologram":
        fx.hologram = !fx.hologram;
        break;
    }

    const newConfig = { ...cfg, effects: fx };
    setConfig(newConfig);
    try {
      await invoke("set_config", { config: newConfig });
    } catch (e) {
      console.error("Save config error:", e);
    }
  }

  // ── Command Palette action router ──
  function handlePaletteAction(actionId: string) {
    if (actionId === "new-tab") createTab();
    else if (actionId === "ai-bar") setShowAI(true);
    else if (actionId === "snippets") setShowSnippets(true);
    else if (actionId === "settings") setShowSettings(true);
    else if (actionId === "toggle-sidebar") setShowSidebar((s) => !s);
    else if (actionId === "toggle-crt") toggleEffect("crt");
    else if (actionId === "toggle-glow") toggleEffect("glow");
    else if (actionId === "toggle-matrix") toggleEffect("matrix");
    else if (actionId === "toggle-particles") toggleEffect("particles");
    else if (actionId === "toggle-hologram") toggleEffect("hologram");
    else if (actionId === "mcp-panel") setShowMCP(true);
    else if (actionId === "mcp-chat") setShowMCPChat(true);
    else if (actionId === "netops") setShowNetops(true);
    else if (actionId === "bharatlink") setShowBharatLink(true);
    else if (actionId.startsWith("theme-")) {
      handleThemeChange(actionId.replace("theme-", ""));
    }
  }

  const fx = () => config()?.effects || ({} as Partial<import("./types").EffectsConfig>);
  const themeAccent = () =>
    theme.currentTheme()?.accent ||
    theme.currentTheme()?.effects?.glowColor ||
    "#00ff41";

  return (
    <div class="app">
      {/* ── Visual Effects ── */}
      <Show when={fx().matrix_rain}>
        <MatrixRain
          color={theme.currentTheme()?.effects?.particleColor || themeAccent()}
          opacity={fx().matrix_rain_opacity || 0.05}
        />
      </Show>

      <CRTEffect
        enabled={fx().crt_scanlines ?? false}
        curvature={fx().crt_curvature ?? false}
        flicker={fx().screen_flicker ?? false}
      />

      <GlowEffect
        color={theme.currentTheme()?.effects?.glowColor || themeAccent()}
        intensity={fx().glow_intensity || 0.3}
        enabled={fx().glow ?? false}
      />

      <ParticleEngine
        color={theme.currentTheme()?.effects?.particleColor || themeAccent()}
        enabled={fx().particles_on_keystroke ?? false}
      />

      <HologramEffect
        color={themeAccent()}
        enabled={fx().hologram ?? false}
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
        <Sidebar
          visible={showSidebar()}
          onClose={() => setShowSidebar(false)}
          activeSessionId={activeTab()}
          tabs={tabs()}
          onSnippetRun={(cmd) => writeToActiveSession(cmd + "\n")}
          onSSHConnect={handleSSHConnect}
          onTabSelect={setActiveTab}
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
                    theme={theme.currentTheme()}
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
        theme={theme.currentTheme()?.name || theme.themeName()}
        onShowShortcuts={() => setShowShortcuts((v) => !v)}
        bharatLinkRunning={blNodeRunning()}
        bharatLinkPeerCount={blPeerCount()}
        onBharatLinkClick={() => {
          closeAllOverlays();
          setShowBharatLink(true);
        }}
      />

      {/* ── Overlays ── */}
      <Show when={showAI()}>
        <AIBar
          sessionId={activeTab()}
          onClose={() => setShowAI(false)}
        />
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
            invoke("get_config")
              .then((cfg) => setConfig(cfg as AppConfig))
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

      <Show when={showShortcuts()}>
        <ShortcutHelp
          visible={true}
          onClose={() => setShowShortcuts(false)}
        />
      </Show>

      <Show when={showMCP()}>
        <MCPPanel onClose={() => setShowMCP(false)} />
      </Show>

      <Show when={showMCPChat()}>
        <MCPChat
          onClose={() => setShowMCPChat(false)}
          onRunCommand={writeToActiveSession}
        />
      </Show>

      <Show when={showNetops()}>
        <NetopsDashboard onClose={() => setShowNetops(false)} />
      </Show>

      <Show when={showBharatLink()}>
        <BharatLinkDashboard onClose={() => setShowBharatLink(false)} />
      </Show>
    </div>
  );
}