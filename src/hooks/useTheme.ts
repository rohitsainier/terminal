import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  getTheme as getLocalTheme,
  applyThemeToDOM,
  type Theme,
} from "../themes/ThemeEngine";

export function useTheme() {
  const [currentTheme, setCurrentTheme] = createSignal<Theme | null>(null);
  const [themeName, setThemeName] = createSignal("hacker-green");

  async function loadTheme(name: string): Promise<Theme | null> {
    // Try backend first
    try {
      const theme = (await invoke("get_theme", { name })) as Theme;
      setCurrentTheme(theme);
      setThemeName(name);
      applyThemeToDOM(theme);
      return theme;
    } catch (_) {
      // Fallback to frontend theme files
      const localTheme = getLocalTheme(name);
      if (localTheme) {
        setCurrentTheme(localTheme);
        setThemeName(name);
        applyThemeToDOM(localTheme);
        return localTheme;
      }
      console.error("Theme not found:", name);
      return null;
    }
  }

  async function listThemes(): Promise<string[]> {
    try {
      return (await invoke("list_themes")) as string[];
    } catch (_) {
      return [
        "hacker-green",
        "cyberpunk",
        "matrix",
        "ghost-protocol",
        "tron",
        "midnight",
      ];
    }
  }

  return { currentTheme, themeName, loadTheme, listThemes };
}