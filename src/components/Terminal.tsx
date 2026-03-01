import { onMount, onCleanup, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
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
  let sessionCreated = false;

  // ── Build xterm theme object from our theme data ──
  function buildXtermTheme(t: any) {
    if (!t) {
      return {
        background: "#0a0e14",
        foreground: "#00ff41",
        cursor: "#00ff41",
      };
    }
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

  // ── LIVE theme update — runs whenever props.theme changes ──
  createEffect(() => {
    const t = props.theme;
    if (!term || !t) return;
    term.options.theme = buildXtermTheme(t);
  });

  // ── LIVE config update — font, cursor, etc. ──
  createEffect(() => {
    const c = props.config;
    if (!term || !c) return;

    let changed = false;

    if (c.font_size && term.options.fontSize !== c.font_size) {
      term.options.fontSize = c.font_size;
      changed = true;
    }
    if (c.font_family && term.options.fontFamily !== c.font_family) {
      term.options.fontFamily = c.font_family;
      changed = true;
    }
    if (c.cursor_style && term.options.cursorStyle !== c.cursor_style) {
      term.options.cursorStyle = c.cursor_style as any;
    }
    if (c.cursor_blink !== undefined && term.options.cursorBlink !== c.cursor_blink) {
      term.options.cursorBlink = c.cursor_blink;
    }

    // Re-fit if font changed
    if (changed && fitAddon && sessionCreated) {
      setTimeout(() => {
        try {
          fitAddon?.fit();
          if (term) {
            invoke("resize_session", {
              id: props.sessionId,
              rows: term.rows,
              cols: term.cols,
            }).catch(() => {});
          }
        } catch (_) {}
      }, 50);
    }
  });

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

    const outputEvent = `pty-output-${props.sessionId}`;
    const exitEvent = `pty-exit-${props.sessionId}`;

    unlistenOutput = await listen(outputEvent, (event: any) => {
      if (!term) return;
      const payload = event.payload;
      try {
        if (Array.isArray(payload)) {
          term.write(new Uint8Array(payload));
        } else if (typeof payload === "string") {
          term.write(payload);
        } else if (payload?.data) {
          term.write(new Uint8Array(payload.data));
        } else {
          term.write(String(payload));
        }
      } catch (_) {}
    });

    unlistenExit = await listen(exitEvent, () => {
      term?.write("\r\n\x1b[31m[Session ended]\x1b[0m\r\n");
    });

    try {
      await invoke("create_session", {
        id: props.sessionId,
        rows,
        cols,
        cwd: null,
      });
      sessionCreated = true;
    } catch (e) {
      term.write(`\x1b[31mError: ${e}\x1b[0m\r\n`);
      return;
    }

    term.onData((data: string) => {
      if (!sessionCreated) return;
      invoke("write_to_session", { id: props.sessionId, data }).catch(() => {});
    });

    resizeObserver = new ResizeObserver(() => {
      if (!fitAddon || !term || !sessionCreated) return;
      try {
        fitAddon.fit();
        invoke("resize_session", {
          id: props.sessionId,
          rows: term.rows,
          cols: term.cols,
        }).catch(() => {});
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
    <div
      ref={containerRef}
      class="terminal-instance"
      style={{ width: "100%", height: "100%", padding: "4px" }}
      onClick={() => term?.focus()}
    />
  );
}