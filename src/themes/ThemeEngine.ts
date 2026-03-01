export interface Theme {
  name: string;
  background: string;
  foreground: string;
  cursor: string;
  cursorAccent: string;
  selection: string;
  border: string;
  accent: string;
  accentDim: string;
  panelBackground: string;
  tabActive: string;
  statusBar: string;
  ansi: {
    black: string;
    red: string;
    green: string;
    yellow: string;
    blue: string;
    magenta: string;
    cyan: string;
    white: string;
    brightBlack: string;
    brightRed: string;
    brightGreen: string;
    brightYellow: string;
    brightBlue: string;
    brightMagenta: string;
    brightCyan: string;
    brightWhite: string;
  };
  effects: {
    glowColor: string;
    scanlineColor: string;
    particleColor: string;
  };
}

// Import all themes
import hackerGreen from "./hacker-green.json";
import cyberpunk from "./cyberpunk.json";
import matrix from "./matrix.json";
import ghostProtocol from "./ghost-protocol.json";
import tron from "./tron.json";
import midnight from "./midnight.json";

const themes: Record<string, Theme> = {
  "hacker-green": hackerGreen as Theme,
  "cyberpunk": cyberpunk as Theme,
  "matrix": matrix as Theme,
  "ghost-protocol": ghostProtocol as Theme,
  "tron": tron as Theme,
  "midnight": midnight as Theme,
};

export function getTheme(name: string): Theme | null {
  return themes[name] || null;
}

export function getAllThemeNames(): string[] {
  return Object.keys(themes);
}

export function applyThemeToDOM(theme: Theme) {
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
  root.style.setProperty("--cursor-color", theme.cursor);
}