import { onMount, onCleanup } from "solid-js";

interface Props {
  color: string;
  enabled: boolean;
}

export default function HologramEffect(props: Props) {
  let canvasRef: HTMLCanvasElement | undefined;
  let animationId: number;

  onMount(() => {
    if (!props.enabled) return;

    const canvas = canvasRef!;
    const ctx = canvas.getContext("2d")!;

    function resize() {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    }
    resize();
    window.addEventListener("resize", resize);

    let sweepY = 0;
    const sweepSpeed = 1.5;
    let time = 0;

    function hexToRgb(hex: string): [number, number, number] {
      const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
      return result
        ? [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)]
        : [0, 255, 65];
    }

    function draw() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const [r, g, b] = hexToRgb(props.color);
      time += 0.02;

      // ── Main sweep line ──
      sweepY += sweepSpeed;
      if (sweepY > canvas.height + 60) sweepY = -60;

      const gradient = ctx.createLinearGradient(0, sweepY - 30, 0, sweepY + 30);
      gradient.addColorStop(0, `rgba(${r}, ${g}, ${b}, 0)`);
      gradient.addColorStop(0.4, `rgba(${r}, ${g}, ${b}, 0.04)`);
      gradient.addColorStop(0.5, `rgba(${r}, ${g}, ${b}, 0.08)`);
      gradient.addColorStop(0.6, `rgba(${r}, ${g}, ${b}, 0.04)`);
      gradient.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0)`);

      ctx.fillStyle = gradient;
      ctx.fillRect(0, sweepY - 30, canvas.width, 60);

      // ── Horizontal refraction lines ──
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, 0.02)`;
      ctx.lineWidth = 1;
      for (let i = 0; i < 8; i++) {
        const y = (sweepY + i * 4 - 16 + canvas.height) % canvas.height;
        ctx.beginPath();
        ctx.moveTo(0, y);
        // Subtle wave
        for (let x = 0; x < canvas.width; x += 20) {
          ctx.lineTo(x, y + Math.sin(x * 0.01 + time + i) * 1.5);
        }
        ctx.stroke();
      }

      // ── Edge vignette shimmer ──
      const shimmer = Math.sin(time * 0.5) * 0.5 + 0.5;
      const edgeGrad = ctx.createRadialGradient(
        canvas.width / 2,
        canvas.height / 2,
        canvas.height * 0.3,
        canvas.width / 2,
        canvas.height / 2,
        canvas.height * 0.8
      );
      edgeGrad.addColorStop(0, "transparent");
      edgeGrad.addColorStop(1, `rgba(${r}, ${g}, ${b}, ${0.01 * shimmer})`);
      ctx.fillStyle = edgeGrad;
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // ── Sparse grid lines (holographic grid) ──
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, 0.008)`;
      ctx.lineWidth = 1;
      const gridSize = 80;
      const offsetY = (time * 10) % gridSize;
      for (let y = -gridSize + offsetY; y < canvas.height; y += gridSize) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(canvas.width, y);
        ctx.stroke();
      }

      animationId = requestAnimationFrame(draw);
    }

    draw();

    onCleanup(() => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("resize", resize);
    });
  });

  if (!props.enabled) return null;

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "fixed",
        inset: "0",
        "pointer-events": "none",
        "z-index": "994",
        opacity: "0.6",
      }}
    />
  );
}