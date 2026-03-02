import { createSignal, Show, For, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// ─── Types ───────────────────────────────

interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "tool" | "error" | "system";
  content: string;
  tool?: {
    server: string;
    name: string;
    arguments: any;
    result?: string;
    isError?: boolean;
  };
  timestamp: number;
}

// Internal conversation format sent to backend
interface AIMessage {
  role: string;
  content: string;
}

interface MCPServerInfo {
  name: string;
  connected: boolean;
  tools_count: number;
  tools: { name: string; description: string }[];
}

interface Props {
  onClose: () => void;
  onRunCommand: (cmd: string) => void;
}

// Safety limit
const STEP_LIMIT = 30;

export default function MCPChat(props: Props) {
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [input, setInput] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [status, setStatus] = createSignal("");
  const [toolSteps, setToolSteps] = createSignal(0);
  const [servers, setServers] = createSignal<MCPServerInfo[]>([]);
  const [aborted, setAborted] = createSignal(false);

  let messagesEnd: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;

  // ─── Init ──────────────────────────────

  onMount(async () => {
    await refreshServers();
    inputRef?.focus();

    const connected = servers().filter((s) => s.connected);
    const tools = connected.reduce((a, s) => a + s.tools_count, 0);

    push({
      role: "system",
      content: connected.length
        ? `Ready — ${connected.length} MCP server${connected.length > 1 ? "s" : ""}, ${tools} tools. Describe any task and I'll handle it step by step.`
        : "No MCP servers connected. Start one from the MCP panel (⌘M).",
    });
  });

  async function refreshServers() {
    try {
      const list = (await invoke("mcp_list_servers")) as MCPServerInfo[];
      setServers(Array.isArray(list) ? list : []);
    } catch (_) {}
  }

  // ─── Helpers ───────────────────────────

  function push(partial: Omit<ChatMessage, "id" | "timestamp">): ChatMessage {
    const msg: ChatMessage = { ...partial, id: crypto.randomUUID(), timestamp: Date.now() };
    setMessages((prev) => [...prev, msg]);
    setTimeout(() => messagesEnd?.scrollIntoView({ behavior: "smooth" }), 40);
    return msg;
  }

  function scroll() {
    setTimeout(() => messagesEnd?.scrollIntoView({ behavior: "smooth" }), 40);
  }

  // Build the AI conversation from our visible messages
  function toAIMessages(): AIMessage[] {
    return messages()
      .filter((m) => m.role !== "system")
      .flatMap((m): AIMessage[] => {
        if (m.role === "user") {
          return [{ role: "user", content: m.content }];
        }
        if (m.role === "assistant") {
          return [{ role: "assistant", content: m.content }];
        }
        if (m.role === "tool" && m.tool) {
          // The AI needs to know what it did and what came back
          return [
            {
              role: "assistant",
              content: `I called tool "${m.tool.name}" on server "${m.tool.server}" with arguments: ${JSON.stringify(m.tool.arguments)}`,
            },
            {
              role: "tool_result",
              content: m.tool.isError
                ? `Error: ${m.tool.result}`
                : m.tool.result || "Done (no output)",
            },
          ];
        }
        if (m.role === "error") {
          return [{ role: "tool_error", content: m.content }];
        }
        return [];
      });
  }

  // ─── Core Agent Loop ───────────────────

  async function send() {
    const text = input().trim();
    if (!text || busy()) return;

    setInput("");
    setAborted(false);
    setToolSteps(0);
    push({ role: "user", content: text });
    setBusy(true);

    try {
      await agentLoop();
    } catch (err: any) {
      push({ role: "error", content: String(err) });
    } finally {
      setBusy(false);
      setStatus("");
      setToolSteps(0);
      inputRef?.focus();
    }
  }

  async function agentLoop() {
    let steps = 0;
    let consecutiveErrors = 0;

    while (steps < STEP_LIMIT) {
      if (aborted()) {
        push({ role: "system", content: `Stopped after ${steps} tool call${steps !== 1 ? "s" : ""}.` });
        return;
      }

      setStatus(steps === 0 ? "Thinking..." : `Thinking... (${steps} tool call${steps !== 1 ? "s" : ""} so far)`);

      // Call AI with full conversation
      const conversation = toAIMessages();
      let response: any;

      try {
        response = await invoke("mcp_ai_step", { messages: conversation });
      } catch (err: any) {
        push({ role: "error", content: `AI error: ${err}` });
        return;
      }

      // ── AI sent a final message → display and stop ──
      if (response.type === "message") {
        push({ role: "assistant", content: response.content });
        return;
      }

      // ── AI wants to call a tool ──
      if (response.type === "tool_call") {
        steps++;
        consecutiveErrors = 0;
        setToolSteps(steps);
        setStatus(`Running: ${response.tool}`);

        push({
          role: "tool",
          content: "",
          tool: {
            server: response.server,
            name: response.tool,
            arguments: response.arguments,
            result: response.result,
            isError: response.is_error,
          },
        });

        if (response.is_error) {
          consecutiveErrors++;
        }

        // Loop continues — AI will see the tool result next iteration
        continue;
      }

      // ── Tool name resolution error ──
      if (response.type === "tool_error") {
        steps++;
        consecutiveErrors++;
        setToolSteps(steps);

        push({
          role: "tool",
          content: "",
          tool: {
            server: response.server,
            name: response.tool,
            arguments: response.arguments,
            result: `Error: ${response.error}`,
            isError: true,
          },
        });

        if (consecutiveErrors >= 3) {
          push({
            role: "system",
            content: "Too many consecutive errors — stopping to avoid an infinite loop.",
          });
          return;
        }

        continue;
      }

      // Unknown
      push({ role: "error", content: `Unknown response: ${JSON.stringify(response)}` });
      return;
    }

    // Hit the safety limit
    push({
      role: "system",
      content: `Reached the ${STEP_LIMIT}-step safety limit. Asking AI for a summary of progress...`,
    });

    // One final call asking for a summary
    try {
      const conv = toAIMessages();
      conv.push({
        role: "user",
        content: "You've reached the step limit. Summarize what was completed and what remains.",
      });
      const summary = (await invoke("mcp_ai_step", { messages: conv })) as any;
      if (summary.type === "message") {
        push({ role: "assistant", content: summary.content });
      }
    } catch (_) {}
  }

  // ─── Event Handlers ───────────────────

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }

  function clearChat() {
    setMessages([]);
    refreshServers().then(() => {
      const c = servers().filter((s) => s.connected);
      push({
        role: "system",
        content: c.length
          ? `Chat cleared. ${c.length} server${c.length > 1 ? "s" : ""} ready.`
          : "Chat cleared. No servers connected.",
      });
    });
  }

  function formatTime(ts: number) {
    return new Date(ts).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  }

  const connected = () => servers().filter((s) => s.connected);

  // ─── Render ────────────────────────────

  return (
    <div class="mcpchat-overlay" onClick={() => props.onClose()}>
      <div class="mcpchat-panel" onClick={(e) => e.stopPropagation()}>

        {/* ── Header ── */}
        <div class="mcpchat-header">
          <div class="mcpchat-header-left">
            <span style={{ "font-size": "18px" }}>🤖</span>
            <span style={{ "font-weight": "600" }}>MCP Chat</span>
            <Show when={connected().length > 0}>
              <span class="mcpchat-badge connected">
                {connected().length} server{connected().length > 1 ? "s" : ""}
              </span>
            </Show>
          </div>
          <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
            <button class="mcpchat-icon-btn" onClick={clearChat} title="Clear">🗑️</button>
            <button class="mcpchat-icon-btn" onClick={refreshServers} title="Refresh">🔄</button>
            <span class="mcpchat-close" onClick={() => props.onClose()}>×</span>
          </div>
        </div>

        {/* ── Server chips ── */}
        <Show when={connected().length > 0}>
          <div class="mcpchat-chips">
            <For each={connected()}>
              {(s) => (
                <span class="mcpchat-chip">
                  <span class="mcpchat-chip-dot" />
                  {s.name}
                  <span class="mcpchat-chip-count">{s.tools_count}</span>
                </span>
              )}
            </For>
          </div>
        </Show>

        {/* ── Messages ── */}
        <div class="mcpchat-messages">
          <For each={messages()}>
            {(msg) => (
              <>
                {/* System */}
                <Show when={msg.role === "system"}>
                  <div class="mcpc-system">
                    <span>💡</span>
                    <span>{msg.content}</span>
                  </div>
                </Show>

                {/* User */}
                <Show when={msg.role === "user"}>
                  <div class="mcpc-row mcpc-user">
                    <div class="mcpc-bubble mcpc-bubble-user">
                      {msg.content}
                    </div>
                    <span class="mcpc-time">{formatTime(msg.timestamp)}</span>
                  </div>
                </Show>

                {/* Assistant */}
                <Show when={msg.role === "assistant"}>
                  <div class="mcpc-row mcpc-ai">
                    <div class="mcpc-avatar">🤖</div>
                    <div class="mcpc-bubble mcpc-bubble-ai">
                      <RichText text={msg.content} onRun={props.onRunCommand} />
                    </div>
                  </div>
                </Show>

                {/* Tool Call */}
                <Show when={msg.role === "tool" && msg.tool}>
                  <div class="mcpc-tool">
                    <div class="mcpc-tool-header">
                      <span class="mcpc-tool-icon">
                        {msg.tool!.isError ? "❌" : "✅"}
                      </span>
                      <span class="mcpc-tool-name">{msg.tool!.name}</span>
                      <span class="mcpc-tool-server">{msg.tool!.server}</span>
                    </div>

                    <Show when={msg.tool!.arguments && Object.keys(msg.tool!.arguments).length > 0}>
                      <details class="mcpc-tool-section">
                        <summary>Arguments</summary>
                        <pre>{JSON.stringify(msg.tool!.arguments, null, 2)}</pre>
                      </details>
                    </Show>

                    <Show when={msg.tool!.result}>
                      <details class="mcpc-tool-section" open={!!msg.tool!.isError}>
                        <summary>
                          <span>Result</span>
                          <button
                            class="mcpc-copy"
                            onClick={() => navigator.clipboard.writeText(msg.tool!.result || "")}
                          >
                            📋
                          </button>
                        </summary>
                        <pre>
                          {(msg.tool!.result || "").length > 500
                            ? msg.tool!.result!.slice(0, 500) + "\n…truncated"
                            : msg.tool!.result}
                        </pre>
                      </details>
                    </Show>
                  </div>
                </Show>

                {/* Error */}
                <Show when={msg.role === "error"}>
                  <div class="mcpc-system mcpc-error-msg">
                    <span>⚠️</span>
                    <span>{msg.content}</span>
                  </div>
                </Show>
              </>
            )}
          </For>

          {/* Busy indicator */}
          <Show when={busy()}>
            <div class="mcpc-busy">
              <div class="mcpc-dots"><span /><span /><span /></div>
              <span class="mcpc-busy-text">{status()}</span>
              <button class="mcpc-stop" onClick={() => setAborted(true)}>
                Stop
              </button>
            </div>
          </Show>

          {/* Starter suggestions */}
          <Show when={messages().length <= 1 && !busy()}>
            <div class="mcpc-suggestions">
              {[
                "Create a landing page layout with hero, features, and footer",
                "Set up a complete project folder structure",
                "Design a mobile app login flow with all screens",
                "Read all files in a folder and summarize them",
              ].map((s) => (
                <button class="mcpc-suggest" onClick={() => { setInput(s); send(); }}>
                  {s}
                </button>
              ))}
            </div>
          </Show>

          <div ref={messagesEnd} />
        </div>

        {/* ── Input ── */}
        <div class="mcpc-input-bar">
          <textarea
            ref={inputRef}
            class="mcpc-input"
            value={input()}
            onInput={(e) => setInput(e.currentTarget.value)}
            onKeyDown={onKeyDown}
            placeholder={busy() ? "Working..." : "Describe a task…"}
            rows={1}
            disabled={busy()}
          />
          <button
            class="mcpc-send"
            onClick={() => send()}
            disabled={busy() || !input().trim()}
          >
            {busy() ? "…" : "↑"}
          </button>
        </div>
      </div>
    </div>
  );
}

// ─── Rich text: renders code blocks with copy/run buttons ───

function RichText(props: { text: string; onRun: (cmd: string) => void }) {
  const segments = () => {
    const t = props.text;
    const out: { kind: "text" | "code"; body: string; lang?: string }[] = [];
    const re = /```(\w*)\n?([\s\S]*?)```/g;
    let last = 0;
    let m;
    while ((m = re.exec(t)) !== null) {
      if (m.index > last) out.push({ kind: "text", body: t.slice(last, m.index) });
      out.push({ kind: "code", body: m[2].trim(), lang: m[1] || "sh" });
      last = m.index + m[0].length;
    }
    if (last < t.length) out.push({ kind: "text", body: t.slice(last) });
    if (!out.length) out.push({ kind: "text", body: t });
    return out;
  };

  return (
    <For each={segments()}>
      {(seg) =>
        seg.kind === "text" ? (
          <span style={{ "white-space": "pre-wrap" }}>{seg.body}</span>
        ) : (
          <div class="mcpc-code">
            <div class="mcpc-code-bar">
              <span>{seg.lang}</span>
              <div style={{ display: "flex", gap: "4px" }}>
                <button class="mcpc-code-btn" onClick={() => navigator.clipboard.writeText(seg.body)}>
                  Copy
                </button>
                <Show when={["sh", "bash", "shell", "zsh"].includes(seg.lang || "")}>
                  <button class="mcpc-code-btn mcpc-run" onClick={() => props.onRun(seg.body + "\n")}>
                    ▶ Run
                  </button>
                </Show>
              </div>
            </div>
            <pre>{seg.body}</pre>
          </div>
        )
      }
    </For>
  );
}