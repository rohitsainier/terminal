// ─── Shared TypeScript interfaces for Flux Terminal ───

export interface EffectsConfig {
  crt_scanlines: boolean;
  crt_curvature: boolean;
  glow: boolean;
  glow_intensity: number;
  matrix_rain: boolean;
  matrix_rain_opacity: number;
  particles_on_keystroke: boolean;
  screen_flicker: boolean;
  hologram: boolean;
}

export interface KeyBindings {
  ai_bar: string;
  command_palette: string;
  new_tab: string;
  close_tab: string;
  split_vertical: string;
  split_horizontal: string;
  snippet_library: string;
}

export interface AIProvider {
  Ollama?: { model: string; base_url: string };
  OpenAI?: { api_key: string; model: string };
  Anthropic?: { api_key: string; model: string };
}

export interface AppConfig {
  theme: string;
  font_family: string;
  font_size: number;
  cursor_style: "block" | "beam" | "underline";
  cursor_blink: boolean;
  opacity: number;
  blur: boolean;
  effects: EffectsConfig;
  ai_provider: AIProvider | null;
  default_shell: string | null;
  default_cwd: string | null;
  snippets: Snippet[];
  keybindings: KeyBindings;
}

export interface Snippet {
  id: string;
  name: string;
  command: string;
  icon: string;
  tags: string[];
  category?: string;
  created_at?: string;
  description?: string;
}

export interface ThemeColors {
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

export interface Tab {
  id: string;
  title: string;
  cwd: string;
}

export interface PlanStep {
  step: number;
  description: string;
  tool: string | null;
  status: "pending" | "running" | "completed" | "error" | "skipped";
}

export interface TaskPlan {
  title: string;
  steps: PlanStep[];
}

/** Map config cursor_style to xterm's expected values ("beam" → "bar") */
export function toXtermCursorStyle(
  style: AppConfig["cursor_style"] | undefined
): "block" | "bar" | "underline" {
  if (style === "beam") return "bar";
  return style || "block";
}
