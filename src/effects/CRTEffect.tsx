interface Props {
  enabled: boolean;
  curvature: boolean;
  flicker: boolean;
}

export default function CRTEffect(props: Props) {
  if (!props.enabled) return null;

  return (
    <>
      {/* Scanlines */}
      <div
        class="crt-scanlines"
        style={{
          position: "fixed",
          inset: "0",
          "pointer-events": "none",
          "z-index": "998",
          background: `repeating-linear-gradient(
            0deg,
            rgba(0, 0, 0, 0.03) 0px,
            rgba(0, 0, 0, 0.03) 1px,
            transparent 1px,
            transparent 2px
          )`,
        }}
      />

      {/* Screen curvature vignette */}
      {props.curvature && (
        <div
          class="crt-curvature"
          style={{
            position: "fixed",
            inset: "0",
            "pointer-events": "none",
            "z-index": "997",
            "box-shadow": "inset 0 0 120px rgba(0, 0, 0, 0.4)",
            "border-radius": "12px",
          }}
        />
      )}

      {/* Flicker */}
      {props.flicker && (
        <div
          class="crt-flicker"
          style={{
            position: "fixed",
            inset: "0",
            "pointer-events": "none",
            "z-index": "996",
            opacity: "0",
            background: "rgba(255, 255, 255, 0.02)",
            animation: "crt-flicker 0.15s infinite",
          }}
        />
      )}
    </>
  );
}