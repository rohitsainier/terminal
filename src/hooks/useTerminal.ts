import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export interface Suggestion {
  text: string;
  type: "history" | "path" | "command" | "snippet";
  icon: string;
}

export function useTerminal(sessionId: string) {
  const [isConnected, setIsConnected] = createSignal(false);
  const [cwd, setCwd] = createSignal("~");

  async function createSession(rows: number, cols: number, initialCwd?: string) {
    try {
      await invoke("create_session", { id: sessionId, rows, cols, cwd: initialCwd });
      setIsConnected(true);
    } catch (e) {
      console.error("Failed to create session:", e);
      throw e;
    }
  }

  async function write(data: string) {
    await invoke("write_to_session", { id: sessionId, data });
  }

  async function resize(rows: number, cols: number) {
    await invoke("resize_session", { id: sessionId, rows, cols });
  }

  async function close() {
    await invoke("close_session", { id: sessionId });
    setIsConnected(false);
  }

  async function updateInfo(title?: string, newCwd?: string) {
    try {
      await invoke("update_session_info", { id: sessionId, title: title || null, cwd: newCwd || null });
      if (newCwd) setCwd(newCwd);
    } catch (_) {}
  }

  async function addToHistory(command: string) {
    try {
      await invoke("add_history_entry", {
        entry: {
          command,
          timestamp: String(Math.floor(Date.now() / 1000)),
          cwd: cwd(),
          exit_code: null,
          duration_ms: null,
          session_id: sessionId,
        },
      });
    } catch (_) {}
  }

  async function getAutocompleteSuggestions(partial: string): Promise<Suggestion[]> {
    const suggestions: Suggestion[] = [];

    // History suggestions
    try {
      const history = (await invoke("unique_commands", { limit: 100 })) as string[];
      const filtered = history.filter((c) =>
        c.toLowerCase().startsWith(partial.toLowerCase())
      );
      for (const cmd of filtered.slice(0, 6)) {
        suggestions.push({ text: cmd, type: "history", icon: "🕐" });
      }
    } catch (_) {}

    // Snippet suggestions
    try {
      const snippets = (await invoke("search_snippets", { query: partial })) as any[];
      for (const s of snippets.slice(0, 4)) {
        suggestions.push({ text: s.command, type: "snippet", icon: s.icon || "📋" });
      }
    } catch (_) {}

    // Path suggestions (if partial looks like a path or is after a space)
    const lastToken = partial.split(/\s+/).pop() || "";
    if (lastToken.includes("/") || lastToken.startsWith(".") || lastToken.startsWith("~")) {
      try {
        const paths = (await invoke("complete_path", {
          partial: lastToken,
          cwd: cwd() === "~" ? null : cwd(),
        })) as string[];
        for (const p of paths.slice(0, 6)) {
          suggestions.push({ text: p, type: "path", icon: p.endsWith("/") ? "📁" : "📄" });
        }
      } catch (_) {}
    }

    return suggestions;
  }

  return {
    isConnected,
    cwd,
    setCwd,
    createSession,
    write,
    resize,
    close,
    updateInfo,
    addToHistory,
    getAutocompleteSuggestions,
  };
}