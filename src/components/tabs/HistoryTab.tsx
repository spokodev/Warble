import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry } from "../../lib/tauri-commands";

interface HistoryTabProps {
  entries: HistoryEntry[];
  setEntries: (e: HistoryEntry[]) => void;
  onClear: () => void;
  onUpdateEntry?: (idx: number, text: string) => void;
}

export function HistoryTab({ entries, setEntries, onClear, onUpdateEntry }: HistoryTabProps) {
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editText, setEditText] = useState("");
  const [copiedIdx, setCopiedIdx] = useState<number | null>(null);

  const handleCopy = async (text: string, idx: number) => {
    await invoke("copy_to_clipboard", { text });
    setCopiedIdx(idx);
    setTimeout(() => setCopiedIdx(null), 1500);
  };

  const handleStartEdit = (idx: number) => {
    setEditingIdx(idx);
    setEditText(entries[idx].text);
  };

  const handleSaveEdit = async (idx: number) => {
    await invoke("update_history_entry", { index: idx, newText: editText });
    const updated = [...entries];
    updated[idx] = { ...updated[idx], text: editText };
    setEntries(updated);
    setEditingIdx(null);
    onUpdateEntry?.(idx, editText);
  };

  const handleCancelEdit = () => {
    setEditingIdx(null);
    setEditText("");
  };

  return (
    <div className="flex flex-col h-full">
      {entries.length === 0 ? (
        <div className="p-8 text-center text-text-tertiary text-sm">
          No transcriptions yet. Press F5 or Right ⌘ to record.
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto divide-y divide-border-subtle">
          {entries.map((entry, idx) => (
            <div
              key={`${entry.timestamp}-${idx}`}
              className="px-4 py-3 hover:bg-surface-inset/50 transition-colors"
            >
              {editingIdx === idx ? (
                <div className="space-y-2">
                  <textarea
                    value={editText}
                    onChange={(e) => setEditText(e.target.value)}
                    rows={3}
                    className="input w-full text-sm"
                    autoFocus
                  />
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleSaveEdit(idx)}
                      className="btn-primary text-xs"
                    >
                      Save correction
                    </button>
                    <button
                      onClick={handleCancelEdit}
                      className="btn-ghost text-xs"
                    >
                      Cancel
                    </button>
                    {entry.datasetId && (
                      <span className="text-xs text-state-idle self-center ml-auto">
                        Updates training data
                      </span>
                    )}
                  </div>
                </div>
              ) : (
                <div>
                  <p className="text-sm text-text-primary break-words font-mono">
                    {entry.text}
                  </p>
                  <div className="flex items-center gap-3 mt-1.5">
                    <time className="text-xs text-text-tertiary">
                      {entry.timestamp}
                    </time>
                    <button
                      onClick={() => handleCopy(entry.text, idx)}
                      className="btn-ghost text-xs"
                    >
                      {copiedIdx === idx ? "Copied!" : "Copy"}
                    </button>
                    <button
                      onClick={() => handleStartEdit(idx)}
                      className="btn-ghost text-xs"
                    >
                      Edit
                    </button>
                    <button
                      onClick={async () => {
                        await invoke("delete_history_entry", { index: idx });
                        setEntries(entries.filter((_, i) => i !== idx));
                      }}
                      className="btn-ghost text-xs text-state-error hover:text-state-error"
                    >
                      Delete
                    </button>
                    {entry.datasetId && (
                      <span
                        className="text-xs text-text-tertiary"
                        title="Linked to training data"
                      >
                        dataset
                      </span>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
      {entries.length > 0 && (
        <div className="p-3 border-t border-border shrink-0">
          <button
            onClick={onClear}
            className="btn-ghost text-xs text-state-error hover:text-state-error"
          >
            Clear All
          </button>
        </div>
      )}
    </div>
  );
}
