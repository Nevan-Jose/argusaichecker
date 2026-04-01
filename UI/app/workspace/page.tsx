import Link from "next/link";
import { ArrowLeft, ShieldCheck } from "lucide-react";

export default function WorkspacePage() {
  return (
    <main className="min-h-screen bg-[#04090c] px-6 py-16 text-[#e8fbf8] sm:px-10">
      <div className="mx-auto max-w-5xl">
        <Link
          href="/"
          className="inline-flex items-center gap-2 rounded-full border border-cyan-400/12 bg-cyan-300/[0.05] px-4 py-2 text-sm text-[#d3fbf5] transition hover:border-cyan-300/22 hover:bg-cyan-300/[0.08]"
        >
          <ArrowLeft className="size-4" />
          Back to landing
        </Link>

        <div className="mt-16 rounded-[2rem] border border-cyan-400/10 bg-[#071118]/58 p-10 backdrop-blur-xl">
          <ShieldCheck className="size-8 text-[#6ce6d0]" />
          <h1 className="mt-6 text-4xl font-semibold tracking-tight text-[#eefdfd]">
            Argus Workspace
          </h1>
          <p className="mt-4 max-w-2xl text-base leading-8 text-[#b3d1cf]">
            Placeholder workspace route for the 3D pin interaction. The landing
            page component now has a real destination instead of a dummy link.
          </p>
        </div>
      </div>
    </main>
  );
}
