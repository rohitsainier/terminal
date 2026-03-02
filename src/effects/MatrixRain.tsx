import { onMount, onCleanup } from "solid-js";

interface Props {
  color: string;
  opacity: number;
}

export default function MatrixRain(props: Props) {
  let canvasRef: HTMLCanvasElement | undefined;
  let animationId: number;

  onMount(() => {
    const canvas = canvasRef!;
    const ctx = canvas.getContext("2d")!;

    function resize() {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    }
    resize();
    window.addEventListener("resize", resize);

    const fontSize = 14;
    const columns = Math.floor(canvas.width / fontSize);
    const drops: number[] = new Array(columns).fill(1);

    const chars =
      "ॐअआइईउऊऋएऐओऔकखगघचछजझटठडढणतथदधनपफबभमयरलवशषसहयोगधर्मध्यानशक्तिॐ☸🕉️";

    function draw() {
      ctx.fillStyle = `rgba(0, 0, 0, 0.05)`;
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      ctx.fillStyle = props.color;
      ctx.font = fontSize + "px monospace";

      for (let i = 0; i < drops.length; i++) {
        const char = chars[Math.floor(Math.random() * chars.length)];
        ctx.globalAlpha = 0.3 + Math.random() * 0.7;
        ctx.fillText(char, i * fontSize, drops[i] * fontSize);

        if (drops[i] * fontSize > canvas.height && Math.random() > 0.975) {
          drops[i] = 0;
        }
        drops[i]++;
      }

      ctx.globalAlpha = 1;
      animationId = requestAnimationFrame(draw);
    }

    draw();

    onCleanup(() => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("resize", resize);
    });
  });

  return (
    <canvas
      ref={canvasRef}
      class="matrix-rain"
      style={{
        position: "fixed",
        top: "0",
        left: "0",
        width: "100%",
        height: "100%",
        "pointer-events": "none",
        "z-index": "0",
        opacity: props.opacity.toString(),
      }}
    />
  );
}