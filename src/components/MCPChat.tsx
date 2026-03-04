import { createSignal, Show, For, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { PlanStep, TaskPlan } from "../types";

// ─── Types ───────────────────────────────

interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "tool" | "error" | "system" | "plan";
  content: string;
  tool?: {
    server: string;
    name: string;
    arguments: any;
    result?: string;
    isError?: boolean;
  };
  plan?: TaskPlan;
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

  // Plan state
  const [plan, setPlan] = createSignal<TaskPlan | null>(null);
  const [planSteps, setPlanSteps] = createSignal<PlanStep[]>([]);
  const [currentPlanStep, setCurrentPlanStep] = createSignal(0);
  const [planMode, setPlanMode] = createSignal<
    "none" | "planning" | "reviewing" | "executing" | "done"
  >("none");

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
        ? `Ready — ${connected.length} MCP server${connected.length > 1 ? "s" : ""}, ${tools} tools. Describe any task and I'll plan it step by step.`
        : "No MCP servers connected. Start one from the MCP panel (\u2318M).",
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

  // Build the AI conversation from our visible messages
  function toAIMessages(): AIMessage[] {
    return messages()
      .filter((m) => m.role !== "system" && m.role !== "plan")
      .flatMap((m): AIMessage[] => {
        if (m.role === "user") {
          return [{ role: "user", content: m.content }];
        }
        if (m.role === "assistant") {
          return [{ role: "assistant", content: m.content }];
        }
        if (m.role === "tool" && m.tool) {
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

  // ─── Plan + Execute Flow ───────────────

  async function send() {
    const text = input().trim();
    if (!text || busy()) return;

    setInput("");
    setAborted(false);
    setToolSteps(0);
    push({ role: "user", content: text });
    setBusy(true);

    try {
      // Check if we have connected servers with tools
      const hasTools = servers().some((s) => s.connected && s.tools_count > 0);

      if (hasTools) {
        // Phase 1: Generate plan
        setPlanMode("planning");
        setStatus("Planning task...");

        const conversation = toAIMessages();
        let planResult: any;

        try {
          planResult = await invoke("mcp_ai_plan", { messages: conversation });
        } catch (err: any) {
          push({ role: "error", content: `Planning failed: ${err}` });
          setPlanMode("none");
          return;
        }

        if (aborted()) {
          push({ role: "system", content: "Planning cancelled." });
          setPlanMode("none");
          return;
        }

        // Parse the plan
        const rawPlan = planResult?.plan;
        if (!rawPlan || !rawPlan.steps || rawPlan.steps.length === 0) {
          // No valid plan — fall back to direct execution
          setPlanMode("none");
          await agentLoop(null);
          return;
        }

        const taskPlan: TaskPlan = {
          title: rawPlan.title || "Task",
          steps: rawPlan.steps.map((s: any) => ({
            step: s.step || 0,
            description: s.description || "",
            tool: s.tool || null,
            status: "pending" as const,
          })),
        };

        // Check if it's a simple single-step with no tool → skip plan UI
        if (taskPlan.steps.length === 1 && !taskPlan.steps[0].tool) {
          setPlanMode("none");
          await agentLoop(null);
          return;
        }

        setPlan(taskPlan);
        setPlanSteps(taskPlan.steps);
        setCurrentPlanStep(0);
        setPlanMode("reviewing");
        setStatus("");

        // Show plan as a message
        push({ role: "plan", content: "", plan: taskPlan });
      } else {
        // No MCP tools — just run the agent directly
        setPlanMode("none");
        await agentLoop(null);
      }
    } catch (err: any) {
      push({ role: "error", content: String(err) });
    } finally {
      if (planMode() !== "reviewing") {
        setBusy(false);
        setStatus("");
        setToolSteps(0);
        inputRef?.focus();
      } else {
        // Still reviewing — keep busy false so user can interact
        setBusy(false);
        setStatus("");
      }
    }
  }

  async function executePlan() {
    const currentPlan = plan();
    if (!currentPlan) return;

    setBusy(true);
    setPlanMode("executing");
    setAborted(false);

    try {
      await agentLoop(currentPlan);
    } catch (err: any) {
      push({ role: "error", content: String(err) });
    } finally {
      setBusy(false);
      setStatus("");
      setToolSteps(0);
      setPlanMode("done");
      setPlan(null);
      inputRef?.focus();
    }
  }

  function cancelPlan() {
    setPlan(null);
    setPlanSteps([]);
    setPlanMode("none");
    setBusy(false);
    setStatus("");
    push({ role: "system", content: "Plan cancelled." });
    inputRef?.focus();
  }

  // ─── Core Agent Loop ───────────────────

  async function agentLoop(taskPlan: TaskPlan | null) {
    let steps = 0;
    let consecutiveErrors = 0;
    let planStepIndex = 0;

    // If we have a plan, update the first step to running
    if (taskPlan) {
      updatePlanStepStatus(0, "running");
    }

    while (steps < STEP_LIMIT) {
      if (aborted()) {
        // Mark remaining plan steps as skipped
        if (taskPlan) {
          for (let i = planStepIndex; i < taskPlan.steps.length; i++) {
            updatePlanStepStatus(i, "skipped");
          }
        }
        push({ role: "system", content: `Stopped after ${steps} tool call${steps !== 1 ? "s" : ""}.` });
        return;
      }

      const planStepDesc = taskPlan && planStepIndex < taskPlan.steps.length
        ? `Step ${planStepIndex + 1}/${taskPlan.steps.length}: ${taskPlan.steps[planStepIndex].description}`
        : null;

      setStatus(
        planStepDesc
          ? `${planStepDesc}${steps > 0 ? ` (${steps} tool calls)` : ""}`
          : steps === 0
            ? "Thinking..."
            : `Thinking... (${steps} tool call${steps !== 1 ? "s" : ""} so far)`
      );

      if (taskPlan) {
        setCurrentPlanStep(planStepIndex);
      }

      // Call AI with full conversation + plan context
      const conversation = toAIMessages();
      let response: any;

      try {
        response = await invoke("mcp_ai_step", {
          messages: conversation,
          planStep: planStepDesc,
        });
      } catch (err: any) {
        push({ role: "error", content: `AI error: ${err}` });
        if (taskPlan) updatePlanStepStatus(planStepIndex, "error");
        return;
      }

      // ── AI sent a final message → display and stop ──
      if (response.type === "message") {
        push({ role: "assistant", content: response.content });

        if (taskPlan) {
          // Mark current step as completed
          updatePlanStepStatus(planStepIndex, "completed");

          // Move to next plan step
          planStepIndex++;
          if (planStepIndex < taskPlan.steps.length) {
            // More steps remain — continue the loop
            updatePlanStepStatus(planStepIndex, "running");
            continue;
          }
          // All steps done
        }
        return;
      }

      // ── AI wants to call a tool ──
      if (response.type === "tool_call") {
        steps++;
        consecutiveErrors = 0;
        setToolSteps(steps);
        setStatus(
          planStepDesc
            ? `${planStepDesc} — Running: ${response.tool}`
            : `Running: ${response.tool}`
        );

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
          if (taskPlan) updatePlanStepStatus(planStepIndex, "error");
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
    if (taskPlan) {
      for (let i = planStepIndex; i < taskPlan.steps.length; i++) {
        if (planSteps()[i]?.status === "running") updatePlanStepStatus(i, "error");
        else if (planSteps()[i]?.status === "pending") updatePlanStepStatus(i, "skipped");
      }
    }

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
      const summary = (await invoke("mcp_ai_step", {
        messages: conv,
        planStep: null,
      })) as any;
      if (summary.type === "message") {
        push({ role: "assistant", content: summary.content });
      }
    } catch (_) {}
  }

  function updatePlanStepStatus(index: number, status: PlanStep["status"]) {
    setPlanSteps((prev) =>
      prev.map((s, i) => (i === index ? { ...s, status } : s))
    );
  }

  // ─── Event Handlers ───────────────────

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }

  function clearChat() {
    setPlan(null);
    setPlanSteps([]);
    setPlanMode("none");
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

                {/* Plan */}
                <Show when={msg.role === "plan" && msg.plan}>
                  <div class="mcpc-plan-card">
                    <div class="mcpc-plan-header">
                      <span class="mcpc-plan-icon">📋</span>
                      <span class="mcpc-plan-title">{msg.plan!.title}</span>
                    </div>
                    <div class="mcpc-plan-steps">
                      <For each={planSteps()}>
                        {(step, i) => (
                          <div
                            class={`mcpc-plan-step ${step.status}`}
                            classList={{
                              "mcpc-plan-step-active": i() === currentPlanStep() && planMode() === "executing",
                            }}
                          >
                            <span class="mcpc-plan-step-indicator">
                              {step.status === "completed" ? "✓" :
                               step.status === "running" ? "▶" :
                               step.status === "error" ? "✗" :
                               step.status === "skipped" ? "–" :
                               String(step.step)}
                            </span>
                            <span class="mcpc-plan-step-desc">{step.description}</span>
                            <Show when={step.tool}>
                              <span class="mcpc-plan-step-tool">{step.tool}</span>
                            </Show>
                          </div>
                        )}
                      </For>
                    </div>
                    <Show when={planMode() === "reviewing"}>
                      <div class="mcpc-plan-actions">
                        <button class="mcpc-plan-execute" onClick={executePlan}>
                          ▶ Execute Plan
                        </button>
                        <button class="mcpc-plan-cancel" onClick={cancelPlan}>
                          Cancel
                        </button>
                      </div>
                    </Show>
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
            placeholder={
              busy()
                ? "Working..."
                : planMode() === "reviewing"
                  ? "Review the plan above, then Execute or Cancel"
                  : "Describe a task…"
            }
            rows={1}
            disabled={busy() || planMode() === "reviewing"}
          />
          <button
            class="mcpc-send"
            onClick={() => send()}
            disabled={busy() || !input().trim() || planMode() === "reviewing"}
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
