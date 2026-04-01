"use client";

import Link from "next/link";
import { DottedSurface } from "@/components/ui/dotted-surface";
import { ProjectUploadPanel } from "@/components/ui/project-upload-panel";

export default function UploadPage() {
  return (
    <main className="relative flex min-h-screen flex-col overflow-hidden bg-[var(--background)] text-[var(--foreground)]">

      {/* Static flat dotted background */}
      <DottedSurface playing={false} className="absolute inset-0 z-[1] opacity-25" particleColor={0xffe0c2} />

      {/* Ambient glows */}
      <div aria-hidden className="pointer-events-none absolute inset-0 z-[2]">
        <div className="absolute right-0 top-0 h-[40rem] w-[40rem] rounded-full bg-[#ffdfb5]/8 blur-3xl" />
        <div className="absolute bottom-0 left-0 h-[32rem] w-[32rem] rounded-full bg-[#644a40]/5 blur-3xl" />
      </div>

      <header className="relative z-10 flex items-center justify-between border-b border-[var(--border)] px-8 py-5">
        <Link href="/" className="text-xl font-semibold tracking-[-0.04em] text-[var(--primary)]">
          argus
        </Link>
      </header>

      <div className="relative z-10 flex flex-1 flex-col items-center justify-center px-6 py-16">
        <div className="mb-12 max-w-lg text-center">
          <p className="text-[11px] uppercase tracking-[0.42em] text-[var(--muted-foreground)]">
            Project Intake
          </p>
          <h1 className="mt-4 text-4xl font-semibold tracking-[-0.05em] text-[var(--foreground)] sm:text-5xl">
            Upload your project.
          </h1>
          <p className="mt-2 text-2xl font-semibold tracking-[-0.04em] text-[var(--primary)]">
            Start the scan.
          </p>
          <p className="mt-5 text-sm leading-7 text-[var(--muted-foreground)]">
            Drop a folder, archive, or file. Argus scans it against SOC 2, OWASP,
            GDPR, and secrets policies — then surfaces every finding with
            file-level context and severity ratings.
          </p>
        </div>

        <ProjectUploadPanel />
      </div>
    </main>
  );
}
