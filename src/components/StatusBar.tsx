import { createSignal, onMount, onCleanup } from "solid-js";

interface Tab {
  id: string;
  title: string;
  cwd: string;
}

interface Props {
  activeTab: Tab | undefined;
  theme: string | undefined;
  onShowShortcuts: () => void;
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