import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Snippet {
  id: string;
  name: string;
  command: string;
  icon: string;
  tags: string[];
}

interface Props {
  sessionId: string;
  onClose: () => void;
}

export default function SnippetLibrary(props: Props) {
  const [snippets, setSnippets] = createSignal<Snippet[]>([]);
  const [search, setSearch] = createSignal("");
  const [showAdd, setShowAdd] = createSignal(false);
  const [newName, setNewName] = createSignal("");
  const [newCommand, setNewCommand] = createSignal("");
  const [newIcon, setNewIcon] = createSignal("📌");

  onMount(async () => {
    const list = (await invoke("list_snippets")) as Snippet[];
    setSnippets(list);
  });

  function filtered() {
    const q = search().toLowerCase();
    if (!q) return snippets();
    return snippets().filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.command.toLowerCase().includes(q) ||
        s.tags.some((t) => t.toLowerCase().includes(q))
    );
  }

  async function runSnippet(snippet: Snippet) {
    await invoke("run_snippet", {
      id: snippet.id,
      sessionId: props.sessionId,
    });
    props.onClose();
  }

  async function addSnippet() {
    if (!newName() || !newCommand()) return;

    const snippet: Snippet = {
      id: crypto.randomUUID(),
      name: newName(),
      command: newCommand(),
      icon: newIcon(),
      tags: [],
    };

    await invoke("add_snippet", { snippet });
    setSnippets([...snippets(), snippet]);
    setNewName("");
    setNewCommand("");
    setShowAdd(false);
  }

  async function deleteSnippet(id: string) {
    await invoke("delete_snippet", { id });
    setSnippets(snippets().filter((s) => s.id !== id));
  }

  return (
    <div class="palette-overlay" onClick={() => props.onClose()}>
      <div class="snippet-library" onClick={(e) => e.stopPropagation()}>
        <div class="snippet-header">
          <span>📋 Snippet Library</span>
          <button class="snippet-add-btn" onClick={() => setShowAdd(!showAdd())}>
            {showAdd() ? "Cancel" : "+ New"}
          </button>
        </div>

        {showAdd() && (
          <div class="snippet-form">
            <div class="snippet-form-row">
              <input
                class="snippet-form-icon"
                value={newIcon()}
                onInput={(e) => setNewIcon(e.currentTarget.value)}
                maxLength={2}
              />
              <input
                class="snippet-form-input"
                placeholder="Snippet name..."
                value={newName()}
                onInput={(e) => setNewName(e.currentTarget.value)}
              />
            </div>
            <input
              class="snippet-form-input full"
              placeholder="Command..."
              value={newCommand()}
              onInput={(e) => setNewCommand(e.currentTarget.value)}
              onKeyDown={(e) => e.key === "Enter" && addSnippet()}
            />
            <button class="snippet-save-btn" onClick={addSnippet}>
              Save Snippet
            </button>
          </div>
        )}

        <input
          class="palette-input"
          type="text"
          placeholder="Search snippets..."
          value={search()}
          onInput={(e) => setSearch(e.currentTarget.value)}
          autofocus
        />

        <div class="snippet-list">
          {filtered().map((snippet) => (
            <div class="snippet-item">
              <div class="snippet-item-main" onClick={() => runSnippet(snippet)}>
                <span class="snippet-item-icon">{snippet.icon}</span>
                <div class="snippet-item-info">
                  <span class="snippet-item-name">{snippet.name}</span>
                  <code class="snippet-item-cmd">{snippet.command}</code>
                </div>
              </div>
              <div class="snippet-item-actions">
                <button
                  class="snippet-copy"
                  onClick={() => navigator.clipboard.writeText(snippet.command)}
                  title="Copy"
                >
                  📋
                </button>
                <button
                  class="snippet-delete"
                  onClick={() => deleteSnippet(snippet.id)}
                  title="Delete"
                >
                  🗑️
                </button>
              </div>
            </div>
          ))}
          {filtered().length === 0 && (
            <div class="snippet-empty">No snippets found</div>
          )}
        </div>
      </div>
    </div>
  );
}