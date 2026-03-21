import { createSignal, onMount } from "solid-js";

interface Props {
  nodeId: string;
  deviceName: string;
  onClose: () => void;
}

export default function QRModal(props: Props) {
  const [qrDataUrl, setQrDataUrl] = createSignal<string | null>(null);
  const [copied, setCopied] = createSignal(false);

  onMount(async () => {
    try {
      // Dynamic import to avoid bundling issues
      const QRCode = await import("qrcode");
      const url = await QRCode.toDataURL(props.nodeId, {
        width: 256,
        margin: 2,
        color: {
          dark: "#ffffff",
          light: "#00000000",
        },
        errorCorrectionLevel: "M",
      });
      setQrDataUrl(url);
    } catch (e) {
      console.error("QR code generation failed:", e);
    }
  });

  const copyId = () => {
    navigator.clipboard.writeText(props.nodeId);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleOverlayClick = (e: MouseEvent) => {
    if ((e.target as HTMLElement).classList.contains("blnk-qr-overlay")) {
      props.onClose();
    }
  };

  return (
    <div class="blnk-qr-overlay" onClick={handleOverlayClick}>
      <div class="blnk-qr-modal">
        <div class="blnk-qr-header">
          <span class="blnk-qr-title">SHARE YOUR ID</span>
          <span class="blnk-qr-close" onClick={() => props.onClose()}>
            {"\u2715"}
          </span>
        </div>

        <div class="blnk-qr-body">
          <div class="blnk-qr-device-name">{props.deviceName}</div>

          <div class="blnk-qr-image-wrap">
            {qrDataUrl() ? (
              <img
                src={qrDataUrl()!}
                alt="QR Code"
                class="blnk-qr-image"
              />
            ) : (
              <div class="blnk-qr-loading">Generating QR...</div>
            )}
          </div>

          <div class="blnk-qr-hint">
            Scan this QR code to get the Endpoint ID
          </div>

          <div class="blnk-qr-id" onClick={copyId} title="Click to copy">
            {props.nodeId.slice(0, 16)}...{props.nodeId.slice(-8)}
          </div>

          <button class="blnk-btn blnk-btn-full" onClick={copyId}>
            {copied() ? "COPIED!" : "COPY FULL ID"}
          </button>
        </div>
      </div>
    </div>
  );
}
