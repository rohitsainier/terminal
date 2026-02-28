import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  sessionId: string;
  onClose: () => void;
}

export default function AIBar(props: Props) {
  const [query, setQuery] = createSignal("");
  const [result, setResult] = createSignal<any>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!query().trim()) return;

    setLoading(true);
    setError("");
    setResult(null);

    try {
      const response = await invoke("ai_translate_command", {
        query: query(),
        cwd: "~",
      });
      setResult(response);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  }

  async function runCommand() {
    if (!result()) return;
    await invoke("write_to_session", {
      id: props.sessionId,
      data: result().command + "\n",
    });
    props.onClose();
  }

  async function explainCommand() {
    if (!result()) return;
    try {
      const explanation = await invoke("ai_explain_command", {
        command: result().command,
      });
      setResult({ ...result(), explanation });
    } catch (err: any) {
      setError(err.toString());
    }
  }

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

        {loading() && (
          <div class="ai-loading">
            <span class="spinner" /> Thinking...
          </div>
        )}

        {error() && <div class="ai-error">❌ {error()}</div>}

        {result() && (
          <div class="ai-result">
            <div class="ai-command">
              <code>{result().command}</code>
              {result().dangerous && (
                <span class="ai-danger">
                  ⚠️ {result().danger_reason || "Potentially dangerous"}
                </span>
              )}
            </div>

            <div class="ai-explanation">{result().explanation}</div>

            <div class="ai-actions">
              <button class="btn-run" onClick={runCommand}>
                ▶ Run
              </button>
              <button class="btn-copy" onClick={() => {
                navigator.clipboard.writeText(result().command);
              }}>
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