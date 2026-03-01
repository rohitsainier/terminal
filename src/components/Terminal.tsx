import { onMount, onCleanup, createEffect, createSignal } from "solid-js";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { useTerminal } from "../hooks/useTerminal";
import Autocomplete from "./Autocomplete";
import type { Suggestion } from "./Autocomplete";
import "@xterm/xterm/css/xterm.css";

interface Props {
  sessionId: string;
  theme: any;
  config: any;
}

export default function Terminal(props: Props) {
  let containerRef: HTMLDivElement | undefined;
  let term: XTerm | undefined;
  let fitAddon: FitAddon | undefined;
  let unlistenOutput: UnlistenFn | undefined;
  let unlistenExit: UnlistenFn | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let lineBuffer = "";

  const pty = useTerminal(props.sessionId);

  // Autocomplete state
  const [showAC, setShowAC] = createSignal(false);
  const [acPos, setAcPos] = createSignal({ x: 0, y: 0 });
  const [acSuggestions, setAcSuggestions] = createSignal<Suggestion[]>([]);
  const [acIndex, setAcIndex] = createSignal(0);

  function buildXtermTheme(t: any) {
    if (!t) return { background: "#0a0e14", foreground: "#00ff41", cursor: "#00ff41" };
    return {
      background: t.background || "#0a0e14",
      foreground: t.foreground || "#00ff41",
      cursor: t.cursor || "#00ff41",
      cursorAccent: t.cursorAccent || "#0a0e14",
      selectionBackground: t.selection || "#00ff4133",
      black: t.ansi?.black || "#000000",
      red: t.ansi?.red || "#ff0000",
      green: t.ansi?.green || "#00ff00",
      yellow: t.ansi?.yellow || "#ffff00",
      blue: t.ansi?.blue || "#0000ff",
      magenta: t.ansi?.magenta || "#ff00ff",
      cyan: t.ansi?.cyan || "#00ffff",
      white: t.ansi?.white || "#ffffff",
      brightBlack: t.ansi?.brightBlack || "#555555",
      brightRed: t.ansi?.brightRed || "#ff5555",
      brightGreen: t.ansi?.brightGreen || "#55ff55",
      brightYellow: t.ansi?.brightYellow || "#ffff55",
      brightBlue: t.ansi?.brightBlue || "#5555ff",
      brightMagenta: t.ansi?.brightMagenta || "#ff55ff",
      brightCyan: t.ansi?.brightCyan || "#55ffff",
      brightWhite: t.ansi?.brightWhite || "#ffffff",
    };
  }

  // ── Live theme updates ──
  createEffect(() => {
    const t = props.theme;
    if (term && t) term.options.theme = buildXtermTheme(t);
  });

  // ── Live config updates ──
  createEffect(() => {
    const c = props.config;
    if (!term || !c) return;
    let refit = false;
    if (c.font_size && term.options.fontSize !== c.font_size) {
      term.options.fontSize = c.font_size;
      refit = true;
    }
    if (c.font_family && term.options.fontFamily !== c.font_family) {
      term.options.fontFamily = c.font_family;
      refit = true;
    }
    if (c.cursor_style) term.options.cursorStyle = c.cursor_style as any;
    if (c.cursor_blink !== undefined) term.options.cursorBlink = c.cursor_blink;

    if (refit && fitAddon && pty.isConnected()) {
      setTimeout(() => {
        try {
          fitAddon?.fit();
          if (term) pty.resize(term.rows, term.cols).catch(() => {});
        } catch (_) {}
      }, 50);
    }
  });

  // ── Cursor pixel position ──
  function getCursorPixelPos() {
    if (!term || !containerRef) return { x: 200, y: 200 };
    const buf = term.buffer.active;
    const rect = containerRef.getBoundingClientRect();
    const cellW = rect.width / term.cols;
    const cellH = rect.height / term.rows;
    return {
      x: rect.left + buf.cursorX * cellW,
      y: rect.top + (buf.cursorY + 1) * cellH + 4,
    };
  }

  // ── Read current line from buffer ──
  function getCurrentLine(): string {
    if (!term) return "";
    const buf = term.buffer.active;
    const line = buf.getLine(buf.cursorY + buf.baseY);
    if (!line) return "";
    return line.translateToString(true, 0, buf.cursorX).trim();
  }

  function getLastToken(): string {
    const line = getCurrentLine();
    const parts = line.split(/\s+/);
    return parts[parts.length - 1] || "";
  }

  // ── Trigger autocomplete ──
  async function triggerAutocomplete() {
    const partial = getLastToken();
    setAcPos(getCursorPixelPos());
    setAcIndex(0);

    const suggestions = await pty.getAutocompleteSuggestions(
      partial || getCurrentLine()
    );
    setAcSuggestions(suggestions);
    setShowAC(suggestions.length > 0);
  }

  // ── Select autocomplete item ──
  function selectSuggestion(s: Suggestion) {
    if (!pty.isConnected()) return;
    const lastToken = getLastToken();
    let text = s.text;

    // For history/snippet type, replace entire line would be weird,
    // so for path type replace just the last token
    if (s.type === "path" && lastToken) {
      // Delete the partial token first
      const deletes = "\x7f".repeat(lastToken.length);
      pty.write(deletes + text);
    } else if (s.type === "history" || s.type === "snippet") {
      // For full commands from history, clear line and write full command
      // Send Ctrl-U to clear line, then type the command
      pty.write("\x15" + text);
    } else {
      pty.write(text);
    }
    setShowAC(false);
  }

  onMount(async () => {
    if (!containerRef) return;

    const t = props.theme;
    const c = props.config;

    term = new XTerm({
      fontFamily:
        c?.font_family ||
        "'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Menlo', monospace",
      fontSize: c?.font_size || 14,
      lineHeight: 1.2,
      cursorStyle: (c?.cursor_style as any) || "block",
      cursorBlink: c?.cursor_blink ?? true,
      allowTransparency: true,
      scrollback: 10000,
      convertEol: true,
      theme: buildXtermTheme(t),
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());
    term.open(containerRef);

    try {
      const { WebglAddon } = await import("@xterm/addon-webgl");
      term.loadAddon(new WebglAddon());
    } catch (_) {}

    await new Promise((r) => setTimeout(r, 200));
    fitAddon.fit();

    const rows = term.rows;
    const cols = term.cols;

    // ── PTY output listener ──
    unlistenOutput = await listen(`pty-output-${props.sessionId}`, (event: any) => {
      if (!term) return;
      const payload = event.payload;
      try {
        if (Array.isArray(payload)) term.write(new Uint8Array(payload));
        else if (typeof payload === "string") term.write(payload);
        else if (payload?.data) term.write(new Uint8Array(payload.data));
        else term.write(String(payload));
      } catch (_) {}
    });

    unlistenExit = await listen(`pty-exit-${props.sessionId}`, () => {
      term?.write("\r\n\x1b[31m[Session ended]\x1b[0m\r\n");
    });

    // ── Create PTY session via hook ──
    try {
      await pty.createSession(rows, cols);
    } catch (e) {
      term.write(`\x1b[31mError: ${e}\x1b[0m\r\n`);
      return;
    }

    // ── Custom key handler for autocomplete ──
    term.attachCustomKeyEventHandler((event) => {
      // Ctrl+Space → trigger autocomplete
      if (event.ctrlKey && event.code === "Space" && event.type === "keydown") {
        triggerAutocomplete();
        return false;
      }

      // When autocomplete is open, intercept navigation keys
      if (showAC()) {
        if (event.type !== "keydown") return false;

        if (event.key === "ArrowDown") {
          setAcIndex((i) => Math.min(i + 1, acSuggestions().length - 1));
          return false;
        }
        if (event.key === "ArrowUp") {
          setAcIndex((i) => Math.max(i - 1, 0));
          return false;
        }
        if (event.key === "Enter" || event.key === "Tab") {
          const s = acSuggestions()[acIndex()];
          if (s) selectSuggestion(s);
          return false;
        }
        if (event.key === "Escape") {
          setShowAC(false);
          return false;
        }
        // Any other key closes autocomplete and proceeds normally
        setShowAC(false);
      }

      return true;
    });

    // ── Input handler with history tracking ──
    term.onData((data: string) => {
      if (!pty.isConnected()) return;

      // Track line buffer for history
      if (data === "\r") {
        // Enter pressed — log command to history
        const line = getCurrentLine();
        if (line.trim()) {
          pty.addToHistory(line);
        }
        lineBuffer = "";
      } else if (data === "\x7f") {
        // Backspace
        lineBuffer = lineBuffer.slice(0, -1);
      } else if (data.length === 1 && data.charCodeAt(0) >= 32) {
        lineBuffer += data;
      }

      pty.write(data);
    });

    // ── Resize observer ──
    resizeObserver = new ResizeObserver(() => {
      if (!fitAddon || !term || !pty.isConnected()) return;
      try {
        fitAddon.fit();
        pty.resize(term.rows, term.cols).catch(() => {});
      } catch (_) {}
    });
    resizeObserver.observe(containerRef);

    term.focus();
  });

  onCleanup(() => {
    unlistenOutput?.();
    unlistenExit?.();
    resizeObserver?.disconnect();
    term?.dispose();
  });

  return (
    <div style={{ position: "relative", width: "100%", height: "100%" }}>
      <div
        ref={containerRef}
        class="terminal-instance"
        style={{ width: "100%", height: "100%", padding: "4px" }}
        onClick={() => term?.focus()}
      />
      <Autocomplete
        suggestions={acSuggestions()}
        visible={showAC()}
        x={acPos().x}
        y={acPos().y}
        selectedIndex={acIndex()}
        onSelect={selectSuggestion}
        onHover={setAcIndex}
      />
    </div>
  );
}