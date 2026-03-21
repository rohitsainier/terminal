import { createSignal, onMount, onCleanup, Show } from "solid-js";

interface Tab {
  id: string;
  title: string;
  cwd: string;
}

interface Props {
  activeTab: Tab | undefined;
  theme: string | undefined;
  onShowShortcuts: () => void;
  bharatLinkRunning?: boolean;
  bharatLinkPeerCount?: number;
  onBharatLinkClick?: () => void;
}

export default function StatusBar(props: Props) {
  const [time, setTime] = createSignal("");
  let timer: number;

  onMount(() => {
    function updateTime() {
      const now = new Date();
      setTime(
        now.toLocaleTimeString("en-US", {
          hour12: false,
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
        })
      );
    }
    updateTime();
    timer = window.setInterval(updateTime, 1000);
  });

  onCleanup(() => clearInterval(timer));

  const os = navigator.platform.includes("Mac")
    ? "macOS"
    : navigator.platform.includes("Win")
      ? "Windows"
      : "Linux";

  return (
    <div class="status-bar">
      <div class="status-bar-left">
        <span class="status-indicator">⚡</span>
        <span>{props.activeTab?.cwd || "~"}</span>
        <span class="status-separator">│</span>
        <span>zsh</span>
      </div>
      <div class="status-bar-right">
        <span
          class="status-bharatlink"
          onClick={() => props.onBharatLinkClick?.()}
          title={props.bharatLinkRunning ? `BharatLink Online · ${props.bharatLinkPeerCount || 0} peers · Click to open (⌘⇧B)` : "BharatLink Offline · Click to open (⌘⇧B)"}
        >
          <span
            class="status-bharatlink-dot"
            classList={{ "status-bharatlink-dot-active": props.bharatLinkRunning }}
          />
          <span class="status-bharatlink-label">BharatLink</span>
          <Show when={props.bharatLinkRunning && (props.bharatLinkPeerCount || 0) > 0}>
            <span class="status-bharatlink-peers">{props.bharatLinkPeerCount}</span>
          </Show>
        </span>
        <span class="status-separator">│</span>
        <span class="status-theme">🎨 {props.theme || "hacker-green"}</span>
        <span class="status-separator">│</span>
        <span>{os}</span>
        <span class="status-separator">│</span>
        <span>{time()}</span>
        <span class="status-separator">│</span>
        <span
          class="status-info-btn"
          onClick={() => props.onShowShortcuts()}
          title="Keyboard Shortcuts"
        >
          ℹ️
        </span>
      </div>
    </div>
  );
}