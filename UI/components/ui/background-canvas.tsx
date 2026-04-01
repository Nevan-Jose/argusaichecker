"use client";

import { usePathname } from "next/navigation";
import { DottedSurface } from "@/components/ui/dotted-surface";

export function BackgroundCanvas() {
  const pathname = usePathname();
  const playing  = pathname !== "/upload";

  return (
    <DottedSurface
      playing={playing}
      particleColor={0xffe0c2}
      className="fixed inset-0 z-0 opacity-25 pointer-events-none"
    />
  );
}
