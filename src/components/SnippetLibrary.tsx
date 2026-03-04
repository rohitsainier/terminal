import { createSignal, createMemo, onMount, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { Snippet } from "../types";

interface Props {
  sessionId: string;
  onClose: () => void;
}

export default function SnippetLibrary(props: Props) {
  const [snippets, setSnippets] = createSignal<Snippet[]>([]);
  const [categories, setCategories] = createSignal<string[]>([]);
  const [selectedCategory, setSelectedCategory] = createSignal("all");
  const [search, setSearch] = createSignal("");
  const [showAdd, setShowAdd] = createSignal(false);
  const [showImport, setShowImport] = createSignal(false);
  const [importJson, setImportJson] = createSignal("");
  const [importMsg, setImportMsg] = createSignal("");
  const [newName, setNewName] = createSignal("");
  const [newCommand, setNewCommand] = createSignal("");
  const [newIcon, setNewIcon] = createSignal("📌");
  const [newCategory, setNewCategory] = createSignal("");
  const [newDescription, setNewDescription] = createSignal("");

  onMount(async () => {
    await reload();
  });

  async function reload() {
    try {
      const list = (await invoke("list_snippets")) as Snippet[];
      setSnippets(list);
      const cats = (await invoke("get_snippet_categories")) as string[];
      setCategories(cats);
    } catch (_) {}
  }

  const filtered = createMemo(() => {
    let list = snippets();

    if (selectedCategory() !== "all") {
      list = list.filter(
        (s) => s.category?.toLowerCase() === selectedCategory().toLowerCase()
      );
    }

    const q = search().toLowerCase();
    if (q) {
      list = list.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          s.command.toLowerCase().includes(q) ||
          s.tags.some((t) => t.toLowerCase().includes(q)) ||
          (s.description || "").toLowerCase().includes(q)
      );
    }

    return list;
  });

  async function runSnippet(snippet: Snippet) {
    await invoke("run_snippet", { id: snippet.id, sessionId: props.sessionId });
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
      category: newCategory() || undefined,
      description: newDescription() || undefined,
    };

    await invoke("add_snippet", { snippet });
    setNewName("");
    setNewCommand("");
    setNewCategory("");
    setNewDescription("");
    setShowAdd(false);
    await reload();
  }

  async function deleteSnippet(id: string) {
    await invoke("delete_snippet", { id });
    await reload();
  }

  async function exportSnippets() {
    try {
      const json = (await invoke("export_snippets")) as string;
      await navigator.clipboard.writeText(json);
      alert("Snippets copied to clipboard as JSON!");
    } catch (e) {
      alert("Export failed: " + e);
    }
  }

  async function importSnippets() {
    if (!importJson().trim()) return;
    setImportMsg("");
    try {
      const count = (await invoke("import_snippets", {
        jsonStr: importJson(),
      })) as number;
      setImportMsg(`✅ Imported ${count} snippets`);
      setImportJson("");
      await reload();
      setTimeout(() => {
        setShowImport(false);
        setImportMsg("");
      }, 2000);
    } catch (e: any) {
      setImportMsg("❌ " + e.toString());
    }
  }

  return (
    <div class="palette-overlay" onClick={() => props.onClose()}>
      <div class="snippet-library" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="snippet-header">
          <span>📋 Snippet Library</span>
          <div style={{ display: "flex", gap: "6px" }}>
            <button
              class="snippet-add-btn"
              onClick={() => setShowImport(!showImport())}
              title="Import/Export"
            >
              📥
            </button>
            <button class="snippet-add-btn" onClick={exportSnippets} title="Export">
              📤
            </button>
            <button
              class="snippet-add-btn"
              onClick={() => setShowAdd(!showAdd())}
            >
              {showAdd() ? "Cancel" : "+ New"}
            </button>
          </div>
        </div>

        {/* Import Panel */}
        <Show when={showImport()}>
          <div class="snippet-form">
            <textarea
              class="snippet-form-input full"
              placeholder="Paste JSON array of snippets..."
              value={importJson()}
              onInput={(e) => setImportJson(e.currentTarget.value)}
              rows={4}
              style={{ resize: "vertical", "min-height": "60px" }}
            />
            <Show when={importMsg()}>
              <div style={{ "font-size": "11px", padding: "4px 0" }}>
                {importMsg()}
              </div>
            </Show>
            <button class="snippet-save-btn" onClick={importSnippets}>
              Import Snippets
            </button>
          </div>
        </Show>

        {/* Add Form */}
        <Show when={showAdd()}>
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
            />
            <div class="snippet-form-row">
              <input
                class="snippet-form-input"
                placeholder="Category (optional)"
                value={newCategory()}
                onInput={(e) => setNewCategory(e.currentTarget.value)}
              />
              <input
                class="snippet-form-input"
                placeholder="Description (optional)"
                value={newDescription()}
                onInput={(e) => setNewDescription(e.currentTarget.value)}
              />
            </div>
            <button class="snippet-save-btn" onClick={addSnippet}>
              Save Snippet
            </button>
          </div>
        </Show>

        {/* Category Tabs */}
        <Show when={categories().length > 0}>
          <div class="snippet-categories">
            <button
              class={`snippet-cat-btn ${selectedCategory() === "all" ? "active" : ""}`}
              onClick={() => setSelectedCategory("all")}
            >
              All ({snippets().length})
            </button>
            <For each={categories()}>
              {(cat) => (
                <button
                  class={`snippet-cat-btn ${selectedCategory() === cat ? "active" : ""}`}
                  onClick={() => setSelectedCategory(cat)}
                >
                  {cat}
                </button>
              )}
            </For>
          </div>
        </Show>

        {/* Search */}
        <input
          class="palette-input"
          type="text"
          placeholder="Search snippets..."
          value={search()}
          onInput={(e) => setSearch(e.currentTarget.value)}
          autofocus
        />

        {/* List */}
        <div class="snippet-list">
          <For each={filtered()}>
            {(snippet) => (
              <div class="snippet-item">
                <div class="snippet-item-main" onClick={() => runSnippet(snippet)}>
                  <span class="snippet-item-icon">{snippet.icon}</span>
                  <div class="snippet-item-info">
                    <span class="snippet-item-name">{snippet.name}</span>
                    <code class="snippet-item-cmd">{snippet.command}</code>
                    <Show when={snippet.description}>
                      <span
                        style={{
                          "font-size": "10px",
                          opacity: 0.35,
                          "margin-top": "2px",
                        }}
                      >
                        {snippet.description}
                      </span>
                    </Show>
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
            )}
          </For>
          <Show when={filtered().length === 0}>
            <div class="snippet-empty">No snippets found</div>
          </Show>
        </div>
      </div>
    </div>
  );
}