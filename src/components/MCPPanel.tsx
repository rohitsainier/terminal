import { createSignal, Show, For, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface MCPServerConfig {
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
  description?: string;
}

interface MCPTool {
  name: string;
  description: string;
  inputSchema: any;
}

interface MCPServerInfo {
  name: string;
  connected: boolean;
  tools_count: number;
  tools: MCPTool[];
  resources: any[];
  server_name?: string;
  server_version?: string;
  error?: string;
  config: MCPServerConfig;
}

interface MCPContent {
  content_type: string;
  text?: string;
}

interface MCPToolResult {
  content: MCPContent[];
  is_error: boolean;
}

interface Props {
  onClose: () => void;
}

export default function MCPPanel(props: Props) {
  const [activeTab, setActiveTab] = createSignal<"servers" | "tools" | "config">("servers");
  const [servers, setServers] = createSignal<MCPServerInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  // Add server form
  const [showAddForm, setShowAddForm] = createSignal(false);
  const [newName, setNewName] = createSignal("");
  const [newCommand, setNewCommand] = createSignal("");
  const [newArgs, setNewArgs] = createSignal("");
  const [newEnv, setNewEnv] = createSignal("");
  const [newDesc, setNewDesc] = createSignal("");

  // Tool execution
  const [selectedTool, setSelectedTool] = createSignal<{ server: string; tool: MCPTool } | null>(null);
  const [toolArgs, setToolArgs] = createSignal("{}");
  const [toolResult, setToolResult] = createSignal<MCPToolResult | null>(null);
  const [toolLoading, setToolLoading] = createSignal(false);
  const [toolError, setToolError] = createSignal("");

  // Raw config editor
  const [rawConfig, setRawConfig] = createSignal("");
  const [configSaved, setConfigSaved] = createSignal(false);

  onMount(() => {
    loadServers();
  });

  async function loadServers() {
    setLoading(true);
    setError("");
    try {
      const list = (await invoke("mcp_list_servers")) as MCPServerInfo[];
      setServers(Array.isArray(list) ? list : []);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function startServer(name: string) {
    setError("");
    try {
      await invoke("mcp_start_server", { name });
      await loadServers();
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function stopServer(name: string) {
    setError("");
    try {
      await invoke("mcp_stop_server", { name });
      await loadServers();
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function restartServer(name: string) {
    setError("");
    try {
      await invoke("mcp_restart_server", { name });
      await loadServers();
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function removeServer(name: string) {
    if (!confirm(`Remove MCP server "${name}"?`)) return;
    try {
      await invoke("mcp_remove_server", { name });
      await loadServers();
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function addServer() {
    if (!newName() || !newCommand()) return;

    const args = newArgs()
      .split(/\s+/)
      .filter((a) => a.trim());

    let env: Record<string, string> = {};
    if (newEnv().trim()) {
      try {
        env = JSON.parse(newEnv());
      } catch {
        // Try KEY=VALUE format
        for (const line of newEnv().split("\n")) {
          const eq = line.indexOf("=");
          if (eq > 0) {
            env[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
          }
        }
      }
    }

    const config: MCPServerConfig = {
      command: newCommand(),
      args,
      env,
      enabled: true,
      description: newDesc() || undefined,
    };

    try {
      await invoke("mcp_add_server", { name: newName(), config });
      setShowAddForm(false);
      setNewName("");
      setNewCommand("");
      setNewArgs("");
      setNewEnv("");
      setNewDesc("");
      await loadServers();
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function callTool() {
    const sel = selectedTool();
    if (!sel) return;

    setToolLoading(true);
    setToolError("");
    setToolResult(null);

    try {
      let args: any = {};
      const raw = toolArgs().trim();
      if (raw) {
        args = JSON.parse(raw);
      }

      const result = (await invoke("mcp_call_tool", {
        server: sel.server,
        tool: sel.tool.name,
        arguments: args,
      })) as MCPToolResult;

      setToolResult(result);
    } catch (e: any) {
      setToolError(String(e));
    } finally {
      setToolLoading(false);
    }
  }

  async function loadRawConfig() {
    try {
      const config = await invoke("mcp_get_config");
      setRawConfig(JSON.stringify(config, null, 2));
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function saveRawConfig() {
    try {
      const config = JSON.parse(rawConfig());
      await invoke("mcp_save_config", { config });
      setConfigSaved(true);
      setTimeout(() => setConfigSaved(false), 2000);
      await loadServers();
    } catch (e: any) {
      setError("Invalid JSON: " + String(e));
    }
  }

  // Generate placeholder args from schema
  function schemaToPlaceholder(schema: any): string {
    if (!schema || !schema.properties) return "{}";
    const obj: any = {};
    for (const [key, prop] of Object.entries(schema.properties as Record<string, any>)) {
      const required = schema.required?.includes(key);
      if (prop.type === "string") obj[key] = required ? "" : undefined;
      else if (prop.type === "number" || prop.type === "integer") obj[key] = 0;
      else if (prop.type === "boolean") obj[key] = false;
      else if (prop.type === "array") obj[key] = [];
      else obj[key] = null;
    }
    // Remove undefined
    const clean = Object.fromEntries(
      Object.entries(obj).filter(([, v]) => v !== undefined)
    );
    return JSON.stringify(clean, null, 2);
  }

  // All tools across servers
  function allTools(): { server: string; tool: MCPTool }[] {
    const result: { server: string; tool: MCPTool }[] = [];
    for (const s of servers()) {
      if (!s.connected) continue;
      for (const t of s.tools) {
        result.push({ server: s.name, tool: t });
      }
    }
    return result;
  }

  return (
    <div class="settings-overlay" onClick={() => props.onClose()}>
      <div
        class="mcp-panel"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div class="settings-header">
          <div style={{ display: "flex", "align-items": "center", gap: "8px" }}>
            <span>🔌</span>
            <span>MCP Servers</span>
            <span class="mcp-badge">
              {servers().filter((s) => s.connected).length} connected
            </span>
          </div>
          <div class="settings-header-right">
            <button
              class="settings-save-btn"
              style={{ "font-size": "11px", padding: "4px 10px" }}
              onClick={loadServers}
            >
              🔄 Refresh
            </button>
            <span class="settings-close" onClick={() => props.onClose()}>×</span>
          </div>
        </div>

        {/* Tabs */}
        <div class="settings-tabs">
          <button
            class={`settings-tab ${activeTab() === "servers" ? "active" : ""}`}
            onClick={() => setActiveTab("servers")}
          >
            🖥️ Servers
          </button>
          <button
            class={`settings-tab ${activeTab() === "tools" ? "active" : ""}`}
            onClick={() => setActiveTab("tools")}
          >
            🛠️ Tools ({allTools().length})
          </button>
          <button
            class={`settings-tab ${activeTab() === "config" ? "active" : ""}`}
            onClick={() => {
              setActiveTab("config");
              loadRawConfig();
            }}
          >
            📝 JSON Config
          </button>
        </div>

        {/* Error */}
        <Show when={error()}>
          <div class="mcp-error">
            ⚠️ {error()}
            <span
              class="mcp-error-close"
              onClick={() => setError("")}
            >
              ×
            </span>
          </div>
        </Show>

        <div class="settings-content">
          {/* ── Servers Tab ── */}
          <Show when={activeTab() === "servers"}>
            <div class="settings-section">
              <div style={{ display: "flex", "justify-content": "space-between", "align-items": "center" }}>
                <h4>Configured Servers</h4>
                <button
                  class="snippet-add-btn"
                  onClick={() => setShowAddForm(!showAddForm())}
                >
                  {showAddForm() ? "Cancel" : "+ Add Server"}
                </button>
              </div>

              {/* Add form */}
              <Show when={showAddForm()}>
                <div class="mcp-add-form">
                  <input
                    class="settings-input"
                    placeholder="Server name (e.g. filesystem)"
                    value={newName()}
                    onInput={(e) => setNewName(e.currentTarget.value)}
                  />
                  <input
                    class="settings-input"
                    placeholder="Command (e.g. npx)"
                    value={newCommand()}
                    onInput={(e) => setNewCommand(e.currentTarget.value)}
                  />
                  <input
                    class="settings-input"
                    placeholder="Arguments (space-separated)"
                    value={newArgs()}
                    onInput={(e) => setNewArgs(e.currentTarget.value)}
                  />
                  <textarea
                    class="settings-input"
                    placeholder='Environment (JSON or KEY=VALUE per line)'
                    value={newEnv()}
                    onInput={(e) => setNewEnv(e.currentTarget.value)}
                    rows={2}
                    style={{ resize: "vertical", "min-height": "36px" }}
                  />
                  <input
                    class="settings-input"
                    placeholder="Description (optional)"
                    value={newDesc()}
                    onInput={(e) => setNewDesc(e.currentTarget.value)}
                  />
                  <button class="settings-save-btn" onClick={addServer}>
                    Add Server
                  </button>
                </div>
              </Show>

              {/* Server list */}
              <Show when={loading()}>
                <div class="mcp-loading">Loading servers...</div>
              </Show>

              <For each={servers()}>
                {(server) => (
                  <div class="mcp-server-card">
                    <div class="mcp-server-header">
                      <div class="mcp-server-info">
                        <div class="mcp-server-name">
                          <span
                            class="mcp-status-dot"
                            style={{
                              background: server.connected ? "#4caf50" : "#666",
                            }}
                          />
                          <span>{server.name}</span>
                          <Show when={server.server_name}>
                            <span class="mcp-server-version">
                              {server.server_name} {server.server_version || ""}
                            </span>
                          </Show>
                        </div>
                        <div class="mcp-server-meta">
                          <code>
                            {server.config.command} {server.config.args.join(" ")}
                          </code>
                        </div>
                        <Show when={server.config.description}>
                          <div class="mcp-server-desc">{server.config.description}</div>
                        </Show>
                      </div>

                      <div class="mcp-server-actions">
                        <Show when={!server.connected}>
                          <button
                            class="mcp-btn mcp-btn-start"
                            onClick={() => startServer(server.name)}
                          >
                            ▶ Start
                          </button>
                        </Show>
                        <Show when={server.connected}>
                          <button
                            class="mcp-btn mcp-btn-restart"
                            onClick={() => restartServer(server.name)}
                          >
                            🔄
                          </button>
                          <button
                            class="mcp-btn mcp-btn-stop"
                            onClick={() => stopServer(server.name)}
                          >
                            ⏹
                          </button>
                        </Show>
                        <button
                          class="mcp-btn mcp-btn-delete"
                          onClick={() => removeServer(server.name)}
                        >
                          🗑️
                        </button>
                      </div>
                    </div>

                    {/* Tools preview */}
                    <Show when={server.connected && server.tools.length > 0}>
                      <div class="mcp-tools-preview">
                        <span class="mcp-tools-label">
                          {server.tools.length} tools:
                        </span>
                        <For each={server.tools.slice(0, 6)}>
                          {(tool) => (
                            <span
                              class="mcp-tool-chip"
                              title={tool.description}
                              onClick={() => {
                                setSelectedTool({ server: server.name, tool });
                                setToolArgs(schemaToPlaceholder(tool.inputSchema));
                                setToolResult(null);
                                setToolError("");
                                setActiveTab("tools");
                              }}
                            >
                              {tool.name}
                            </span>
                          )}
                        </For>
                        <Show when={server.tools.length > 6}>
                          <span class="mcp-tool-chip" style={{ opacity: "0.4" }}>
                            +{server.tools.length - 6} more
                          </span>
                        </Show>
                      </div>
                    </Show>

                    <Show when={server.error}>
                      <div class="mcp-server-error">⚠️ {server.error}</div>
                    </Show>
                  </div>
                )}
              </For>

              <Show when={servers().length === 0 && !loading()}>
                <div class="mcp-empty">
                  <p>No MCP servers configured.</p>
                  <p class="mcp-empty-hint">
                    Add a server above, or edit the JSON config at:
                    <br />
                    <code>~/.config/flux-terminal/mcp.json</code>
                  </p>
                  <div class="mcp-example">
                    <p>Example config:</p>
                    <pre>{`{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {}
    }
  }
}`}</pre>
                  </div>
                </div>
              </Show>
            </div>
          </Show>

          {/* ── Tools Tab ── */}
          <Show when={activeTab() === "tools"}>
            <div class="settings-section">
              <h4>Available Tools</h4>

              <Show when={allTools().length === 0}>
                <div class="mcp-empty">
                  No tools available. Start an MCP server first.
                </div>
              </Show>

              <div class="mcp-tools-list">
                <For each={allTools()}>
                  {(item) => (
                    <div
                      class={`mcp-tool-item ${
                        selectedTool()?.tool.name === item.tool.name &&
                        selectedTool()?.server === item.server
                          ? "active"
                          : ""
                      }`}
                      onClick={() => {
                        setSelectedTool(item);
                        setToolArgs(schemaToPlaceholder(item.tool.inputSchema));
                        setToolResult(null);
                        setToolError("");
                      }}
                    >
                      <div class="mcp-tool-item-header">
                        <span class="mcp-tool-name">
                          🛠️ {item.tool.name}
                        </span>
                        <span class="mcp-tool-server">{item.server}</span>
                      </div>
                      <Show when={item.tool.description}>
                        <div class="mcp-tool-desc">{item.tool.description}</div>
                      </Show>
                    </div>
                  )}
                </For>
              </div>

              {/* Tool executor */}
              <Show when={selectedTool()}>
                <div class="mcp-tool-executor">
                  <div class="mcp-executor-header">
                    <span>
                      Execute: <strong>{selectedTool()!.server}/{selectedTool()!.tool.name}</strong>
                    </span>
                  </div>

                  {/* Schema info */}
                  <Show when={selectedTool()!.tool.inputSchema?.properties}>
                    <div class="mcp-schema-info">
                      <span class="mcp-schema-label">Parameters:</span>
                      <For
                        each={Object.entries(
                          (selectedTool()!.tool.inputSchema?.properties || {}) as Record<string, any>
                        )}
                      >
                        {([key, prop]) => (
                          <div class="mcp-schema-param">
                            <code>{key}</code>
                            <span class="mcp-schema-type">
                              {(prop as any).type || "any"}
                            </span>
                            <Show
                              when={
                                selectedTool()!.tool.inputSchema?.required?.includes(key)
                              }
                            >
                              <span class="mcp-schema-required">required</span>
                            </Show>
                            <Show when={(prop as any).description}>
                              <span class="mcp-schema-desc">
                                — {(prop as any).description}
                              </span>
                            </Show>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>

                  <textarea
                    class="mcp-args-input"
                    value={toolArgs()}
                    onInput={(e) => setToolArgs(e.currentTarget.value)}
                    rows={5}
                    placeholder='{"key": "value"}'
                    spellcheck={false}
                  />

                  <div style={{ display: "flex", gap: "8px" }}>
                    <button
                      class="settings-save-btn"
                      onClick={callTool}
                      disabled={toolLoading()}
                    >
                      {toolLoading() ? "⏳ Running..." : "▶ Execute Tool"}
                    </button>
                    <button
                      class="mcp-btn"
                      onClick={() => {
                        setSelectedTool(null);
                        setToolResult(null);
                        setToolError("");
                      }}
                    >
                      Clear
                    </button>
                  </div>

                  {/* Result */}
                  <Show when={toolError()}>
                    <div class="mcp-tool-result error">
                      <strong>Error:</strong>
                      <pre>{toolError()}</pre>
                    </div>
                  </Show>

                  <Show when={toolResult()}>
                    <div
                      class={`mcp-tool-result ${toolResult()!.is_error ? "error" : "success"}`}
                    >
                      <div class="mcp-result-header">
                        <span>
                          {toolResult()!.is_error ? "❌ Error" : "✅ Success"}
                        </span>
                        <button
                          class="mcp-btn"
                          onClick={() => {
                            const text = toolResult()!
                              .content.map((c) => c.text || "")
                              .join("\n");
                            navigator.clipboard.writeText(text);
                          }}
                        >
                          📋 Copy
                        </button>
                      </div>
                      <For each={toolResult()!.content}>
                        {(content) => (
                          <pre class="mcp-result-content">
                            {content.text || JSON.stringify(content, null, 2)}
                          </pre>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              </Show>
            </div>
          </Show>

          {/* ── JSON Config Tab ── */}
          <Show when={activeTab() === "config"}>
            <div class="settings-section">
              <div style={{ display: "flex", "justify-content": "space-between", "align-items": "center", "margin-bottom": "10px" }}>
                <h4>mcp.json</h4>
                <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
                  <Show when={configSaved()}>
                    <span class="settings-saved">✅ Saved!</span>
                  </Show>
                  <button class="settings-save-btn" onClick={saveRawConfig}>
                    Save & Apply
                  </button>
                </div>
              </div>

              <p class="settings-hint">
                Config file: <code>~/.config/flux-terminal/mcp.json</code>
                <br />
                Edit the JSON directly or use the Servers tab to add servers.
              </p>

              <textarea
                class="mcp-config-editor"
                value={rawConfig()}
                onInput={(e) => setRawConfig(e.currentTarget.value)}
                rows={18}
                spellcheck={false}
              />
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}