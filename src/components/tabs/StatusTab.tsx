import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { RecordingState } from "../../lib/tauri-commands";

interface StatusTabProps {
  state: RecordingState;
  lastText: string;
  lastError: string;
  apiKeySet: boolean;
  onUpdateLastText: (text: string) => void;
}

export function StatusTab({
  state,
  lastText,
  lastError,
  apiKeySet,
  onUpdateLastText,
}: StatusTabProps) {
  const [editing, setEditing] = useState(false);
  const [editText, setEditText] = useState("");

  const handleStartEdit = () => {
    setEditText(lastText);
    setEditing(true);
  };

  const handleSave = async () => {
    await invoke("update_history_entry", { index: 0, newText: editText });
    onUpdateLastText(editText);
    setEditing(false);
  };

  return (
    <div className="p-4 space-y-4 text-sm">
      {!apiKeySet && (
        <div className="state-banner" data-state="error">
          API key not set — go to Settings tab.
        </div>
      )}

      <div className="glass-panel rounded-lg p-3 space-y-1 text-xs text-text-secondary">
        <p>
          <kbd className="kbd">Right ⌘</kbd> — tap to start, release to stop
        </p>
        <p>
          <kbd className="kbd">F5</kbd> — toggle recording
        </p>
        <p className="text-text-tertiary mt-2">
          Requires Accessibility permission for Right ⌘
        </p>
      </div>

      {state === "RECORDING" && (
        <div className="text-xs text-state-recording flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-state-recording animate-warble-pulse" />
          Recording...
        </div>
      )}

      {state === "TRANSCRIBING" && (
        <div className="state-banner" data-state="transcribing">
          Transcribing audio...
        </div>
      )}

      {lastError && (
        <div className="state-banner text-xs" data-state="error">
          {lastError}
        </div>
      )}

      {lastText && (
        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-text-tertiary">
              Last transcription:
            </span>
            <div className="flex gap-2">
              <button onClick={() => invoke("copy_to_clipboard", { text: lastText })} className="btn-ghost">
                Copy
              </button>
              {!editing && (
                <button onClick={handleStartEdit} className="btn-ghost">
                  Edit
                </button>
              )}
            </div>
          </div>
          {editing ? (
            <div className="space-y-2">
              <textarea
                value={editText}
                onChange={(e) => setEditText(e.target.value)}
                rows={3}
                className="input w-full font-mono text-xs"
                autoFocus
              />
              <div className="flex gap-2">
                <button onClick={handleSave} className="btn-primary text-xs">
                  Save correction
                </button>
                <button onClick={() => setEditing(false)} className="btn-ghost text-xs">
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div
              className="glass-panel rounded-lg p-3 text-text-primary break-words select-text cursor-text font-mono text-xs"
              onDoubleClick={handleStartEdit}
            >
              {lastText}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
