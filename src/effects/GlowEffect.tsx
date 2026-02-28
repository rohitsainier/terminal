import { onMount, onCleanup } from "solid-js";

interface Props {
  color: string;
  intensity: number;
  enabled: boolean;
}

export default function GlowEffect(props: Props) {
  onMount(() => {
    if (!props.enabled) return;

    const root = document.documentElement;
    root.style.setProperty("--glow-color", props.color);
    root.style.setProperty("--glow-intensity", props.intensity.toString());
    document.body.classList.add("glow-enabled");
  });

  onCleanup(() => {
    document.body.classList.remove("glow-enabled");
  });

  return null;
}