import { createSignal, Show, For, onMount } from "solid-js";
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

const MAX_TOOL_STEPS = 20;

export default function MCPChat(props: Props) {
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [input, setInput] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [currentStep, setCurrentStep] = createSignal("");
  const [stepCount, setStepCount] = createSignal(0);
  const [servers, setServers] = createSignal<MCPServerInfo[]>([]);
  const [connectedCount, setConnectedCount] = createSignal(0);
  const [abortRequested, setAbortRequested] = createSignal(false);
  let messagesEndRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;

  onMount(async () => {
    await loadServers();
    inputRef?.focus();

    const connected = servers().filter((s) => s.connected);
    const toolCount = connected.reduce((a, s) => a + s.tools_count, 0);

    addMessage({
      role: "system",
      content:
        connected.length > 0
          ? `Connected to ${connected.length} MCP server${connected.length > 1 ? "s" : ""} with ${toolCount} tools available. I can execute multiple tools in sequence to complete complex tasks. Ask me anything!`
          : "No MCP servers connected. Open MCP Panel (⌘M) to start a server first.",
    });
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

  function addMessage(msg: Omit<ChatMessage, "id" | "timestamp">): ChatMessage {
    const newMsg: ChatMessage = {
      ...msg,
      id: crypto.randomUUID(),
      timestamp: Date.now(),
    };
    setMessages((prev) => [...prev, newMsg]);
    scrollToBottom();
    return newMsg;
  }

  // Build AI conversation history from our chat messages
  function buildAIHistory(): { role: string; content: string }[] {
    return messages()
      .filter((m) => m.role !== "system")
      .map((m) => {
        if (m.role === "tool_call" && m.toolCall) {
          return {
            role: "assistant",
            content: `I called tool "${m.toolCall.tool}" on server "${m.toolCall.server}" with arguments ${JSON.stringify(m.toolCall.arguments)}. Result: ${m.toolCall.result || "success"}`,
          };
        }
        if (m.role === "error") {
          return { role: "assistant", content: `Error: ${m.content}` };
        }
        return {
          role: m.role === "tool_result" ? "assistant" : m.role,
          content: m.content,
        };
      });
  }

  async function handleSubmit(e?: Event) {
    e?.preventDefault();
    const text = input().trim();
    if (!text || loading()) return;

    setInput("");
    setAbortRequested(false);
    addMessage({ role: "user", content: text });
    setLoading(true);
    setStepCount(0);

    try {
      await executeLoop(text);
    } catch (err: any) {
      addMessage({ role: "error", content: String(err) });
    } finally {
      setLoading(false);
      setCurrentStep("");
      setStepCount(0);
      inputRef?.focus();
    }
  }

  async function executeLoop(initialPrompt: string) {
    let step = 0;
    let consecutiveErrors = 0;
    const maxConsecutiveErrors = 3;

    while (step < MAX_TOOL_STEPS) {
      if (abortRequested()) {
        addMessage({
          role: "system",
          content: `⏹ Stopped after ${step} step${step !== 1 ? "s" : ""}.`,
        });
        return;
      }

      step++;
      setStepCount(step);
      setCurrentStep(`Step ${step}/${MAX_TOOL_STEPS}...`);

      const history = buildAIHistory();

      // If this is a continuation after tool calls, add a nudge
      if (step > 1) {
        history.push({
          role: "user",
          content:
            "Continue with the next step. If all tasks are complete, respond with a final summary message. Remember: use EXACT tool names without server prefix.",
        });
      }

      let response: any;
      try {
        response = await invoke("mcp_ai_step", { messages: history });
      } catch (err: any) {
        addMessage({ role: "error", content: `AI call failed: ${err}` });
        return;
      }

      // ── Final message from AI ──
      if (response.type === "message") {
        addMessage({ role: "assistant", content: response.content });

        // Check if AI is saying it needs to do more
        const lower = response.content.toLowerCase();
        const wantsContinue =
          lower.includes("let me continue") ||
          lower.includes("next, i'll") ||
          lower.includes("now i'll") ||
          lower.includes("proceeding to") ||
          lower.includes("moving on to");

        if (wantsContinue && step < MAX_TOOL_STEPS) {
          // AI sent a status message but wants to continue — keep going
          continue;
        }

        return; // Done!
      }

      // ── Tool call ──
      if (response.type === "tool_call") {
        consecutiveErrors = 0; // Reset error counter on success

        setCurrentStep(
          `Step ${step}: ${response.server}/${response.tool}`
        );

        addMessage({
          role: "tool_call",
          content: `${response.server}/${response.tool}`,
          toolCall: {
            server: response.server,
            tool: response.tool,
            arguments: response.arguments,
            result: response.is_error
              ? `❌ ${response.result}`
              : response.result,
            isError: response.is_error,
          },
        });

        if (response.is_error) {
          consecutiveErrors++;
          if (consecutiveErrors >= maxConsecutiveErrors) {
            addMessage({
              role: "system",
              content: `⚠️ ${maxConsecutiveErrors} consecutive tool errors. Stopping to prevent infinite loop.`,
            });
            return;
          }
        }

        // Continue loop — AI will see the tool result and decide next step
        continue;
      }

      // ── Tool error (name resolution failed) ──
      if (response.type === "tool_error") {
        consecutiveErrors++;

        addMessage({
          role: "error",
          content: `Tool "${response.tool}" failed: ${response.error}`,
        });

        if (consecutiveErrors >= maxConsecutiveErrors) {
          addMessage({
            role: "system",
            content: `⚠️ Too many errors. Stopping. Check if the MCP server has the tools you need.`,
          });
          return;
        }

        // Continue — AI will see the error and try a different approach
        continue;
      }

      // Unknown response type
      addMessage({
        role: "error",
        content: `Unexpected response type: ${response.type}`,
      });
      return;
    }

    // Max steps reached
    addMessage({
      role: "system",
      content: `⚠️ Reached maximum of ${MAX_TOOL_STEPS} steps. The task may not be fully complete.`,
    });

    // Ask AI for a summary of what was done
    try {
      const history = buildAIHistory();
      history.push({
        role: "user",
        content:
          "You've reached the step limit. Please provide a summary of what was completed and what remains to be done.",
      });

      const summary = (await invoke("mcp_ai_step", {
        messages: history,
      })) as any;

      if (summary.type === "message") {
        addMessage({ role: "assistant", content: summary.content });
      }
    } catch (_) {}
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function handleAbort() {
    setAbortRequested(true);
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  }

  function clearChat() {
    setMessages([]);
    loadServers().then(() => {
      const connected = servers().filter((s) => s.connected);
      const toolCount = connected.reduce((a, s) => a + s.tools_count, 0);
      addMessage({
        role: "system",
        content:
          connected.length > 0
            ? `Chat cleared. ${connected.length} server${connected.length > 1 ? "s" : ""} connected with ${toolCount} tools.`
            : "Chat cleared. No servers connected.",
      });
    });
  }

  const suggestions = [
    "Create a complete dashboard layout with sidebar, header, and cards",
    "Set up a project folder structure with config files",
    "Design a mobile app login screen with all UI elements",
    "Read all markdown files and summarize them",
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
              onClick={clearChat}
              title="Clear chat"
            >
              🗑️
            </button>
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

        {/* Progress bar during multi-step */}
        <Show when={loading() && stepCount() > 0}>
          <div class="mcpchat-progress">
            <div class="mcpchat-progress-bar">
              <div
                class="mcpchat-progress-fill"
                style={{
                  width: `${Math.min((stepCount() / MAX_TOOL_STEPS) * 100, 100)}%`,
                }}
              />
            </div>
            <div class="mcpchat-progress-info">
              <span>{currentStep()}</span>
              <button class="mcpchat-abort-btn" onClick={handleAbort}>
                ⏹ Stop
              </button>
            </div>
          </div>
        </Show>

        {/* Messages */}
        <div class="mcpchat-messages">
          <For each={messages()}>
            {(msg) => (
              <div class={`mcpchat-msg mcpchat-msg-${msg.role}`}>
                {/* System */}
                <Show when={msg.role === "system"}>
                  <div class="mcpchat-system">
                    <span class="mcpchat-system-icon">💡</span>
                    <pre class="mcpchat-system-text">{msg.content}</pre>
                  </div>
                </Show>

                {/* User */}
                <Show when={msg.role === "user"}>
                  <div class="mcpchat-bubble user">
                    <div class="mcpchat-bubble-header">
                      <span>You</span>
                      <span class="mcpchat-time">
                        {formatTime(msg.timestamp)}
                      </span>
                    </div>
                    <div class="mcpchat-bubble-content">{msg.content}</div>
                  </div>
                </Show>

                {/* Assistant */}
                <Show when={msg.role === "assistant"}>
                  <div class="mcpchat-bubble assistant">
                    <div class="mcpchat-bubble-header">
                      <span>🤖 AI</span>
                      <span class="mcpchat-time">
                        {formatTime(msg.timestamp)}
                      </span>
                    </div>
                    <div class="mcpchat-bubble-content">
                      <MessageContent
                        text={msg.content}
                        onRunCommand={props.onRunCommand}
                      />
                    </div>
                  </div>
                </Show>

                {/* Tool Call */}
                <Show when={msg.role === "tool_call" && msg.toolCall}>
                  <div
                    class={`mcpchat-tool-call ${msg.toolCall!.isError ? "errored" : ""}`}
                  >
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

                    <Show
                      when={
                        msg.toolCall!.arguments &&
                        Object.keys(msg.toolCall!.arguments).length > 0
                      }
                    >
                      <details class="mcpchat-tool-details">
                        <summary>Arguments</summary>
                        <pre>
                          {JSON.stringify(msg.toolCall!.arguments, null, 2)}
                        </pre>
                      </details>
                    </Show>

                    <Show when={msg.toolCall!.result}>
                      <details class="mcpchat-tool-details" open={msg.toolCall!.isError}>
                        <summary>
                          Result
                          <button
                            class="mcpchat-copy-btn"
                            onClick={() =>
                              navigator.clipboard.writeText(
                                msg.toolCall!.result || ""
                              )
                            }
                          >
                            📋
                          </button>
                        </summary>
                        <pre class="mcpchat-tool-result-content">
                          {msg.toolCall!.result!.length > 300
                            ? msg.toolCall!.result!.slice(0, 300) +
                              "\n... (truncated)"
                            : msg.toolCall!.result}
                        </pre>
                      </details>
                    </Show>
                  </div>
                </Show>

                {/* Error */}
                <Show when={msg.role === "error"}>
                  <div class="mcpchat-bubble error">
                    <div class="mcpchat-bubble-header">
                      <span>⚠️ Error</span>
                      <span class="mcpchat-time">
                        {formatTime(msg.timestamp)}
                      </span>
                    </div>
                    <div class="mcpchat-bubble-content">{msg.content}</div>
                  </div>
                </Show>
              </div>
            )}
          </For>

          {/* Loading */}
          <Show when={loading()}>
            <div class="mcpchat-loading">
              <div class="mcpchat-loading-dots">
                <span />
                <span />
                <span />
              </div>
              <span>{currentStep() || "Thinking..."}</span>
            </div>
          </Show>

          {/* Suggestions */}
          <Show when={messages().length <= 1 && !loading()}>
            <div class="mcpchat-suggestions">
              <span class="mcpchat-suggestions-label">
                Try a multi-step task:
              </span>
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
                loading()
                  ? "Working on it..."
                  : connectedCount() > 0
                    ? "Describe a complex task — I'll use multiple tools to complete it..."
                    : "Ask me anything..."
              }
              rows={2}
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
            <span>
              Enter to send · Shift+Enter for new line · Max{" "}
              {MAX_TOOL_STEPS} steps per task
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Code block renderer ──

function MessageContent(props: {
  text: string;
  onRunCommand: (cmd: string) => void;
}) {
  const parts = () => {
    const text = props.text;
    const result: {
      type: "text" | "code";
      content: string;
      lang?: string;
    }[] = [];
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
                  >
                    📋
                  </button>
                  <Show
                    when={
                      part.lang === "shell" ||
                      part.lang === "bash" ||
                      part.lang === "sh" ||
                      !part.lang
                    }
                  >
                    <button
                      class="mcpchat-code-btn run"
                      onClick={() =>
                        props.onRunCommand(part.content + "\n")
                      }
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