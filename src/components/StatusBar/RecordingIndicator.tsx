export function RecordingIndicator() {
  return (
    <div className="relative w-6 h-6 flex items-center justify-center">
      {/* Center dot */}
      <div className="w-1.5 h-1.5 rounded-full bg-white animate-warble-pulse z-10" />
      {/* Ring 1 */}
      <div className="absolute inset-0 rounded-full border border-white/40 animate-warble-ring" />
      {/* Ring 2 */}
      <div
        className="absolute inset-0 rounded-full border border-white/25 animate-warble-ring"
        style={{ animationDelay: "0.15s" }}
      />
      {/* Ring 3 */}
      <div
        className="absolute inset-0 rounded-full border border-white/10 animate-warble-ring"
        style={{ animationDelay: "0.3s" }}
      />
    </div>
  );
}
