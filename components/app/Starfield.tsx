"use client";

import { useEffect, useRef } from "react";
import { CosmicBackdrop } from "@/components/app/CosmicBackdrop";

export function Starfield({ density = 220 }: { density?: number }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    let raf: number;
    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);
    const stars = Array.from({ length: density }, () => ({
      x: Math.random() * window.innerWidth,
      y: Math.random() * window.innerHeight,
      r: Math.random() * 1.6 + 0.3,
      speed: Math.random() * 0.1 + 0.02,
      opacity: Math.random(),
      td: Math.random() * 0.012 + 0.003,
      tdir: Math.random() > 0.5 ? 1 : -1,
    }));
    const draw = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      stars.forEach((s) => {
        s.opacity += s.td * s.tdir;
        if (s.opacity > 1 || s.opacity < 0.05) s.tdir *= -1;
        s.y += s.speed;
        if (s.y > canvas.height) {
          s.y = 0;
          s.x = Math.random() * canvas.width;
        }
        ctx.beginPath();
        ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(255,255,255,${s.opacity})`;
        ctx.fill();
      });
      raf = requestAnimationFrame(draw);
    };
    draw();
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", resize);
    };
  }, [density]);
  return (
    <canvas
      ref={canvasRef}
      className="pointer-events-none fixed inset-0 z-0 h-full w-full"
    />
  );
}

export function SpaceFx() {
  return (
    <>
      <Starfield />
      <CosmicBackdrop />
      <div className="scanlines pointer-events-none fixed inset-0 z-[2]" />
      <div className="scan-sweep pointer-events-none fixed top-0 left-0 z-[2] w-full" />
    </>
  );
}
