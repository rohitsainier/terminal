import { onMount, onCleanup } from "solid-js";
import { useNetopsData } from "./useNetopsData";
import TopBar from "./TopBar";
import ToolPanel from "./ToolPanel";
import ResultPanel from "./ResultPanel";
import InfoPanel from "./InfoPanel";

interface Props {
  onClose: () => void;
}

export default function NetopsDashboard(props: Props) {
  const store = useNetopsData();

  onMount(() => {
    store.setUtc(new Date().toISOString().slice(11, 19));
    const clockTimer = setInterval(
      () => store.setUtc(new Date().toISOString().slice(11, 19)),
      1000
    );

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        props.onClose();
      }
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        store.runTool();
      }
    };
    window.addEventListener("keydown", onKey);

    onCleanup(() => {
      clearInterval(clockTimer);
      window.removeEventListener("keydown", onKey);
    });
  });

  return (
    <div class="nops-overlay" onClick={() => props.onClose()}>
      <div class="nops-dashboard" onClick={(e) => e.stopPropagation()}>
        <div class="nops-scanlines" />
        <div class="nops-vignette" />

        <TopBar store={store} onClose={props.onClose} />

        <main class="nops-main">
          <ToolPanel store={store} />
          <ResultPanel store={store} />
          <InfoPanel store={store} />
        </main>

        <footer class="nops-statusbar">
          <span
            class="nops-status-dot"
            data-status={store.loading() ? "scanning" : store.error() ? "error" : store.result() ? "complete" : "ready"}
          />
          <span>STATUS: {store.statusText()}</span>
          <span class="nops-status-sep">◈</span>
          <span>⌘+Enter to run</span>
          <span class="nops-status-sep">◈</span>
          <span>Escape to close</span>
        </footer>
      </div>
    </div>
  );
}
