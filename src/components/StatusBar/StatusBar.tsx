import { useEffect } from "react";
import { useRecordingState } from "../../hooks/useRecordingState";
import { useTheme } from "../../hooks/useTheme";
import { WaveformBars } from "./WaveformBars";

const STATE_COLORS: Record<string, string> = {
  RECORDING: "rgba(239, 68, 68, 0.92)",
  TRANSCRIBING: "rgba(245, 158, 11, 0.88)",
  STOPPING: "rgba(234, 179, 8, 0.88)",
  IDLE: "rgba(34, 197, 94, 0.8)",
};

const STATE_LABELS: Record<string, string> = {
  RECORDING: "REC",
  TRANSCRIBING: "Transcribing...",
  STOPPING: "Stopping...",
  IDLE: "Ready",
};

export function StatusBar() {
  useTheme();
  const { state, audioLevel } = useRecordingState();

  useEffect(() => {
    document.documentElement.classList.add("status-bar-window");
  }, []);

  const isRecording = state === "RECORDING";
  const isTranscribing = state === "TRANSCRIBING";

  const handleMouseDown = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    getCurrentWindow().startDragging();
  };

  return (
    <div
      onMouseDown={handleMouseDown}
      className="h-full w-full rounded-full flex items-center justify-center gap-2 px-3 text-white text-[11px] font-semibold select-none shadow-lg cursor-grab active:cursor-grabbing"
      style={{
        background: STATE_COLORS[state] || STATE_COLORS.IDLE,
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
        border: "1px solid rgba(255, 255, 255, 0.2)",
      }}
    >
      {/* Indicator dot — always present, animated per state */}
      <div className="relative flex items-center justify-center w-3 h-3 shrink-0">
        <div
          className={`w-2 h-2 rounded-full bg-white ${isRecording ? "animate-warble-pulse" : ""}`}
        />
        {isRecording && (
          <>
            <div className="absolute inset-0 rounded-full border border-white/40 animate-warble-ring" />
            <div
              className="absolute inset-0 rounded-full border border-white/20 animate-warble-ring"
              style={{ animationDelay: "0.3s" }}
            />
          </>
        )}
      </div>

      {/* Waveform — only during recording */}
      {isRecording && <WaveformBars level={audioLevel} />}

      {/* Label — always present */}
      <span className={isTranscribing ? "animate-pulse" : ""}>
        {STATE_LABELS[state] || "Ready"}
      </span>
    </div>
  );
}
