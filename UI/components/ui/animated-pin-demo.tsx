"use client";

import React from "react";
import Image from "next/image";
import { Activity, ArrowRight, ShieldCheck, Target } from "lucide-react";
import { PinContainer } from "@/components/ui/3d-pin";

export function AnimatedPinDemo() {
  return (
    <div className="flex h-[52rem] w-full items-center justify-center lg:justify-end">
      <div className="origin-center scale-[0.72] sm:scale-[0.82] xl:scale-100">
        <PinContainer
          title="Open Workspace"
          href="/workspace"
          containerClassName="mx-auto"
        >
          <div className="flex h-[34rem] w-[30rem] flex-col rounded-[2rem] border border-cyan-400/20 bg-gradient-to-b from-[#08151d]/92 via-[#071018]/88 to-[#04080c]/84 p-5 tracking-tight text-slate-100/50 backdrop-blur-sm">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="size-3 animate-pulse rounded-full bg-emerald-400" />
                <div className="text-xs text-[#8cc7c8]">Live Connection</div>
              </div>
              <Target className="size-5 text-[#6fd9f4]" />
            </div>

            <div className="mt-6 flex-1 space-y-5">
              <div>
                <div className="text-[3rem] font-semibold tracking-[-0.06em] text-[#effefd]">
                  Argus Review Mesh
                </div>
                <div className="mt-2 text-lg text-[#93c8c9]">
                  Pinned compliance signal surface
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2 rounded-[1.75rem] border border-cyan-400/12 bg-cyan-300/[0.03] p-5">
                  <div className="flex items-center gap-2">
                    <Activity className="size-5 text-sky-400" />
                    <div className="text-sm text-[#8ebcc3]">Signals</div>
                  </div>
                  <div className="text-6xl font-bold tracking-[-0.06em] text-sky-400">
                    427
                  </div>
                  <div className="text-sm text-[#71959b]">Active findings</div>
                </div>

                <div className="space-y-2 rounded-[1.75rem] border border-emerald-400/12 bg-emerald-300/[0.03] p-5">
                  <div className="flex items-center gap-2">
                    <ShieldCheck className="size-5 text-emerald-400" />
                    <div className="text-sm text-[#8ebcc3]">Integrity</div>
                  </div>
                  <div className="text-6xl font-bold tracking-[-0.06em] text-emerald-400">
                    98%
                  </div>
                  <div className="text-sm text-[#71959b]">Controls aligned</div>
                </div>
              </div>

              <div className="relative flex-1 overflow-hidden rounded-[1.75rem] border border-cyan-400/12 bg-[#03070b]/92">
                <div className="absolute inset-x-0 top-0 z-10 bg-gradient-to-b from-black/45 to-transparent p-5">
                  <span className="text-sm uppercase tracking-[0.42em] text-[#63d8f2]">
                    Findings preview
                  </span>
                </div>

                <Image
                  src="/findings-preview.svg"
                  alt="Preview of Argus findings list"
                  width={1200}
                  height={760}
                  priority
                  className="h-full w-full object-cover object-top opacity-95"
                />
              </div>

              <div className="flex items-end justify-between">
                <div className="text-sm text-[#73969b]">
                  Workspace preview synced now
                </div>
                <div className="flex items-center gap-2 text-base font-medium text-[#63d8f2]">
                  Open
                  <ArrowRight className="size-4" />
                </div>
              </div>
            </div>
          </div>
        </PinContainer>
      </div>
    </div>
  );
}
