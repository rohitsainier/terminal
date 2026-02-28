import { createSignal, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useTerminal(sessionId: string) {
  const [isConnected, setIsConnected] = createSignal(false);
  const [cwd, setCwd] = createSignal("~");

  async function createSession(rows: number, cols: number, initialCwd?: string) {
    try {
      await invoke("create_session", {
        id: sessionId,
        rows,
        cols,
        cwd: initialCwd,
      });
      setIsConnected(true);
    } catch (e) {
      console.error("Failed to create session:", e);
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

  return {
    isConnected,
    cwd,
    createSession,
    write,
    resize,
    close,
  };
}