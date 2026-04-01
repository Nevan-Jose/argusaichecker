"use client";

import { useEffect, useRef } from "react";
import { motion } from "motion/react";
import { cn } from "@/lib/utils";

interface AnimatedGradientBackgroundProps {
  className?: string;
  children?: React.ReactNode;
  intensity?: "subtle" | "medium" | "strong";
}

interface Beam {
  x: number;
  y: number;
  width: number;
  length: number;
  angle: number;
  speed: number;
  opacity: number;
  hue: number;
  saturation: number;
  lightness: number;
  pulse: number;
  pulseSpeed: number;
  profile: "whisper" | "core" | "flare";
}

function createBeam(width: number, height: number): Beam {
  const random = Math.random();
  const angle = -35 + Math.random() * 10;
  const profile =
    random < 0.34 ? "whisper" : random < 0.8 ? "core" : "flare";

  // Warm amber/brown palette: hue 18–42°
  const profileMap = {
    whisper: {
      width: 16 + Math.random() * 26,
      speed: 0.34 + Math.random() * 0.35,
      opacity: 0.06 + Math.random() * 0.08,
      hue: 18 + Math.random() * 10,
      saturation: 70 + Math.random() * 15,
      lightness: 55 + Math.random() * 10,
      pulseSpeed: 0.012 + Math.random() * 0.018,
      length: height * (2 + Math.random() * 0.35),
    },
    core: {
      width: 38 + Math.random() * 46,
      speed: 0.46 + Math.random() * 0.48,
      opacity: 0.1 + Math.random() * 0.12,
      hue: 28 + Math.random() * 10,
      saturation: 80 + Math.random() * 12,
      lightness: 60 + Math.random() * 10,
      pulseSpeed: 0.016 + Math.random() * 0.02,
      length: height * (2.15 + Math.random() * 0.4),
    },
    flare: {
      width: 84 + Math.random() * 110,
      speed: 0.24 + Math.random() * 0.26,
      opacity: 0.12 + Math.random() * 0.12,
      hue: 35 + Math.random() * 8,
      saturation: 85 + Math.random() * 10,
      lightness: 65 + Math.random() * 10,
      pulseSpeed: 0.01 + Math.random() * 0.012,
      length: height * (2.35 + Math.random() * 0.4),
    },
  } as const;

  const selectedProfile = profileMap[profile];

  return {
    x: Math.random() * width * 1.5 - width * 0.25,
    y: Math.random() * height * 1.5 - height * 0.25,
    width: selectedProfile.width,
    length: selectedProfile.length,
    angle,
    speed: selectedProfile.speed,
    opacity: selectedProfile.opacity,
    hue: selectedProfile.hue,
    saturation: selectedProfile.saturation,
    lightness: selectedProfile.lightness,
    pulse: Math.random() * Math.PI * 2,
    pulseSpeed: selectedProfile.pulseSpeed,
    profile,
  };
}

export function BeamsBackground({
  className,
  intensity = "strong",
  children,
}: AnimatedGradientBackgroundProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const beamsRef = useRef<Beam[]>([]);
  const animationFrameRef = useRef<number>(0);
  const MINIMUM_BEAMS = 20;

  const opacityMap = {
    subtle: 0.56,
    medium: 0.72,
    strong: 0.9,
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const updateCanvasSize = () => {
      const dpr = window.devicePixelRatio || 1;
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      canvas.width = viewportWidth * dpr;
      canvas.height = viewportHeight * dpr;
      canvas.style.width = `${viewportWidth}px`;
      canvas.style.height = `${viewportHeight}px`;
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.scale(dpr, dpr);

      const totalBeams = MINIMUM_BEAMS * 1.5;
      beamsRef.current = Array.from({ length: totalBeams }, () =>
        createBeam(viewportWidth, viewportHeight)
      );
    };

    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    function resetBeam(beam: Beam, index: number, totalBeams: number) {
      if (!canvas) return beam;

      const column = index % 3;
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      const spacing = viewportWidth / 3;

      beam.y = viewportHeight + 100;
      beam.x =
        column * spacing +
        spacing / 2 +
        (Math.random() - 0.5) * spacing * 0.5;

      if (beam.profile === "whisper") {
        beam.width = 16 + Math.random() * 24;
        beam.speed = 0.32 + Math.random() * 0.32;
        beam.hue = 18 + (index * 4) / totalBeams;
        beam.saturation = 70 + Math.random() * 15;
        beam.lightness = 55 + Math.random() * 10;
        beam.opacity = 0.06 + Math.random() * 0.08;
        beam.length = viewportHeight * (2 + Math.random() * 0.35);
      } else if (beam.profile === "flare") {
        beam.width = 92 + Math.random() * 110;
        beam.speed = 0.22 + Math.random() * 0.22;
        beam.hue = 35 + (index * 3) / totalBeams;
        beam.saturation = 85 + Math.random() * 10;
        beam.lightness = 65 + Math.random() * 10;
        beam.opacity = 0.12 + Math.random() * 0.12;
        beam.length = viewportHeight * (2.3 + Math.random() * 0.4);
      } else {
        beam.width = 38 + Math.random() * 52;
        beam.speed = 0.44 + Math.random() * 0.42;
        beam.hue = 28 + (index * 4) / totalBeams;
        beam.saturation = 80 + Math.random() * 12;
        beam.lightness = 60 + Math.random() * 10;
        beam.opacity = 0.10 + Math.random() * 0.12;
        beam.length = viewportHeight * (2.12 + Math.random() * 0.4);
      }

      return beam;
    }

    function drawBeam(context: CanvasRenderingContext2D, beam: Beam) {
      context.save();
      context.translate(beam.x, beam.y);
      context.rotate((beam.angle * Math.PI) / 180);

      const pulsingOpacity =
        beam.opacity *
        (0.8 + Math.sin(beam.pulse) * 0.2) *
        opacityMap[intensity];

      const gradient = context.createLinearGradient(0, 0, 0, beam.length);

      gradient.addColorStop(
        0,
        `hsla(${beam.hue}, ${beam.saturation}%, ${beam.lightness}%, 0)`
      );
      gradient.addColorStop(
        0.1,
        `hsla(${beam.hue}, ${beam.saturation}%, ${beam.lightness}%, ${
          pulsingOpacity * 0.3
        })`
      );
      gradient.addColorStop(
        0.34,
        `hsla(${beam.hue}, ${beam.saturation}%, ${beam.lightness + 1}%, ${
          pulsingOpacity * 0.8
        })`
      );
      gradient.addColorStop(
        0.6,
        `hsla(${beam.hue}, ${beam.saturation}%, ${beam.lightness + 3}%, ${
          pulsingOpacity
        })`
      );
      gradient.addColorStop(
        0.9,
        `hsla(${beam.hue}, ${beam.saturation - 8}%, ${beam.lightness}%, ${
          pulsingOpacity * 0.38
        })`
      );
      gradient.addColorStop(
        0.96,
        `hsla(${beam.hue}, ${beam.saturation - 10}%, ${beam.lightness - 4}%, ${
          pulsingOpacity * 0.1
        })`
      );
      gradient.addColorStop(
        1,
        `hsla(${beam.hue}, ${beam.saturation}%, ${beam.lightness}%, 0)`
      );

      context.fillStyle = gradient;
      context.fillRect(-beam.width / 2, 0, beam.width, beam.length);
      context.restore();
    }

    function animate() {
      if (!canvas || !ctx) return;

      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.filter = "blur(35px)";

      const totalBeams = beamsRef.current.length;
      beamsRef.current.forEach((beam, index) => {
        beam.y -= beam.speed;
        beam.pulse += beam.pulseSpeed;

        if (beam.y + beam.length < -100) {
          resetBeam(beam, index, totalBeams);
        }

        drawBeam(ctx, beam);
      });

      animationFrameRef.current = requestAnimationFrame(animate);
    }

    animate();

    return () => {
      window.removeEventListener("resize", updateCanvasSize);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [intensity]);

  return (
    <div className={cn("relative w-full bg-[var(--background)]", className)}>
      {/* Fixed canvas so beams cover the full page as user scrolls */}
      <canvas
        ref={canvasRef}
        className="fixed inset-0 pointer-events-none z-0"
        style={{ filter: "blur(15px)" }}
      />

      <motion.div
        className="fixed inset-0 pointer-events-none z-0"
        animate={{ opacity: [0.4, 0.6, 0.4] }}
        transition={{ duration: 10, ease: "easeInOut", repeat: Number.POSITIVE_INFINITY }}
        style={{ backdropFilter: "blur(40px)" }}
      />

      <div className="relative z-10">
        {children}
      </div>
    </div>
  );
}
