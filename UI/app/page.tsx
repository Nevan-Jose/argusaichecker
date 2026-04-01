import Link from "next/link";
import { ArrowRight, GitBranch, CheckCircle2, AlertTriangle } from "lucide-react";
import { BeamsBackground } from "@/components/ui/beams-background";

const stats = [
  { value: "4",    label: "Compliance frameworks" },
  { value: "200+", label: "Checks per scan" },
  { value: "<60s", label: "Median scan time" },
  { value: "100%", label: "Local processing" },
];

export default function Home() {
  return (
    <BeamsBackground className="min-h-screen text-[var(--foreground)]">
      {/* Nav */}
      <header className="relative z-10 mx-auto flex max-w-7xl items-center justify-between px-6 py-6 sm:px-10">
        <nav className="hidden items-center gap-8 text-sm text-[var(--muted-foreground)] sm:flex">
          <a href="#how-it-works" className="transition hover:text-[var(--foreground)]">How it works</a>
        </nav>
        <Link
          href="/upload"
          className="group relative overflow-hidden rounded-full bg-[#644a40] px-5 py-2.5 text-sm font-medium text-[#ffe0c2] shadow-md transition-all duration-300 hover:scale-105 hover:shadow-lg hover:shadow-[#644a40]/40 active:scale-95"
        >
          <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/40 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
          <span className="relative">Get started</span>
        </Link>
      </header>

      {/* Hero */}
      <section className="relative z-10 mx-auto max-w-7xl px-6 pb-24 pt-16 sm:px-10 sm:pt-24">
        <div className="max-w-4xl">
          <div className="inline-flex items-center gap-2 rounded-full border border-[var(--border)] bg-[var(--card)]/70 px-4 py-2 text-[11px] uppercase tracking-[0.36em] text-[var(--muted-foreground)] backdrop-blur-sm">
            <span className="size-1.5 rounded-full bg-[var(--primary)]" />
            AI Compliance Intelligence
          </div>

          <h1 className="mt-8 text-6xl font-semibold tracking-[-0.06em] text-[var(--foreground)] sm:text-8xl lg:text-[9rem]">
            Argus
          </h1>

          <p className="mt-6 max-w-2xl text-lg leading-8 text-[var(--muted-foreground)]">
            Upload your codebase. Argus scans it against SOC 2, OWASP, GDPR,
            and secrets policies — then surfaces every finding with file-level
            context and AI-powered remediation guidance.
          </p>

          <div className="mt-10 flex flex-wrap gap-4">
            <Link
              href="/upload"
              className="group relative inline-flex overflow-hidden items-center gap-2 rounded-full bg-[#644a40] px-7 py-3.5 text-sm font-medium text-[#ffe0c2] shadow-md transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-[#644a40]/40 active:scale-95"
            >
              <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/40 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
              <span className="relative flex items-center gap-2">
                Upload a project
                <ArrowRight className="size-4" />
              </span>
            </Link>
          </div>
        </div>

        {/* Stats row */}
        <div className="mt-20 grid grid-cols-2 gap-4 sm:grid-cols-4">
          {stats.map((stat) => (
            <div
              key={stat.label}
              className="rounded-2xl border border-[var(--border)] bg-[var(--card)]/70 p-5 backdrop-blur-sm"
            >
              <p className="text-3xl font-semibold tracking-tight text-[var(--primary)]">
                {stat.value}
              </p>
              <p className="mt-1 text-sm text-[var(--muted-foreground)]">
                {stat.label}
              </p>
            </div>
          ))}
        </div>
      </section>

      {/* How it works */}
      <section id="how-it-works" className="relative z-10 px-6 py-24 sm:px-10">
        <div className="mx-auto max-w-7xl">
          <p className="text-[11px] uppercase tracking-[0.38em] text-[var(--muted-foreground)]">
            The workflow
          </p>
          <h2 className="mt-5 max-w-xl text-4xl font-semibold tracking-[-0.05em] text-[var(--foreground)] sm:text-5xl">
            From upload to remediation in three steps.
          </h2>

          <div className="mt-12 grid gap-6 sm:grid-cols-3">
            {[
              {
                step: "01",
                icon: GitBranch,
                title: "Upload your project",
                body: "Drop a folder, archive, or file directly in the browser. No CLI install. No account required.",
              },
              {
                step: "02",
                icon: AlertTriangle,
                title: "Watch the scan",
                body: "Argus walks each file in real time, flagging anomalies against all four compliance frameworks simultaneously.",
              },
              {
                step: "03",
                icon: CheckCircle2,
                title: "Review findings",
                body: "Each finding shows the exact file, severity, framework, and an AI-generated remediation summary.",
              },
            ].map((item) => {
              const Icon = item.icon;
              return (
                <div key={item.step} className="rounded-2xl border border-[var(--border)] bg-[var(--card)]/70 p-6 backdrop-blur-sm">
                  <span className="font-mono text-[11px] tracking-[0.3em] text-[var(--muted-foreground)]">
                    {item.step}
                  </span>
                  <div className="mt-4 flex items-center gap-3">
                    <div className="rounded-xl bg-[var(--primary)]/10 p-2.5">
                      <Icon className="size-4 text-[var(--primary)]" />
                    </div>
                    <h3 className="font-semibold text-[var(--foreground)]">{item.title}</h3>
                  </div>
                  <p className="mt-3 text-sm leading-6 text-[var(--muted-foreground)]">
                    {item.body}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="relative z-10 px-6 py-20 sm:px-10">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-4xl font-semibold tracking-[-0.04em] text-[var(--foreground)] sm:text-5xl">
            Ready to audit your code?
          </h2>
          <p className="mt-4 text-[var(--muted-foreground)]">
            No account needed. Scan starts immediately after upload.
          </p>
          <Link
            href="/upload"
            className="group relative mt-8 inline-flex overflow-hidden items-center gap-2 rounded-full bg-[#644a40] px-8 py-4 text-sm font-medium text-[#ffe0c2] shadow-md transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-[#644a40]/40 active:scale-95"
          >
            <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/40 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
            <span className="relative flex items-center gap-2">
              Upload a project
              <ArrowRight className="size-4" />
            </span>
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="relative z-10 px-6 py-8 sm:px-10">
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-4">
          <span className="text-sm font-medium text-[var(--primary)]">argus</span>
          <p className="text-xs text-[var(--muted-foreground)]">
            AI compliance scanner — all processing is local
          </p>
        </div>
      </footer>
    </BeamsBackground>
  );
}
