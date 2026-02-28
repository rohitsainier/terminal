import { onMount, onCleanup } from "solid-js";

interface Props {
  color: string;
  enabled: boolean;
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  size: number;
}

export default function ParticleEngine(props: Props) {
  let canvasRef: HTMLCanvasElement | undefined;
  let animationId: number;
  let particles: Particle[] = [];

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

    // Spawn particles on keypress
    function handleKeyDown(e: KeyboardEvent) {
      if (e.metaKey || e.ctrlKey || e.altKey) return;

      // Get approximate cursor position (center of screen as fallback)
      const x = window.innerWidth / 2 + (Math.random() - 0.5) * 200;
      const y = window.innerHeight / 2 + (Math.random() - 0.5) * 100;

      for (let i = 0; i < 3; i++) {
        particles.push({
          x,
          y,
          vx: (Math.random() - 0.5) * 4,
          vy: (Math.random() - 0.5) * 4 - 2,
          life: 0,
          maxLife: 20 + Math.random() * 20,
          size: 1 + Math.random() * 2,
        });
      }
    }

    document.addEventListener("keydown", handleKeyDown);

    function draw() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      particles = particles.filter((p) => p.life < p.maxLife);

      for (const p of particles) {
        p.x += p.vx;
        p.y += p.vy;
        p.vy += 0.1; // gravity
        p.life++;

        const alpha = 1 - p.life / p.maxLife;
        ctx.globalAlpha = alpha;
        ctx.fillStyle = props.color;
        ctx.shadowColor = props.color;
        ctx.shadowBlur = 6;

        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fill();
      }

      ctx.globalAlpha = 1;
      ctx.shadowBlur = 0;

      animationId = requestAnimationFrame(draw);
    }

    draw();

    onCleanup(() => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("resize", resize);
      document.removeEventListener("keydown", handleKeyDown);
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
        "z-index": "995",
      }}
    />
  );
}