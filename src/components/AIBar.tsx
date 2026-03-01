import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useAI } from "../hooks/useAI";

interface Props {
  sessionId: string;
  onClose: () => void;
}

export default function AIBar(props: Props) {
  const [query, setQuery] = createSignal("");
  const ai = useAI();

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!query().trim()) return;
    await ai.translate(query(), "~");
  }

  async function runCommand() {
    const r = ai.result();
    if (!r) return;
    await invoke("write_to_session", {
      id: props.sessionId,
      data: r.command + "\n",
    });
    props.onClose();
  }

  async function explainCommand() {
    const r = ai.result();
    if (!r) return;
    const explanation = await ai.explain(r.command);
    if (explanation) {
      // Merge explanation into existing result
      ai.result(); // trigger reactivity read
      // We need to update result — use the suggestFix pattern or manually
      // Since useAI doesn't expose setResult, let's add a small workaround:
      // Actually the explain() doesn't update result, it returns the string.
      // Let's show it inline.
      setExplanationText(explanation);
    }
  }

  const [explanationText, setExplanationText] = createSignal("");

  return (
    <div class="ai-overlay" onClick={() => props.onClose()}>
      <div class="ai-bar" onClick={(e) => e.stopPropagation()}>
        <div class="ai-header">
          <span class="ai-icon">⚡</span>
          <span>AI Command Bar</span>
          <span class="ai-shortcut">⌘K</span>
        </div>

        <form onSubmit={handleSubmit}>
          <input
            class="ai-input"
            type="text"
            placeholder="Describe what you want to do..."
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
            autofocus
          />
        </form>

        {ai.loading() && (
          <div class="ai-loading">
            <span class="spinner" /> Thinking...
          </div>
        )}

        {ai.error() && <div class="ai-error">❌ {ai.error()}</div>}

        {ai.result() && (
          <div class="ai-result">
            <div class="ai-command">
              <code>{ai.result()!.command}</code>
              {ai.result()!.dangerous && (
                <span class="ai-danger">
                  ⚠️ {ai.result()!.danger_reason || "Potentially dangerous"}
                </span>
              )}
            </div>

            <div class="ai-explanation">
              {explanationText() || ai.result()!.explanation}
            </div>

            <div class="ai-actions">
              <button class="btn-run" onClick={runCommand}>
                ▶ Run
              </button>
              <button
                class="btn-copy"
                onClick={() => navigator.clipboard.writeText(ai.result()!.command)}
              >
                📋 Copy
              </button>
              <button class="btn-explain" onClick={explainCommand}>
                💡 Explain
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}