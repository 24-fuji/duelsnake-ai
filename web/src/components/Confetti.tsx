import { useEffect, useRef } from "react";

const COLORS = ["#fdd835", "#4fc3f7", "#ffb74d", "#ef5350", "#81c784", "#ba68c8"];
const COUNT = 160;
const DURATION_MS = 4000;

interface Piece {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  angle: number;
  spin: number;
  color: string;
}

/** 画面全体に紙吹雪を降らせる。出して数秒で止まり、操作の邪魔にならない */
export function Confetti() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    const width = (canvas.width = window.innerWidth);
    const height = (canvas.height = window.innerHeight);
    const pieces: Piece[] = Array.from({ length: COUNT }, () => ({
      x: Math.random() * width,
      y: -Math.random() * height * 0.5,
      vx: (Math.random() - 0.5) * 2,
      vy: 2 + Math.random() * 3,
      size: 6 + Math.random() * 6,
      angle: Math.random() * Math.PI * 2,
      spin: (Math.random() - 0.5) * 0.3,
      color: COLORS[Math.floor(Math.random() * COLORS.length)],
    }));
    const start = performance.now();
    let frameId = 0;
    const draw = (now: number) => {
      const elapsed = now - start;
      ctx.clearRect(0, 0, width, height);
      ctx.globalAlpha = Math.min(1, (DURATION_MS - elapsed) / 1000);
      for (const p of pieces) {
        p.x += p.vx;
        p.y += p.vy;
        p.angle += p.spin;
        ctx.save();
        ctx.translate(p.x, p.y);
        ctx.rotate(p.angle);
        ctx.fillStyle = p.color;
        ctx.fillRect(-p.size / 2, -p.size / 4, p.size, p.size / 2);
        ctx.restore();
      }
      if (elapsed < DURATION_MS) frameId = requestAnimationFrame(draw);
      else ctx.clearRect(0, 0, width, height);
    };
    frameId = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frameId);
  }, []);

  return <canvas ref={canvasRef} className="confetti" />;
}
