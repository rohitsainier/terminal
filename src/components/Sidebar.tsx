import { createSignal, onMount, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Tab {
  id: string;
  title: string;
  cwd: string;
}

interface SSHEntry {
  id: string;
  name: string;
  host: string;
  port: number;
  icon?: string;
  group?: string;
}

interface HistoryEntry {
  command: string;
  timestamp: string;
  cwd: string;
}

interface SnippetEntry {
  id: string;
  name: string;
  command: string;
  icon: string;
}

interface Props {
  visible: boolean;
  onClose: () => void;
  activeSessionId: string;
  tabs: Tab[];
  onSnippetRun: (command: string) => void;
  onSSHConnect: (id: string) => void;
  onTabSelect: (id: string) => void;
}

export default function Sidebar(props: Props) {
  const [activeSection, setActiveSection] = createSignal("sessions");

  // Live data
  const [sshConnections, setSSHConnections] = createSignal<SSHEntry[]>([]);
  const [sshStatus, setSSHStatus] = createSignal<Record<string, boolean | null>>({});
  const [history, setHistory] = createSignal<HistoryEntry[]>([]);
  const [snippets, setSnippets] = createSignal<SnippetEntry[]>([]);
  const [knownHosts, setKnownHosts] = createSignal<string[]>([]);
  const [topCommands, setTopCommands] = createSignal<[string, number][]>([]);

  // SSH form
  const [showSSHForm, setShowSSHForm] = createSignal(false);
  const [sshName, setSSHName] = createSignal("");
  const [sshHost, setSSHHost] = createSignal("");
  const [sshUser, setSSHUser] = createSignal("");
  const [sshPort, setSSHPort] = createSignal(22);

  async function loadSectionData(section: string) {
    try {
      if (section === "ssh") {
        const conns = (await invoke("list_ssh_connections")) as SSHEntry[];
        setSSHConnections(conns);
        const hosts = (await invoke("get_known_hosts")) as string[];
        setKnownHosts(hosts);
        for (const conn of conns) {
          checkReachability(conn.id, conn.host, conn.port);
        }
      } else if (section === "history") {
        const hist = (await invoke("recent_history", { limit: 50 })) as HistoryEntry[];
        setHistory(hist);
        const top = (await invoke("most_used_commands", { limit: 10 })) as [string, number][];
        setTopCommands(top);
      } else if (section === "snippets") {
        const snips = (await invoke("list_snippets")) as SnippetEntry[];
        setSnippets(snips);
      }
    } catch (e) {
      console.error("Sidebar load error:", e);
    }
  }

  async function checkReachability(id: string, host: string, port: number) {
    try {
      const reachable = (await invoke("check_ssh_reachable", { host, port })) as boolean;
      setSSHStatus((prev) => ({ ...prev, [id]: reachable }));
    } catch (_) {
      setSSHStatus((prev) => ({ ...prev, [id]: false }));
    }
  }

  function switchSection(section: string) {
    setActiveSection(section);
    loadSectionData(section);
  }

  async function addSSHConnection() {
    if (!sshName() || !sshHost() || !sshUser()) return;
    const conn = {
      id: crypto.randomUUID(),
      name: sshName(),
      host: sshHost(),
      username: sshUser(),
      port: sshPort(),
      auth_method: { Agent: null } as any,
      jump_host: null,
      local_forwards: [],
      remote_forwards: [],
      startup_command: null,
      color: null,
      icon: "🔗",
      group: null,
      last_connected: null,
    };
    try {
      await invoke("add_ssh_connection", { connection: conn });
      setShowSSHForm(false);
      setSSHName("");
      setSSHHost("");
      setSSHUser("");
      loadSectionData("ssh");
    } catch (e) {
      console.error("Add SSH error:", e);
    }
  }

  async function deleteSSH(id: string) {
    await invoke("delete_ssh_connection", { id }).catch(() => {});
    loadSectionData("ssh");
  }

  async function clearAllHistory() {
    await invoke("clear_history").catch(() => {});
    setHistory([]);
    setTopCommands([]);
  }

  // ── THIS IS THE KEY FIX ──
  // Use <Show> instead of early return
  return (
    <Show when={props.visible}>
      <div class="sidebar">
        <div class="sidebar-header">
          <span class="sidebar-title">⚡ Flux</span>
          <span class="sidebar-close" onClick={props.onClose}>×</span>
        </div>

        <div class="sidebar-nav">
          <button
            class={`sidebar-nav-btn ${activeSection() === "sessions" ? "active" : ""}`}
            onClick={() => switchSection("sessions")}
          >
            📂 Sessions
          </button>
          <button
            class={`sidebar-nav-btn ${activeSection() === "ssh" ? "active" : ""}`}
            onClick={() => switchSection("ssh")}
          >
            🔗 SSH
          </button>
          <button
            class={`sidebar-nav-btn ${activeSection() === "snippets" ? "active" : ""}`}
            onClick={() => switchSection("snippets")}
          >
            📋 Snippets
          </button>
          <button
            class={`sidebar-nav-btn ${activeSection() === "history" ? "active" : ""}`}
            onClick={() => switchSection("history")}
          >
            🕐 History
          </button>
        </div>

        <div class="sidebar-content">
          {/* ── Sessions ── */}
          <Show when={activeSection() === "sessions"}>
            <div class="sidebar-section">
              <For each={props.tabs}>
                {(tab) => (
                  <div
                    class={`sidebar-item ${tab.id === props.activeSessionId ? "active" : ""}`}
                    onClick={() => props.onTabSelect(tab.id)}
                  >
                    <span class="sidebar-item-icon">❯</span>
                    <span>{tab.title} — {tab.cwd}</span>
                  </div>
                )}
              </For>
              <Show when={props.tabs.length === 0}>
                <div class="sidebar-empty">No active sessions</div>
              </Show>
            </div>
          </Show>

          {/* ── SSH ── */}
          <Show when={activeSection() === "ssh"}>
            <div class="sidebar-section">
              <div
                class="sidebar-item"
                style={{ opacity: "0.6", cursor: "pointer" }}
                onClick={() => setShowSSHForm(!showSSHForm())}
              >
                <span class="sidebar-item-icon">➕</span>
                <span>{showSSHForm() ? "Cancel" : "Add connection"}</span>
              </div>

              <Show when={showSSHForm()}>
                <div class="sidebar-ssh-form">
                  <input
                    class="sidebar-form-input"
                    placeholder="Name"
                    value={sshName()}
                    onInput={(e) => setSSHName(e.currentTarget.value)}
                  />
                  <input
                    class="sidebar-form-input"
                    placeholder="user@host"
                    value={sshUser() ? `${sshUser()}@${sshHost()}` : ""}
                    onInput={(e) => {
                      const val = e.currentTarget.value;
                      const at = val.indexOf("@");
                      if (at > 0) {
                        setSSHUser(val.slice(0, at));
                        setSSHHost(val.slice(at + 1));
                      } else {
                        setSSHHost(val);
                      }
                    }}
                  />
                  <button class="sidebar-form-btn" onClick={addSSHConnection}>
                    Save
                  </button>
                </div>
              </Show>

              <For each={sshConnections()}>
                {(conn) => (
                  <div class="sidebar-item" style={{ "justify-content": "space-between" }}>
                    <div
                      style={{
                        display: "flex",
                        gap: "8px",
                        "align-items": "center",
                        cursor: "pointer",
                        flex: "1",
                      }}
                      onClick={() => props.onSSHConnect(conn.id)}
                    >
                      <span class="sidebar-item-icon">{conn.icon || "🔗"}</span>
                      <div style={{ display: "flex", "flex-direction": "column" }}>
                        <span>{conn.name}</span>
                        <span
                          class="sidebar-item-mono"
                          style={{ "font-size": "10px", opacity: "0.4" }}
                        >
                          {conn.host}:{conn.port}
                        </span>
                      </div>
                    </div>
                    <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
                      <span
                        style={{
                          width: "6px",
                          height: "6px",
                          "border-radius": "50%",
                          background:
                            sshStatus()[conn.id] === true
                              ? "#4caf50"
                              : sshStatus()[conn.id] === false
                                ? "#ff4444"
                                : "#888",
                        }}
                        title={
                          sshStatus()[conn.id] === true
                            ? "Reachable"
                            : sshStatus()[conn.id] === false
                              ? "Unreachable"
                              : "Checking..."
                        }
                      />
                      <span
                        style={{
                          cursor: "pointer",
                          "font-size": "10px",
                          opacity: "0.4",
                        }}
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteSSH(conn.id);
                        }}
                      >
                        🗑️
                      </span>
                    </div>
                  </div>
                )}
              </For>

              <Show when={sshConnections().length === 0 && !showSSHForm()}>
                <div class="sidebar-empty">
                  No SSH connections.
                  <br />
                  <Show when={knownHosts().length > 0}>
                    <span style={{ "font-size": "10px", opacity: "0.5" }}>
                      {knownHosts().length} hosts in known_hosts
                    </span>
                  </Show>
                </div>
              </Show>
            </div>
          </Show>

          {/* ── Snippets ── */}
          <Show when={activeSection() === "snippets"}>
            <div class="sidebar-section">
              <For each={snippets()}>
                {(snippet) => (
                  <div
                    class="sidebar-item"
                    onClick={() => props.onSnippetRun(snippet.command)}
                  >
                    <span class="sidebar-item-icon">{snippet.icon}</span>
                    <div style={{ display: "flex", "flex-direction": "column" }}>
                      <span>{snippet.name}</span>
                      <span
                        class="sidebar-item-mono"
                        style={{ "font-size": "10px", opacity: "0.4" }}
                      >
                        {snippet.command.length > 30
                          ? snippet.command.slice(0, 30) + "..."
                          : snippet.command}
                      </span>
                    </div>
                  </div>
                )}
              </For>
              <Show when={snippets().length === 0}>
                <div class="sidebar-empty">No snippets saved</div>
              </Show>
            </div>
          </Show>

          {/* ── History ── */}
          <Show when={activeSection() === "history"}>
            <div class="sidebar-section">
              <Show when={topCommands().length > 0}>
                <div style={{ padding: "6px 10px", "font-size": "10px", opacity: "0.4" }}>
                  TOP COMMANDS
                </div>
                <For each={topCommands().slice(0, 5)}>
                  {([cmd, count]) => (
                    <div
                      class="sidebar-item"
                      onClick={() => props.onSnippetRun(cmd)}
                    >
                      <span
                        class="sidebar-item-icon"
                        style={{ "font-size": "9px", opacity: "0.4" }}
                      >
                        ×{count}
                      </span>
                      <span class="sidebar-item-mono">{cmd}</span>
                    </div>
                  )}
                </For>
                <div style={{ "border-bottom": "1px solid var(--border)", margin: "6px 0" }} />
              </Show>

              <div
                style={{
                  display: "flex",
                  "justify-content": "space-between",
                  "align-items": "center",
                  padding: "6px 10px",
                  "font-size": "10px",
                  opacity: "0.4",
                }}
              >
                <span>RECENT</span>
                <Show when={history().length > 0}>
                  <span
                    style={{ cursor: "pointer", color: "var(--accent)" }}
                    onClick={clearAllHistory}
                  >
                    Clear
                  </span>
                </Show>
              </div>

              <For each={history()}>
                {(entry) => (
                  <div
                    class="sidebar-item"
                    onClick={() => props.onSnippetRun(entry.command)}
                  >
                    <span class="sidebar-item-icon">$</span>
                    <span class="sidebar-item-mono">{entry.command}</span>
                  </div>
                )}
              </For>
              <Show when={history().length === 0}>
                <div class="sidebar-empty">No command history yet</div>
              </Show>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
}