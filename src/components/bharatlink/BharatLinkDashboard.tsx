import { onMount, onCleanup } from "solid-js";
import { useBharatLinkData } from "./useBharatLinkData";
import TopBar from "./TopBar";
import PeerPanel from "./PeerPanel";
import TransferPanel from "./TransferPanel";
import InfoPanel from "./InfoPanel";

interface Props {
  onClose: () => void;
}

export default function BharatLinkDashboard(props: Props) {
  const store = useBharatLinkData();

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      props.onClose();
    }
  };

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
    store.refreshHistory();
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handleKeyDown);
  });

  return (
    <div class="blnk-overlay" onClick={() => props.onClose()}>
      <div class="blnk-dashboard" onClick={(e) => e.stopPropagation()}>
        {/* Scanline + vignette effects */}
        <div class="blnk-scanlines" />
        <div class="blnk-vignette" />

        <TopBar store={store} onClose={props.onClose} />

        <div class="blnk-main">
          <PeerPanel store={store} />
          <TransferPanel store={store} />
          <InfoPanel store={store} />
        </div>

        <div class="blnk-footer">
          <div class="blnk-footer-left">
            <span
              class="blnk-footer-dot"
              classList={{ "blnk-footer-dot-active": store.isRunning() }}
            />
            <span>{store.statusText()}</span>
          </div>
          <span class="blnk-footer-right">
            BHARATLINK v1.0 · QUIC+mDNS · E2E Encrypted
          </span>
        </div>
      </div>
    </div>
  );
}
