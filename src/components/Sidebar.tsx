import { createSignal } from "solid-js";

interface Props {
  visible: boolean;
  onClose: () => void;
}

export default function Sidebar(props: Props) {
  const [activeSection, setActiveSection] = createSignal("sessions");

  if (!props.visible) return null;

  return (
    <div class="sidebar">
      <div class="sidebar-header">
        <span class="sidebar-title">⚡ Flux</span>
        <span class="sidebar-close" onClick={props.onClose}>
          ×
        </span>
      </div>

      <div class="sidebar-nav">
        <button
          class={`sidebar-nav-btn ${
            activeSection() === "sessions" ? "active" : ""
          }`}
          onClick={() => setActiveSection("sessions")}
        >
          📂 Sessions
        </button>
        <button
          class={`sidebar-nav-btn ${
            activeSection() === "ssh" ? "active" : ""
          }`}
          onClick={() => setActiveSection("ssh")}
        >
          🔗 SSH
        </button>
        <button
          class={`sidebar-nav-btn ${
            activeSection() === "snippets" ? "active" : ""
          }`}
          onClick={() => setActiveSection("snippets")}
        >
          📋 Snippets
        </button>
        <button
          class={`sidebar-nav-btn ${
            activeSection() === "history" ? "active" : ""
          }`}
          onClick={() => setActiveSection("history")}
        >
          🕐 History
        </button>
      </div>

      <div class="sidebar-content">
        {activeSection() === "sessions" && (
          <div class="sidebar-section">
            <div class="sidebar-item active">
              <span class="sidebar-item-icon">❯</span>
              <span>~/Projects/flux</span>
            </div>
            <div class="sidebar-item">
              <span class="sidebar-item-icon">❯</span>
              <span>~/Documents</span>
            </div>
          </div>
        )}

        {activeSection() === "ssh" && (
          <div class="sidebar-section">
            <div class="sidebar-empty">
              No SSH connections saved.
              <br />
              <span class="sidebar-link">+ Add connection</span>
            </div>
          </div>
        )}

        {activeSection() === "snippets" && (
          <div class="sidebar-section">
            <div class="sidebar-item">
              <span class="sidebar-item-icon">🐳</span>
              <span>Docker Cleanup</span>
            </div>
            <div class="sidebar-item">
              <span class="sidebar-item-icon">💀</span>
              <span>Kill Port</span>
            </div>
            <div class="sidebar-item">
              <span class="sidebar-item-icon">⏪</span>
              <span>Git Undo Commit</span>
            </div>
            <div class="sidebar-item">
              <span class="sidebar-item-icon">🔍</span>
              <span>Find Large Files</span>
            </div>
          </div>
        )}

        {activeSection() === "history" && (
          <div class="sidebar-section">
            <div class="sidebar-item">
              <span class="sidebar-item-icon">$</span>
              <span class="sidebar-item-mono">git status</span>
            </div>
            <div class="sidebar-item">
              <span class="sidebar-item-icon">$</span>
              <span class="sidebar-item-mono">npm run dev</span>
            </div>
            <div class="sidebar-item">
              <span class="sidebar-item-icon">$</span>
              <span class="sidebar-item-mono">docker ps</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}