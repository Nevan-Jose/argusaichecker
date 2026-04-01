"use client";

import { useState } from "react";
import Link from "next/link";
import { AlignJustify } from "lucide-react";

type Finding = {
  id: string;
  severity: "Critical" | "High" | "Medium";
  framework: "SOC2" | "GDPR" | "OWASP";
  title: string;
  summary: string;
  file: string;
  highlightLine: number;
  code: string[];
};

const findings: Finding[] = [
  {
    id: "01",
    severity: "Critical",
    framework: "SOC2",
    title: "Hardcoded AWS Credentials",
    summary: "AWS access key and secret embedded directly in source file.",
    file: "src/config/aws.ts",
    highlightLine: 4,
    code: [
      "const config = {",
      "    region: 'us-east-1',",
      "    accessKeyId: 'AKIAIOSFODNN7EXAMPLE',",
      "    secretAccessKey: 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY',",
      "    bucket: 'argus-production-data',",
      "}",
      "",
      "export default config",
      "",
    ],
  },
  {
    id: "02",
    severity: "High",
    framework: "SOC2",
    title: "Unauthenticated API Endpoint",
    summary: "POST /api/users has no auth middleware — any caller can create accounts.",
    file: "src/routes/users.ts",
    highlightLine: 6,
    code: [
      "import express from 'express'",
      "import { createUser } from '../services/user'",
      "",
      "const router = express.Router()",
      "",
      "router.post('/users', async (req, res) => {",
      "    const user = await createUser(req.body)",
      "    res.json(user)",
      "})",
      "",
      "export default router",
    ],
  },
  {
    id: "03",
    severity: "High",
    framework: "GDPR",
    title: "PII Written to Console",
    summary: "User email address logged to stdout on every login event.",
    file: "src/services/auth.ts",
    highlightLine: 4,
    code: [
      "export async function login(email: string, password: string) {",
      "    const user = await db.users.findOne({ email })",
      "    if (!user || !bcrypt.compare(password, user.hash)) {",
      "        console.log(`Login attempt: ${email}`)",
      "        throw new Error('Invalid credentials')",
      "    }",
      "    return generateToken(user)",
      "}",
      "",
    ],
  },
  {
    id: "04",
    severity: "High",
    framework: "OWASP",
    title: "Insecure Cookie Configuration",
    summary: "Session cookie missing HttpOnly and Secure flags.",
    file: "src/middleware/session.ts",
    highlightLine: 5,
    code: [
      "import session from 'express-session'",
      "",
      "export const sessionMiddleware = session({",
      "    secret: process.env.SESSION_SECRET!,",
      "    cookie: { maxAge: 86400000 },",
      "    resave: false,",
      "    saveUninitialized: false,",
      "})",
      "",
    ],
  },
  {
    id: "05",
    severity: "High",
    framework: "OWASP",
    title: "Weak Hashing Algorithm (MD5)",
    summary: "MD5 used to hash user passwords — cryptographically broken.",
    file: "src/utils/crypto.ts",
    highlightLine: 3,
    code: [
      "import crypto from 'crypto'",
      "",
      "export const hashPassword = (pw: string) =>",
      "    crypto.createHash('md5').update(pw).digest('hex')",
      "",
      "export const verifyPassword = (pw: string, hash: string) =>",
      "    hashPassword(pw) === hash",
      "",
    ],
  },
  {
    id: "06",
    severity: "Medium",
    framework: "OWASP",
    title: "Missing Rate Limiting",
    summary: "No rate limiting on authentication endpoints.",
    file: "src/routes/auth.ts",
    highlightLine: 4,
    code: [
      "import express from 'express'",
      "import { loginHandler } from '../controllers/auth'",
      "",
      "router.post('/login', loginHandler)",
      "router.post('/register', registerHandler)",
      "",
      "export default router",
      "",
    ],
  },
  {
    id: "07",
    severity: "Medium",
    framework: "SOC2",
    title: "Verbose Error Messages",
    summary: "Stack traces exposed to clients in production.",
    file: "src/middleware/error.ts",
    highlightLine: 4,
    code: [
      "export function errorHandler(err: Error, req: any, res: any) {",
      "    console.error(err)",
      "    res.status(500).json({",
      "        message: err.message,",
      "        stack: err.stack,",
      "    })",
      "}",
      "",
    ],
  },
  {
    id: "08",
    severity: "Medium",
    framework: "GDPR",
    title: "Missing Data Retention Policy",
    summary: "User data stored indefinitely without automated cleanup.",
    file: "src/models/User.ts",
    highlightLine: 6,
    code: [
      "const UserSchema = new Schema({",
      "    email:     { type: String, required: true },",
      "    password:  { type: String, required: true },",
      "    createdAt: { type: Date, default: Date.now },",
      "    lastLogin: { type: Date },",
      "    data:      { type: Object },",
      "})",
      "",
      "export const User = model('User', UserSchema)",
    ],
  },
];

const severityStyle = {
  Critical: { border: "border-[#cc3a2a]", text: "text-[#cc3a2a]", bar: "#cc3a2a" },
  High:     { border: "border-[#c47040]", text: "text-[#c47040]", bar: "#c47040" },
  Medium:   { border: "border-[#c4a030]", text: "text-[#c4a030]", bar: "#c4a030" },
} as const;

const totalCritical = findings.filter((f) => f.severity === "Critical").length;
const totalHigh     = findings.filter((f) => f.severity === "High").length;
const totalMedium   = findings.filter((f) => f.severity === "Medium").length;

export default function ResultsPage() {
  const [activeId, setActiveId] = useState("01");
  const active = findings.find((f) => f.id === activeId) ?? findings[0];
  const sStyle = severityStyle[active.severity];

  return (
    <div className="relative flex h-screen flex-col overflow-hidden bg-[var(--background)] text-[var(--foreground)]">

      {/* ── Top header bar ── */}
      <header className="flex h-11 shrink-0 items-center border-b border-[var(--border)] px-5">
        <Link href="/" className="text-xl font-semibold tracking-[-0.04em] text-[var(--primary)]">
          argus
        </Link>

        <span className="mx-4 h-4 w-px bg-[var(--border)]" />

        <span className="text-[11px] text-[var(--muted-foreground)]">{findings.length} findings</span>

        <span className="mx-3 text-[11px] font-medium text-[#cc3a2a]">
          {totalCritical} CRITICAL
        </span>
        <span className="mr-3 text-[11px] font-medium text-[#c47040]">
          {totalHigh} HIGH
        </span>
        <span className="text-[11px] font-medium text-[#c4a030]">
          {totalMedium} MEDIUM
        </span>

        <div className="ml-auto flex items-center gap-4">
          <Link href="/scan" className="text-[11px] uppercase tracking-[0.4em] text-[var(--muted-foreground)] hover:text-[var(--foreground)] transition">
            New Scan
          </Link>
          <Link href="/scan" className="group relative overflow-hidden rounded-sm border border-[#644a40]/40 bg-[#644a40]/20 px-4 py-2 shadow-sm transition-all duration-300 hover:scale-105 hover:border-[#644a40]/70 hover:shadow-md hover:shadow-[#644a40]/20 active:scale-95">
            <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/30 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
            <div className="relative flex items-end gap-[3px]">
              {[10, 6, 14, 8, 12, 6, 10, 14].map((h, i) => (
                <span key={i} className="w-[3px] rounded-sm bg-[var(--muted-foreground)]/40" style={{ height: `${h}px` }} />
              ))}
            </div>
          </Link>
        </div>
      </header>

      {/* ── Two-panel body ── */}
      <div className="flex flex-1 overflow-hidden">

        {/* ── Left panel — findings list ── */}
        <aside className="flex w-[430px] shrink-0 flex-col overflow-y-auto border-r border-[var(--border)]">
          <div className="px-6 py-4">
            <p className="text-[10px] uppercase tracking-[0.5em] text-[var(--muted-foreground)]/50">Findings</p>
          </div>

          <div className="flex-1">
            {findings.map((f) => {
              const ss = severityStyle[f.severity];
              const isActive = f.id === activeId;
              return (
                <button
                  key={f.id}
                  onClick={() => setActiveId(f.id)}
                  className={`relative w-full border-t border-[var(--border)] px-6 py-5 text-left transition first:border-t-0 ${
                    isActive ? "bg-[var(--card)]" : "hover:bg-[var(--accent)]"
                  }`}
                >
                  {isActive && (
                    <div
                      className="absolute bottom-0 left-0 top-0 w-[3px]"
                      style={{ backgroundColor: ss.bar }}
                    />
                  )}

                  <div className="flex flex-wrap items-center gap-2">
                    <span className="font-mono text-[10px] text-[var(--muted-foreground)]/50">{f.id}</span>
                    <span className={`border px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest ${ss.border} ${ss.text}`}>
                      {f.severity}
                    </span>
                    <span className="border border-[var(--border)] px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest text-[var(--muted-foreground)]">
                      {f.framework}
                    </span>
                  </div>

                  <p className="mt-2 font-serif text-base italic leading-snug text-[var(--foreground)]">
                    {f.title}
                  </p>
                  <p className="mt-1.5 text-[12px] leading-5 text-[var(--muted-foreground)]">
                    {f.summary}
                  </p>
                  <p className="mt-2 font-mono text-[11px] text-[var(--muted-foreground)]/50">{f.file}</p>
                </button>
              );
            })}
          </div>
        </aside>

        {/* ── Right panel — code viewer ── */}
        <main className="flex flex-1 flex-col overflow-hidden">
          <div className="shrink-0 px-10 pt-8 pb-5">
            <h1 className="font-serif text-3xl italic leading-tight text-[var(--foreground)] sm:text-4xl">
              {active.title}
            </h1>
            <p className="mt-2 text-sm text-[var(--muted-foreground)]">{active.summary}</p>
          </div>

          {/* File bar */}
          <div className="flex shrink-0 items-center justify-between border-t border-b border-[var(--border)] bg-[var(--card)] px-10 py-3">
            <div className="flex items-center gap-2.5 text-[12px] text-[var(--muted-foreground)]">
              <AlignJustify className="size-3.5 shrink-0" />
              <span className="font-mono">{active.file}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="border border-[var(--border)] px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest text-[var(--muted-foreground)]">
                {active.framework}
              </span>
              <span className={`border px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest ${sStyle.border} ${sStyle.text}`}>
                {active.severity}
              </span>
            </div>
          </div>

          {/* Code area */}
          <div className="flex-1 overflow-auto bg-[var(--card)] px-0 py-6">
            <table className="w-full border-collapse font-mono text-sm">
              <tbody>
                {active.code.map((line, i) => {
                  const lineNum = i + 1;
                  const isHighlighted = lineNum === active.highlightLine;
                  return (
                    <tr key={i} className={isHighlighted ? "bg-[#cc3a2a]/8" : ""}>
                      <td className="w-14 select-none px-6 py-0.5 text-right text-[var(--muted-foreground)]/30 align-top">
                        {lineNum}
                      </td>
                      <td className={`px-4 py-0.5 text-[13px] leading-6 ${isHighlighted ? "text-[var(--foreground)]" : "text-[var(--muted-foreground)]"}`}>
                        <pre className="whitespace-pre">{line}</pre>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </main>
      </div>

      {/* ── FIX CODE button ── */}
      <button className="group fixed bottom-6 right-6 z-20 overflow-hidden rounded-full bg-[#644a40] px-8 py-4 text-[12px] font-medium uppercase tracking-[0.4em] text-[#ffe0c2] shadow-md transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-[#644a40]/40 active:scale-95">
        <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/40 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
        <span className="relative">Fix Code</span>
      </button>
    </div>
  );
}
