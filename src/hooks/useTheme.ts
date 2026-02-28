import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export function useTheme() {
  const [currentTheme, setCurrentTheme] = createSignal<any>(null);
  const [themeName, setThemeName] = createSignal("hacker-green");

  async function loadTheme(name: string) {
    try {
      const theme = await invoke("get_theme", { name });
      setCurrentTheme(theme);
      setThemeName(name);
      applyThemeToDOM(theme);
      return theme;
    } catch (e) {
      console.error("Failed to load theme:", e);
      return null;
    }
  }

  async function listThemes(): Promise<string[]> {
    return (await invoke("list_themes")) as string[];
  }

  function applyThemeToDOM(theme: any) {
    const root = document.documentElement;
    root.style.setProperty("--bg", theme.background);
    root.style.setProperty("--fg", theme.foreground);
    root.style.setProperty("--accent", theme.accent);
    root.style.setProperty("--accent-dim", theme.accentDim);
    root.style.setProperty("--panel-bg", theme.panelBackground);
    root.style.setProperty("--tab-active", theme.tabActive);
    root.style.setProperty("--status-bg", theme.statusBar);
    root.style.setProperty("--border", theme.border);
    root.style.setProperty("--selection", theme.selection);
    root.style.setProperty("--glow-color", theme.effects?.glowColor || theme.accent);
  }

  return {
    currentTheme,
    themeName,
    loadTheme,
    listThemes,
  };
}