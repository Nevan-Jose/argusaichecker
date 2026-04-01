"use client";

import { useEffect, useRef, useState } from "react";
import { DottedSurface } from "@/components/ui/dotted-surface";
import { useRouter } from "next/navigation";
import Link from "next/link";

type AnalysisSession = {
  projectName: string;
  fileCount: number;
  totalSize: number;
  topFiles: string[];
  createdAt?: number;
};

const SESSION_KEY = "argus-analysis-session";

const fallbackSession: AnalysisSession = {
  projectName: "GDPR",
  fileCount: 248,
  totalSize: 18_900_000,
  topFiles: ["src/config/aws.ts", "src/routes/users.ts", "src/services/auth.ts", "src/middleware/session.ts"],
};

type ErrorType = "critical" | "high" | "medium" | null;

const ALL_FILES: { path: string; errorType: ErrorType }[] = [
  { path: "src/utils/sanitize.ts",          errorType: null },
  { path: "src/auth/middleware.ts",          errorType: "high" },
  { path: "src/utils/validation.ts",         errorType: null },
  { path: "src/controllers/auth.ts",         errorType: "high" },
  { path: "src/config/env.ts",               errorType: null },
  { path: "src/models/Token.ts",             errorType: "medium" },
  { path: "src/api/routes/admin.ts",         errorType: "medium" },
  { path: "src/utils/logger.ts",             errorType: null },
  { path: "src/models/Session.ts",           errorType: "critical" },
  { path: "src/middleware/rateLimit.ts",     errorType: "medium" },
  { path: "src/services/auth.ts",            errorType: "high" },
  { path: "src/config/database.ts",          errorType: "medium" },
  { path: "src/services/storage.ts",         errorType: null },
  { path: "src/routes/users.ts",             errorType: "critical" },
  { path: "src/api/middleware/cors.ts",      errorType: "medium" },
  { path: "src/utils/crypto.ts",             errorType: "high" },
  { path: "src/models/User.ts",              errorType: null },
  { path: "src/config/jwt.ts",               errorType: "critical" },
  { path: "src/controllers/session.ts",      errorType: "medium" },
  { path: "src/utils/hash.ts",               errorType: "high" },
  { path: "src/api/handlers/upload.ts",      errorType: "high" },
  { path: "src/services/email.ts",           errorType: null },
  { path: "src/config/redis.ts",             errorType: "medium" },
  { path: "src/middleware/auth.ts",          errorType: "high" },
  { path: "src/models/Role.ts",              errorType: null },
  { path: "src/utils/format.ts",             errorType: null },
  { path: "src/routes/api.ts",               errorType: "critical" },
  { path: "src/services/oauth.ts",           errorType: "high" },
  { path: "src/config/cors.ts",              errorType: "medium" },
  { path: "src/controllers/user.ts",         errorType: "high" },
  { path: "src/middleware/csrf.ts",          errorType: "critical" },
  { path: "src/services/token.ts",           errorType: "medium" },
  { path: "src/utils/encrypt.ts",            errorType: null },
  { path: "src/models/Permission.ts",        errorType: "high" },
  { path: "src/config/secrets.ts",           errorType: "critical" },
];

type RadarDot = {
  left: string;
  top: string;
  color: string;
  size: number;
  revealed: boolean;
};

const SWEEP_DURATION_MS = 8000;

function msUntilAngle(sweepStartMs: number, targetAngle: number): number {
  const elapsed = (Date.now() - sweepStartMs) % SWEEP_DURATION_MS;
  const current = (elapsed / SWEEP_DURATION_MS) * 360;
  const delta   = current <= targetAngle ? targetAngle - current : 360 - current + targetAngle;
  return (delta / 360) * SWEEP_DURATION_MS;
}

// Generates a dot just ahead of the current sweep position so it always appears
// when the sweep line physically crosses it. Positions are relative to the
// circle container (inset-[8%] div) so dots are guaranteed inside the circle.
function generateRadarDot(
  errorType: ErrorType,
  sweepStartMs: number,
  angleOffset = 0,
): RadarDot & { angle: number } {
  const elapsed      = (Date.now() - sweepStartMs) % SWEEP_DURATION_MS;
  const currentAngle = (elapsed / SWEEP_DURATION_MS) * 360;
  const lookahead    = 8 + Math.random() * 35 + angleOffset;
  const angleDeg     = (currentAngle + lookahead + 360) % 360;
  const theta        = (angleDeg * Math.PI) / 180;
  // sqrt for uniform area distribution; r stays well inside the circle radius (50%)
  const r            = Math.sqrt(0.06 + Math.random() * 0.78);
  const left         = 50 + Math.sin(theta) * r * 46;
  const top          = 50 - Math.cos(theta) * r * 46;
  const color        =
    errorType === "critical" ? "#cc3a2a"
    : errorType === "high"   ? "#c47040"
    : "#c4a030";
  const sizes = [3, 3, 4, 5, 6, 7];
  const size  = sizes[Math.floor(Math.random() * sizes.length)];
  return { left: `${left.toFixed(1)}%`, top: `${top.toFixed(1)}%`, color, size, revealed: false, angle: angleDeg };
}

const ACTIVITY_MESSAGES = [
  "Initializing ruleset...",
  "Parsing AST for authentication flows...",
  "Checking injection vulnerabilities...",
  "Analyzing middleware chain...",
  "Suspicious pattern in auth middleware",
  "Scanning route handlers...",
  "Elevated privileges detected in admin routes",
  "Evaluating session management...",
  "Running OWASP Top 10 checks...",
  "Critical: session token exposure detected",
  "Checking data transmission security...",
  "Analyzing dependency graph...",
  "Insecure storage pattern in services/auth",
  "Scanning configuration files...",
  "Critical: hardcoded credentials in users route",
  "Checking cryptographic implementations...",
  "Weak cipher usage in utils/crypto",
  "Auditing JWT configuration...",
  "Critical: JWT secret exposure in config/jwt",
  "Finalizing analysis...",
];

function getFrameworkStatus(name: string, progress: number, done: boolean): { status: string; active: boolean } {
  if (done) return { status: "complete", active: false };
  if (name === "SOC 2") {
    if (progress >= 50) return { status: "complete", active: false };
    return { status: "scanning...", active: true };
  }
  if (name === "OWASP") {
    if (progress < 30) return { status: "queued", active: false };
    if (progress >= 85) return { status: "complete", active: false };
    return { status: "scanning...", active: true };
  }
  if (name === "GDPR") {
    if (progress < 65) return { status: "queued", active: false };
    return { status: "scanning...", active: true };
  }
  return { status: "queued", active: false };
}

const FRAMEWORKS = [
  { name: "SOC 2", detailTop: "Type II controls" },
  { name: "OWASP", detailTop: "Top 10 vulns" },
  { name: "GDPR",  detailTop: "Data & privacy" },
];

export function ScanConsole() {
  const router = useRouter();

  const [session, setSession]             = useState<AnalysisSession>(fallbackSession);
  const [scannedCount, setScannedCount]   = useState(0);
  const [radarDots, setRadarDots]         = useState<RadarDot[]>([]);
  const [activityIndex, setActivityIndex] = useState(0);
  const [scanDone, setScanDone]           = useState(false);

  const listRef       = useRef<HTMLDivElement>(null);
  const sweepStartRef = useRef(Date.now());
  const dotCountRef   = useRef(0);
  const pendingIds    = useRef<ReturnType<typeof setTimeout>[]>([]);

  useEffect(() => {
    const stored = sessionStorage.getItem(SESSION_KEY);
    if (!stored) return;
    try { setSession(JSON.parse(stored) as AnalysisSession); } catch { /* fallback */ }
  }, []);

  // ── Scan simulation ──
  useEffect(() => {
    if (scannedCount >= ALL_FILES.length) {
      setScanDone(true);
      return;
    }

    const timer = setTimeout(() => {
      const file = ALL_FILES[scannedCount];
      setScannedCount((n) => n + 1);

      if (file.errorType !== null) {
        // Critical errors get 3 dots, high gets 2, medium gets 1
        const count = file.errorType === "critical" ? 3 : file.errorType === "high" ? 2 : 1;
        for (let d = 0; d < count; d++) {
          const dot = generateRadarDot(file.errorType, sweepStartRef.current, d * 12);
          const idx = dotCountRef.current;
          dotCountRef.current += 1;
          setRadarDots((prev) => [...prev, { left: dot.left, top: dot.top, color: dot.color, size: dot.size, revealed: false }]);
          const delay = msUntilAngle(sweepStartRef.current, dot.angle);
          const id = setTimeout(() => {
            setRadarDots((prev) => prev.map((dd, i) => i === idx ? { ...dd, revealed: true } : dd));
          }, delay);
          pendingIds.current.push(id);
        }
      }

      const msgIdx = Math.round((scannedCount / (ALL_FILES.length - 1)) * (ACTIVITY_MESSAGES.length - 1));
      setActivityIndex(Math.min(msgIdx, ACTIVITY_MESSAGES.length - 1));
    }, 500);

    return () => clearTimeout(timer);
  }, [scannedCount]);

  // Cleanup pending timeouts on unmount
  useEffect(() => {
    return () => { pendingIds.current.forEach(clearTimeout); };
  }, []);

  // Smooth-scroll file list to bottom
  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTo({ top: listRef.current.scrollHeight, behavior: "smooth" });
    }
  }, [scannedCount]);

  const progress     = Math.round((scannedCount / ALL_FILES.length) * 100);
  const visibleFiles = ALL_FILES.slice(0, scannedCount);
  const currentFile  = scannedCount < ALL_FILES.length ? ALL_FILES[scannedCount] : null;

  return (
    <div className="relative min-h-screen overflow-hidden bg-[var(--background)]">
      <DottedSurface className="absolute inset-0 z-[1] opacity-25" particleColor={0xffe0c2} />

      <div className="pointer-events-none absolute inset-0 z-[2]">
        <div className="absolute right-0 top-0 h-[40rem] w-[40rem] rounded-full bg-[#ffdfb5]/6 blur-3xl" />
        <div className="absolute bottom-0 left-0 h-[32rem] w-[32rem] rounded-full bg-[#644a40]/5 blur-3xl" />
      </div>

      <div className="relative z-10 min-h-screen">

        {/* ── Header ── */}
        <header className="flex items-center justify-between border-b border-[var(--border)] px-8 py-5">
          <Link href="/" className="text-xl font-semibold tracking-[-0.04em] text-[var(--primary)]">
            argus
          </Link>
          <div className="hidden text-[11px] uppercase tracking-[0.52em] text-[var(--muted-foreground)] lg:block">
            Analysis In Progress
          </div>
          <Link href="/results" className="group relative overflow-hidden rounded border border-[#644a40]/40 bg-[#644a40]/20 px-4 py-2 shadow-sm transition-all duration-300 hover:scale-105 hover:border-[#644a40]/70 hover:shadow-md hover:shadow-[#644a40]/20 active:scale-95" aria-label="View findings">
            <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/30 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
            <div className="flex items-end gap-[3px]">
              {[10, 6, 14, 8, 12, 6, 10, 14].map((h, i) => (
                <span key={i} className="w-[3px] rounded-sm bg-[var(--muted-foreground)]/30" style={{ height: `${h}px` }} />
              ))}
            </div>
          </Link>
        </header>

        {/* ── Main canvas ── */}
        <div className="relative mx-auto max-w-[1700px] px-4 pb-12 sm:px-8 lg:min-h-[calc(100vh-4rem)]">
          <div className="flex flex-col gap-10 lg:block">

            {/* ── File list — LEFT ── */}
            <div className="lg:absolute lg:left-[10%] lg:top-[28%]">
              <p className="text-[9px] uppercase tracking-[0.52em] text-[var(--muted-foreground)]/60">
                Scanning Files
              </p>
            </div>
            <section className="order-2 lg:absolute lg:left-[10%] lg:top-[33%] lg:w-[16rem]">
              <div
                ref={listRef}
                className="space-y-[3px] overflow-y-auto"
                style={{ maxHeight: "22rem", scrollbarWidth: "none" }}
              >
                {visibleFiles.map((file, i) => (
                  <div
                    key={i}
                    className="flex items-center justify-between gap-2"
                    style={{ animation: "slide-in 0.35s ease-out both" }}
                  >
                    <span
                      className="font-mono text-[10px] leading-[1.4]"
                      style={{
                        color:
                          file.errorType === "critical" ? "#cc3a2a"
                          : file.errorType === "high"   ? "#c47040"
                          : file.errorType === "medium" ? "#c4a030"
                          : "var(--muted-foreground)",
                        opacity: file.errorType ? 1 : 0.5,
                      }}
                    >
                      {file.path}
                    </span>
                    <span
                      className="shrink-0 font-mono text-[9px] uppercase tracking-[0.3em]"
                      style={{
                        color:
                          file.errorType === "critical" ? "#cc3a2a"
                          : file.errorType === "high"   ? "#c47040"
                          : file.errorType === "medium" ? "#c4a030"
                          : "transparent",
                      }}
                    >
                      {file.errorType === "critical" ? "FLAG" : file.errorType ? "WARN" : "—"}
                    </span>
                  </div>
                ))}
                {currentFile && (
                  <div
                    className="flex items-center justify-between gap-2"
                    style={{ animation: "slide-in 0.35s ease-out both" }}
                  >
                    <span
                      className="font-mono text-[10px] leading-[1.4] [animation:pulse-soft_1.4s_ease-in-out_infinite]"
                      style={{ color: "var(--foreground)", opacity: 0.45 }}
                    >
                      {currentFile.path}
                    </span>
                    <span
                      className="shrink-0 font-mono text-[9px] [animation:pulse-soft_1.4s_ease-in-out_infinite]"
                      style={{ color: "var(--muted-foreground)", opacity: 0.35 }}
                    >
                      ...
                    </span>
                  </div>
                )}
              </div>
            </section>

            {/* ── Radar circle + legend ── */}
            <section className="order-1 mx-auto flex w-full max-w-xl flex-col items-center lg:absolute lg:left-1/2 lg:top-[6%] lg:max-w-none lg:-translate-x-1/2">
              <div className="relative h-[14rem] w-[14rem] sm:h-[18rem] sm:w-[18rem] lg:h-[22rem] lg:w-[22rem]">

                {/* Concentric rings */}
                <div className="absolute inset-[8%]  rounded-full border border-[var(--border)]" style={{ opacity: 0.9 }} />
                <div className="absolute inset-[22%] rounded-full border border-[var(--border)]" style={{ opacity: 0.65 }} />
                <div className="absolute inset-[36%] rounded-full border border-[var(--border)]" style={{ opacity: 0.4 }} />

                {/* Disk glow */}
                <div className="absolute inset-[8%] rounded-full bg-[radial-gradient(circle_at_50%_50%,rgba(100,74,64,0.07),transparent_70%)]" />

                {/* Clip ring — overflow-hidden kept separate from animated element */}
                <div
                  className="absolute inset-[8%] overflow-hidden rounded-full transition-opacity duration-700"
                  style={{ opacity: scanDone ? 0 : 1 }}
                >
                  {/* Rotating sweep — will-change:transform enables GPU compositing */}
                  <div
                    className="absolute inset-0 [animation:radar-sweep_8s_linear_infinite] [will-change:transform]"
                  >
                    <div
                      className="absolute inset-0"
                      style={{
                        background: "conic-gradient(from -70deg at 50% 50%, transparent 0deg, rgba(100,74,64,0.02) 15deg, rgba(100,74,64,0.07) 45deg, rgba(100,74,64,0.11) 63deg, rgba(100,74,64,0.03) 68deg, transparent 70deg, transparent 360deg)",
                      }}
                    />
                    <div
                      style={{
                        position: "absolute",
                        left: "50%",
                        top: 0,
                        height: "50%",
                        width: "1px",
                        transform: "translateX(-50%)",
                        background: "linear-gradient(to top, transparent 0%, rgba(158,142,132,0.25) 35%, rgba(158,142,132,0.7) 100%)",
                      }}
                    />
                  </div>
                </div>

                {/* Dots container — clipped to circle boundary, no overflow */}
                <div className="pointer-events-none absolute inset-[8%] overflow-hidden rounded-full">
                  {radarDots.map((dot, i) =>
                    !dot.revealed ? null : (
                      <div
                        key={i}
                        style={{
                          position: "absolute",
                          left: dot.left,
                          top: dot.top,
                          transform: "translate(-50%, -50%)",
                          animation: "dot-appear 0.45s ease-out forwards",
                        }}
                      >
                        <div style={{ animation: "pulse-soft 3.4s ease-in-out 0.45s infinite" }}>
                          <div style={{ position: "absolute", inset: -(dot.size * 2.2), borderRadius: "50%", backgroundColor: dot.color, opacity: 0.12, filter: "blur(6px)" }} />
                          <div style={{ position: "absolute", inset: -(dot.size * 0.8), borderRadius: "50%", backgroundColor: dot.color, opacity: 0.22, filter: "blur(2px)" }} />
                          <div style={{ width: dot.size, height: dot.size, borderRadius: "50%", backgroundColor: dot.color, position: "relative", zIndex: 1, boxShadow: `0 0 ${dot.size * 0.7}px 1px ${dot.color}` }} />
                        </div>
                      </div>
                    )
                  )}
                </div>
              </div>

              {/* Severity legend */}
              <div className="mt-2 flex items-center gap-5 text-[10px] text-[var(--muted-foreground)]/70">
                <span className="flex items-center gap-1.5"><span className="size-1.5 rounded-full bg-[#cc3a2a]" />Critical</span>
                <span className="flex items-center gap-1.5"><span className="size-1.5 rounded-full bg-[#c47040]" />High</span>
                <span className="flex items-center gap-1.5"><span className="size-1.5 rounded-full bg-[#c4a030]" />Medium</span>
              </div>
            </section>

            {/* ── Scanning label + progress / See Results ── */}
            <div className="order-4 text-center lg:absolute lg:left-1/2 lg:top-[66%] lg:-translate-x-1/2">
              <p className="font-serif text-3xl italic tracking-tight text-[var(--foreground)]">
                <em>{scanDone ? "Analysis complete" : "Scanning"}</em>
              </p>
              <p className="mt-1 text-xs italic text-[var(--muted-foreground)]">
                {scanDone ? `${ALL_FILES.length} files processed` : `Running ${session.projectName} checks...`}
              </p>
              <p className="mt-1 h-3.5 font-mono text-[9px] italic text-[var(--muted-foreground)]/60 transition-all duration-500">
                {scanDone ? "" : ACTIVITY_MESSAGES[activityIndex]}
              </p>

              {!scanDone ? (
                <div className="mx-auto mt-4 w-[11rem] max-w-full">
                  <div className="h-px bg-[var(--border)]">
                    <div
                      className="h-px bg-[var(--primary)] transition-all duration-500"
                      style={{ width: `${progress}%` }}
                    />
                  </div>
                  <p className="mt-2 text-lg font-light tracking-[0.3em] text-[var(--primary)]">
                    {progress}%
                  </p>
                </div>
              ) : (
                <div className="mt-6" style={{ animation: "fade-up 0.7s ease-out both" }}>
                  <button
                    onClick={() => router.push("/results")}
                    className="group relative overflow-hidden rounded-full bg-[#644a40] px-8 py-3 text-sm font-medium text-[#ffe0c2] shadow-md transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-[#644a40]/40 active:scale-95"
                  >
                    <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/40 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
                    <span className="relative">See Results</span>
                  </button>
                </div>
              )}
            </div>

            {/* ── Frameworks — RIGHT ── */}
            <div className="lg:absolute lg:right-[10%] lg:top-[28%]">
              <p className="text-[9px] uppercase tracking-[0.52em] text-[var(--muted-foreground)]/60">
                Frameworks
              </p>
            </div>
            <section className="order-3 lg:absolute lg:right-[10%] lg:top-[33%] lg:w-[10rem]">
              <div className="space-y-6">
                {FRAMEWORKS.map((fw) => {
                  const { status, active } = getFrameworkStatus(fw.name, progress, scanDone);
                  return (
                    <div key={fw.name}>
                      <div className="flex items-center gap-2.5">
                        <span
                          className="size-2 rounded-full transition-all duration-700"
                          style={{
                            backgroundColor: active ? "var(--primary)" : status === "complete" ? "#4a9e6a" : "var(--border)",
                            boxShadow: active ? "0 0 7px rgba(100,74,64,0.6)" : "none",
                          }}
                        />
                        <p
                          className="text-[10px] uppercase tracking-[0.36em] transition-colors duration-700"
                          style={{ color: active || status === "complete" ? "var(--foreground)" : "var(--muted-foreground)" }}
                        >
                          {fw.name}
                        </p>
                      </div>
                      <p className="mt-1 pl-[18px] text-[10px] text-[var(--muted-foreground)]/50">
                        {fw.detailTop}
                      </p>
                      <p
                        className="pl-[18px] text-[10px] transition-colors duration-700"
                        style={{ color: active ? "var(--muted-foreground)" : status === "complete" ? "#4a9e6a" : "var(--border)" }}
                      >
                        {status}
                      </p>
                    </div>
                  );
                })}
              </div>
            </section>

          </div>
        </div>
      </div>
    </div>
  );
}
