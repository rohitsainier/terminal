import { onMount, onCleanup } from "solid-js";
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

  onMount(async () => {
    if (!containerRef) return;

    const t = props.theme;
    const c = props.config;

    console.log("[Terminal] Creating xterm for session:", props.sessionId);

    term = new XTerm({
      fontFamily: c?.font_family || "'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Menlo', monospace",
      fontSize: c?.font_size || 14,
      lineHeight: 1.2,
      cursorStyle: (c?.cursor_style as any) || "block",
      cursorBlink: c?.cursor_blink ?? true,
      allowTransparency: true,
      scrollback: 10000,
      convertEol: true,
      theme: t
        ? {
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
          }
        : {
            background: "#0a0e14",
            foreground: "#00ff41",
            cursor: "#00ff41",
          },
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    term.open(containerRef);

    // Try WebGL
    try {
      const { WebglAddon } = await import("@xterm/addon-webgl");
      term.loadAddon(new WebglAddon());
      console.log("[Terminal] WebGL renderer loaded");
    } catch (e) {
      console.warn("[Terminal] WebGL not available, using canvas");
    }

    // Fit after a small delay to ensure container has size
    await new Promise((r) => setTimeout(r, 200));
    fitAddon.fit();

    const rows = term.rows;
    const cols = term.cols;
    console.log("[Terminal] Size:", cols, "x", rows);

    // Set up event listeners BEFORE creating session
    const outputEvent = `pty-output-${props.sessionId}`;
    const exitEvent = `pty-exit-${props.sessionId}`;

    console.log("[Terminal] Listening for:", outputEvent);

    unlistenOutput = await listen(outputEvent, (event: any) => {
      if (!term) return;

      const payload = event.payload;
      console.log("[Terminal] Got output, type:", typeof payload, "length:", payload?.length);

      try {
        if (payload instanceof Array || Array.isArray(payload)) {
          // Rust Vec<u8> comes as number array
          const bytes = new Uint8Array(payload);
          term.write(bytes);
        } else if (typeof payload === "string") {
          // String data
          term.write(payload);
        } else if (payload && payload.data) {
          // Wrapped payload
          const bytes = new Uint8Array(payload.data);
          term.write(bytes);
        } else {
          console.warn("[Terminal] Unknown payload format:", payload);
          // Try writing as string anyway
          term.write(String(payload));
        }
      } catch (e) {
        console.error("[Terminal] Write error:", e);
      }
    });

    unlistenExit = await listen(exitEvent, () => {
      if (!term) return;
      console.log("[Terminal] Session exited");
      term.write("\r\n\x1b[31m[Session ended]\x1b[0m\r\n");
    });

    // Create PTY session
    try {
      console.log("[Terminal] Creating PTY session...");
      await invoke("create_session", {
        id: props.sessionId,
        rows: rows,
        cols: cols,
        cwd: null,
      });
      sessionCreated = true;
      console.log("[Terminal] PTY session created successfully");
    } catch (e) {
      console.error("[Terminal] Failed to create PTY:", e);
      term.write(`\x1b[31mError: ${e}\x1b[0m\r\n`);
      return;
    }

    // Input handler
    term.onData((data: string) => {
      if (!sessionCreated) return;
      console.log("[Terminal] Input:", JSON.stringify(data));

      invoke("write_to_session", {
        id: props.sessionId,
        data: data,
      }).catch((e: any) => {
        console.error("[Terminal] Write failed:", e);
      });
    });

    // Resize handler
    resizeObserver = new ResizeObserver(() => {
      if (!fitAddon || !term || !sessionCreated) return;
      try {
        fitAddon.fit();
        invoke("resize_session", {
          id: props.sessionId,
          rows: term.rows,
          cols: term.cols,
        }).catch(() => {});
      } catch (e) {}
    });
    resizeObserver.observe(containerRef);

    term.focus();
  });

  onCleanup(() => {
    console.log("[Terminal] Cleanup:", props.sessionId);
    if (unlistenOutput) unlistenOutput();
    if (unlistenExit) unlistenExit();
    if (resizeObserver) resizeObserver.disconnect();
    if (term) term.dispose();
  });

  return (
    <div
      ref={containerRef}
      class="terminal-instance"
      style={{
        width: "100%",
        height: "100%",
        padding: "4px",
      }}
      onClick={() => term?.focus()}
    />
  );
}