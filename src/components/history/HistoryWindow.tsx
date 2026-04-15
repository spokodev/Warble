import { useEffect, useState } from "react";
import { getHistory, clearHistory, type HistoryEntry } from "../../lib/tauri-commands";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

export default function HistoryWindow() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [search, setSearch] = useState("");
  const [copiedIdx, setCopiedIdx] = useState<number | null>(null);

  const loadHistory = () => {
    getHistory().then(setEntries);
  };

  useEffect(() => {
    loadHistory();
    // Refresh history every 2 seconds when window is open
    const interval = setInterval(loadHistory, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleCopy = async (text: string, idx: number) => {
    await writeText(text);
    setCopiedIdx(idx);
    setTimeout(() => setCopiedIdx(null), 1500);
  };

  const handleClear = async () => {
    await clearHistory();
    setEntries([]);
  };

  const filtered = search
    ? entries.filter((e) =>
        e.text.toLowerCase().includes(search.toLowerCase())
      )
    : entries;

  return (
    <div className="flex flex-col h-screen bg-white text-sm">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 flex-shrink-0">
        <div className="flex items-center justify-between mb-3">
          <h1 className="text-lg font-semibold">History</h1>
          <span className="text-xs text-gray-400">
            {entries.length} item{entries.length !== 1 ? "s" : ""}
          </span>
        </div>
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search transcriptions..."
          className="w-full border border-gray-300 rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="p-8 text-center text-gray-400">
            {search ? "No matching transcriptions" : "No transcriptions yet"}
          </div>
        ) : (
          <div className="divide-y divide-gray-100">
            {filtered.map((entry, idx) => (
              <div
                key={`${entry.timestamp}-${idx}`}
                className="px-4 py-3 hover:bg-gray-50 group"
              >
                <div className="flex items-start justify-between gap-2">
                  <p className="text-gray-800 flex-1 break-words">
                    {entry.text}
                  </p>
                  <button
                    onClick={() => handleCopy(entry.text, idx)}
                    className="text-gray-400 hover:text-blue-500 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 text-xs"
                  >
                    {copiedIdx === idx ? "Copied!" : "Copy"}
                  </button>
                </div>
                <time className="text-xs text-gray-400 mt-1 block">
                  {entry.timestamp}
                </time>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      {entries.length > 0 && (
        <div className="p-3 border-t border-gray-200 flex-shrink-0">
          <button
            onClick={handleClear}
            className="text-xs text-red-500 hover:text-red-700"
          >
            Clear All History
          </button>
        </div>
      )}
    </div>
  );
}
