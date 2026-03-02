import { createSignal, Show, For, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "tool_call" | "tool_result" | "error" | "system";
  content: string;
  toolCall?: {
    server: string;
    tool: string;
    arguments: any;
    result?: string;
    isError?: boolean;
  };
  timestamp: number;
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

export default function MCPChat(props: Props) {
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [input, setInput] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [servers, setServers] = createSignal<MCPServerInfo[]>([]);
  const [connectedCount, setConnectedCount] = createSignal(0);
  let messagesEndRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;

  onMount(async () => {
    await loadServers();
    inputRef?.focus();

    // Welcome message
    const connected = servers().filter((s) => s.connected);
    const toolCount = connected.reduce((a, s) => a + s.tools_count, 0);

    setMessages([
      {
        id: "welcome",
        role: "system",
        content:
          connected.length > 0
            ? `Connected to ${connected.length} MCP server${connected.length > 1 ? "s" : ""} with ${toolCount} tools. Ask me anything!`
            : "No MCP servers connected. Open MCP Panel (⌘M) to start a server, or ask me anything — I can still help with terminal commands.",
        timestamp: Date.now(),
      },
    ]);
  });

  async function loadServers() {
    try {
      const list = (await invoke("mcp_list_servers")) as MCPServerInfo[];
      setServers(Array.isArray(list) ? list : []);
      setConnectedCount(list.filter((s) => s.connected).length);
    } catch (_) {}
  }

  function scrollToBottom() {
    setTimeout(() => {
      messagesEndRef?.scrollIntoView({ behavior: "smooth" });
    }, 50);
  }

  function addMessage(msg: Omit<ChatMessage, "id" | "timestamp">) {
    const newMsg: ChatMessage = {
      ...msg,
      id: crypto.randomUUID(),
      timestamp: Date.now(),
    };
    setMessages((prev) => [...prev, newMsg]);
    scrollToBottom();
    return newMsg;
  }

  async function handleSubmit(e?: Event) {
    e?.preventDefault();
    const text = input().trim();
    if (!text || loading()) return;

    setInput("");
    addMessage({ role: "user", content: text });
    setLoading(true);

    try {
      // Build message history for AI
      const history = messages()
        .filter((m) => m.role !== "system")
        .map((m) => {
          if (m.role === "tool_call" && m.toolCall) {
            return {
              role: "assistant",
              content: `I called tool ${m.toolCall.server}/${m.toolCall.tool} and got: ${m.toolCall.result || "no result"}`,
            };
          }
          return {
            role: m.role === "error" ? "assistant" : m.role,
            content: m.content,
          };
        });

      // Add current user message
      history.push({ role: "user", content: text });

      // Call AI with MCP context
      const response = (await invoke("mcp_ai_chat", {
        messages: history,
      })) as any;

      if (response.type === "message") {
        addMessage({ role: "assistant", content: response.content });
      } else if (response.type === "tool_call") {
        // Show tool call
        const toolMsg = addMessage({
          role: "tool_call",
          content: `Calling ${response.server}/${response.tool}...`,
          toolCall: {
            server: response.server,
            tool: response.tool,
            arguments: response.arguments,
            result: response.result,
            isError: response.is_error,
          },
        });

        // Get AI to summarize the result
        if (!response.is_error) {
          try {
            const summary = (await invoke("mcp_ai_followup", {
              messages: history,
              toolResult: response.result,
            })) as string;

            addMessage({ role: "assistant", content: summary });
          } catch (_) {
            // If summary fails, show raw result
            addMessage({
              role: "assistant",
              content: response.result || "Tool executed successfully.",
            });
          }
        } else {
          addMessage({
            role: "error",
            content: `Tool error: ${response.result}`,
          });
        }
      }
    } catch (err: any) {
      addMessage({
        role: "error",
        content: String(err),
      });
    } finally {
      setLoading(false);
      inputRef?.focus();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  }

  // Quick actions
  const suggestions = [
    "What files are in the current directory?",
    "Show me system information",
    "Search for large files",
    "Read the README.md file",
  ];

  return (
    <div class="mcpchat-overlay" onClick={() => props.onClose()}>
      <div class="mcpchat-panel" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="mcpchat-header">
          <div class="mcpchat-header-left">
            <span class="mcpchat-header-icon">🤖</span>
            <span>AI + MCP Chat</span>
            <Show when={connectedCount() > 0}>
              <span class="mcpchat-badge connected">
                🔌 {connectedCount()} server{connectedCount() > 1 ? "s" : ""}
              </span>
            </Show>
            <Show when={connectedCount() === 0}>
              <span class="mcpchat-badge disconnected">No servers</span>
            </Show>
          </div>
          <div class="mcpchat-header-right">
            <button
              class="mcpchat-header-btn"
              onClick={loadServers}
              title="Refresh servers"
            >
              🔄
            </button>
            <span class="mcpchat-close" onClick={() => props.onClose()}>
              ×
            </span>
          </div>
        </div>

        {/* Connected tools bar */}
        <Show when={connectedCount() > 0}>
          <div class="mcpchat-tools-bar">
            <For each={servers().filter((s) => s.connected)}>
              {(server) => (
                <div class="mcpchat-server-chip">
                  <span class="mcpchat-server-dot" />
                  <span>{server.name}</span>
                  <span class="mcpchat-tool-count">{server.tools_count}</span>
                </div>
              )}
            </For>
          </div>
        </Show>

        {/* Messages */}
        <div class="mcpchat-messages">
          <For each={messages()}>
            {(msg) => (
              <div class={`mcpchat-msg mcpchat-msg-${msg.role}`}>
                {/* System message */}
                <Show when={msg.role === "system"}>
                  <div class="mcpchat-system">
                    <span class="mcpchat-system-icon">💡</span>
                    <span>{msg.content}</span>
                  </div>
                </Show>

                {/* User message */}
                <Show when={msg.role === "user"}>
                  <div class="mcpchat-bubble user">
                    <div class="mcpchat-bubble-header">
                      <span>You</span>
                      <span class="mcpchat-time">{formatTime(msg.timestamp)}</span>
                    </div>
                    <div class="mcpchat-bubble-content">{msg.content}</div>
                  </div>
                </Show>

                {/* Assistant message */}
                <Show when={msg.role === "assistant"}>
                  <div class="mcpchat-bubble assistant">
                    <div class="mcpchat-bubble-header">
                      <span>🤖 AI</span>
                      <span class="mcpchat-time">{formatTime(msg.timestamp)}</span>
                    </div>
                    <div class="mcpchat-bubble-content">
                      <MessageContent
                        text={msg.content}
                        onRunCommand={props.onRunCommand}
                      />
                    </div>
                  </div>
                </Show>

                {/* Tool call */}
                <Show when={msg.role === "tool_call" && msg.toolCall}>
                  <div class="mcpchat-tool-call">
                    <div class="mcpchat-tool-call-header">
                      <span class="mcpchat-tool-icon">🛠️</span>
                      <span>
                        {msg.toolCall!.server}/{msg.toolCall!.tool}
                      </span>
                      <span
                        class={`mcpchat-tool-status ${msg.toolCall!.isError ? "error" : "success"}`}
                      >
                        {msg.toolCall!.isError ? "❌" : "✅"}
                      </span>
                    </div>

                    {/* Arguments */}
                    <Show when={msg.toolCall!.arguments && Object.keys(msg.toolCall!.arguments).length > 0}>
                      <div class="mcpchat-tool-args">
                        <span class="mcpchat-tool-label">Arguments:</span>
                        <pre>{JSON.stringify(msg.toolCall!.arguments, null, 2)}</pre>
                      </div>
                    </Show>

                    {/* Result */}
                    <Show when={msg.toolCall!.result}>
                      <div class="mcpchat-tool-result-box">
                        <div class="mcpchat-tool-result-header">
                          <span class="mcpchat-tool-label">Result</span>
                          <button
                            class="mcpchat-copy-btn"
                            onClick={() =>
                              navigator.clipboard.writeText(msg.toolCall!.result || "")
                            }
                          >
                            📋
                          </button>
                        </div>
                        <pre class="mcpchat-tool-result-content">
                          {msg.toolCall!.result!.length > 500
                            ? msg.toolCall!.result!.slice(0, 500) + "\n... (truncated)"
                            : msg.toolCall!.result}
                        </pre>
                      </div>
                    </Show>
                  </div>
                </Show>

                {/* Error */}
                <Show when={msg.role === "error"}>
                  <div class="mcpchat-bubble error">
                    <div class="mcpchat-bubble-header">
                      <span>⚠️ Error</span>
                      <span class="mcpchat-time">{formatTime(msg.timestamp)}</span>
                    </div>
                    <div class="mcpchat-bubble-content">{msg.content}</div>
                  </div>
                </Show>
              </div>
            )}
          </For>

          {/* Loading indicator */}
          <Show when={loading()}>
            <div class="mcpchat-loading">
              <div class="mcpchat-loading-dots">
                <span />
                <span />
                <span />
              </div>
              <span>Thinking...</span>
            </div>
          </Show>

          {/* Suggestions (show only when no messages besides system) */}
          <Show when={messages().length <= 1 && !loading()}>
            <div class="mcpchat-suggestions">
              <span class="mcpchat-suggestions-label">Try asking:</span>
              <For each={suggestions}>
                {(s) => (
                  <button
                    class="mcpchat-suggestion"
                    onClick={() => {
                      setInput(s);
                      handleSubmit();
                    }}
                  >
                    {s}
                  </button>
                )}
              </For>
            </div>
          </Show>

          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div class="mcpchat-input-area">
          <form onSubmit={handleSubmit} class="mcpchat-form">
            <textarea
              ref={inputRef}
              class="mcpchat-input"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder={
                connectedCount() > 0
                  ? "Ask me anything... I can use MCP tools to help"
                  : "Ask me anything..."
              }
              rows={1}
              disabled={loading()}
            />
            <button
              class="mcpchat-send"
              type="submit"
              disabled={loading() || !input().trim()}
            >
              {loading() ? "⏳" : "➤"}
            </button>
          </form>
          <div class="mcpchat-input-hint">
            <span>Enter to send · Shift+Enter for new line · Esc to close</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Helper: render message with code blocks and run buttons ──

function MessageContent(props: { text: string; onRunCommand: (cmd: string) => void }) {
  // Split text into regular text and code blocks
  const parts = () => {
    const text = props.text;
    const result: { type: "text" | "code"; content: string; lang?: string }[] = [];
    const codeRegex = /```(\w*)\n?([\s\S]*?)```/g;
    let lastIndex = 0;
    let match;

    while ((match = codeRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        result.push({
          type: "text",
          content: text.slice(lastIndex, match.index),
        });
      }
      result.push({
        type: "code",
        content: match[2].trim(),
        lang: match[1] || "shell",
      });
      lastIndex = match.index + match[0].length;
    }

    if (lastIndex < text.length) {
      result.push({ type: "text", content: text.slice(lastIndex) });
    }

    if (result.length === 0) {
      result.push({ type: "text", content: text });
    }

    return result;
  };

  return (
    <div class="mcpchat-content-parsed">
      <For each={parts()}>
        {(part) => (
          <Show
            when={part.type === "code"}
            fallback={<span class="mcpchat-text">{part.content}</span>}
          >
            <div class="mcpchat-code-block">
              <div class="mcpchat-code-header">
                <span>{part.lang}</span>
                <div class="mcpchat-code-actions">
                  <button
                    class="mcpchat-code-btn"
                    onClick={() =>
                      navigator.clipboard.writeText(part.content)
                    }
                    title="Copy"
                  >
                    📋
                  </button>
                  <Show when={part.lang === "shell" || part.lang === "bash" || part.lang === "sh" || !part.lang}>
                    <button
                      class="mcpchat-code-btn run"
                      onClick={() => props.onRunCommand(part.content + "\n")}
                      title="Run in terminal"
                    >
                      ▶ Run
                    </button>
                  </Show>
                </div>
              </div>
              <pre class="mcpchat-code-content">{part.content}</pre>
            </div>
          </Show>
        )}
      </For>
    </div>
  );
}