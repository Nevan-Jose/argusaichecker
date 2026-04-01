"use client";

import { useEffect, useRef, useState, type DragEventHandler } from "react";
import { useRouter } from "next/navigation";
import { FolderUp, FileUp } from "lucide-react";

type UploadSummary = {
  projectName: string;
  fileCount: number;
  totalSize: number;
  topFiles: string[];
};

const SESSION_KEY = "argus-analysis-session";

const fallbackSummary: UploadSummary = {
  projectName: "argus-demo-repo",
  fileCount: 248,
  totalSize: 18_900_000,
  topFiles: ["src/config/aws.ts", "src/routes/users.ts", "src/services/auth.ts", "src/middleware/session.ts"],
};

function summarizeFiles(files: File[]): UploadSummary {
  const first = files[0];
  return {
    projectName: first?.webkitRelativePath?.split("/")[0] || first?.name.replace(/\.[^.]+$/, "") || "sample-project",
    fileCount: files.length,
    totalSize: files.reduce((sum, f) => sum + f.size, 0),
    topFiles: files.slice(0, 4).map((f) => f.name),
  };
}

export function ProjectUploadPanel() {
  const router = useRouter();
  const folderInputRef  = useRef<HTMLInputElement>(null);
  const fileInputRef    = useRef<HTMLInputElement>(null);
  const [files, setFiles]       = useState<File[]>([]);
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    folderInputRef.current?.setAttribute("webkitdirectory", "");
    folderInputRef.current?.setAttribute("directory", "");
  }, []);

  const persist = (nextFiles: File[]) => {
    const s = nextFiles.length ? summarizeFiles(nextFiles) : fallbackSummary;
    sessionStorage.setItem(SESSION_KEY, JSON.stringify({ ...s, createdAt: Date.now() }));
  };

  const handleSelection = (list: FileList | null) => {
    if (!list) return;
    const next = Array.from(list);
    setFiles(next);
    persist(next);
  };

  const handleDrop: DragEventHandler<HTMLDivElement> = (e) => {
    e.preventDefault();
    setIsDragging(false);
    const next = Array.from(e.dataTransfer.files);
    if (!next.length) return;
    setFiles(next);
    persist(next);
  };

  const startScan = () => {
    persist(files);
    router.push("/scan");
  };

  const hasFiles = files.length > 0;

  return (
    <div className="flex w-full flex-col items-center gap-6">
      {/* Drop zone */}
      <div
        className={`relative w-full max-w-lg rounded-2xl border-2 border-dashed p-14 text-center transition cursor-pointer ${
          isDragging
            ? "border-[var(--primary)] bg-[var(--primary)]/5"
            : "border-[var(--border)] hover:border-[var(--primary)]/40 hover:bg-[var(--card)]"
        }`}
        onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
        onDragEnter={() => setIsDragging(true)}
        onDragLeave={() => setIsDragging(false)}
        onDrop={handleDrop}
        onClick={() => folderInputRef.current?.click()}
      >
        <div className="flex flex-col items-center gap-4">
          <div className="rounded-2xl border border-[var(--border)] bg-[var(--card)] p-4">
            <FolderUp className="size-7 text-[var(--primary)]" />
          </div>

          {hasFiles ? (
            <>
              <p className="text-base font-medium text-[var(--foreground)]">
                {files.length} file{files.length !== 1 ? "s" : ""} selected
              </p>
              <p className="text-sm text-[var(--muted-foreground)]">
                {summarizeFiles(files).projectName}
              </p>
            </>
          ) : (
            <>
              <p className="text-base font-medium text-[var(--foreground)]">
                Drop your project here
              </p>
              <p className="text-sm text-[var(--muted-foreground)]">
                Folder, archive, or individual files
              </p>
            </>
          )}

          <div className="flex gap-3" onClick={(e) => e.stopPropagation()}>
            <button
              type="button"
              onClick={() => folderInputRef.current?.click()}
              className="group relative overflow-hidden inline-flex items-center gap-2 rounded-full border border-[#644a40]/40 bg-[#644a40]/20 px-4 py-2 text-sm text-[#ffe0c2] shadow-sm transition-all duration-300 hover:scale-105 hover:border-[#644a40]/70 hover:bg-[#644a40]/30 hover:shadow-md hover:shadow-[#644a40]/20 active:scale-95"
            >
              <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/30 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
              <FolderUp className="relative size-4" />
              <span className="relative">Folder</span>
            </button>
            <button
              type="button"
              onClick={() => fileInputRef.current?.click()}
              className="group relative overflow-hidden inline-flex items-center gap-2 rounded-full border border-[#644a40]/40 bg-[#644a40]/20 px-4 py-2 text-sm text-[#ffe0c2] shadow-sm transition-all duration-300 hover:scale-105 hover:border-[#644a40]/70 hover:bg-[#644a40]/30 hover:shadow-md hover:shadow-[#644a40]/20 active:scale-95"
            >
              <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/30 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
              <FileUp className="relative size-4" />
              <span className="relative">File</span>
            </button>
          </div>
        </div>

        <input ref={folderInputRef} type="file" className="hidden" multiple onChange={(e) => handleSelection(e.target.files)} />
        <input ref={fileInputRef}   type="file" className="hidden" accept=".zip,.tar,.gz,.ts,.js,.py,.go" multiple onChange={(e) => handleSelection(e.target.files)} />
      </div>

      {/* Start Scan button */}
      <button
        type="button"
        onClick={startScan}
        className="group relative overflow-hidden rounded-full bg-[#644a40] px-10 py-3.5 text-sm font-medium text-[#ffe0c2] shadow-md transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-[#644a40]/40 active:scale-95"
      >
        <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-[#ffdfb5]/40 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
        <span className="relative">Start Scan</span>
      </button>
    </div>
  );
}
