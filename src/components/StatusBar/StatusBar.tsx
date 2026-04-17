import { useEffect } from "react";
import { useRecordingState } from "../../hooks/useRecordingState";
import { useTheme } from "../../hooks/useTheme";
import { WaveformBars } from "./WaveformBars";

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
      className={`h-full w-full rounded-full flex items-center justify-center gap-2 px-3 text-white text-[11px] font-semibold select-none cursor-grab active:cursor-grabbing statusbar-state-${state || "IDLE"}`}
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
