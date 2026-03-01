import { createEffect, onCleanup } from "solid-js";

interface Props {
  color: string;
  intensity: number;
  enabled: boolean;
}

export default function GlowEffect(props: Props) {
  createEffect(() => {
    const root = document.documentElement;
    if (props.enabled) {
      root.style.setProperty("--glow-color", props.color);
      root.style.setProperty("--glow-intensity", props.intensity.toString());
      document.body.classList.add("glow-enabled");
    } else {
      document.body.classList.remove("glow-enabled");
    }
  });

  onCleanup(() => {
    document.body.classList.remove("glow-enabled");
  });

  return null;
}