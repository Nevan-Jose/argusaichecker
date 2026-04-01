export function AmbientBackground() {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none fixed inset-0 overflow-hidden bg-[#120d08]"
    >
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_18%_20%,rgba(214,177,112,0.16),transparent_32%),radial-gradient(circle_at_78%_16%,rgba(118,78,43,0.16),transparent_26%),radial-gradient(circle_at_52%_74%,rgba(86,57,32,0.24),transparent_38%)]" />
      <div className="absolute left-[12%] top-[14%] h-72 w-72 rounded-full bg-[#d4b27c]/10 blur-3xl" />
      <div className="absolute right-[14%] top-[28%] h-96 w-96 rounded-full bg-[#7b5533]/12 blur-3xl" />
      <div className="absolute bottom-[-12%] left-1/2 h-[32rem] w-[32rem] -translate-x-1/2 rounded-full bg-[#2b1b10]/70 blur-3xl" />
      <div className="absolute inset-0 bg-[linear-gradient(to_bottom,rgba(18,13,8,0.18),rgba(18,13,8,0.74))]" />
    </div>
  );
}
