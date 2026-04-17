import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { RecordingState } from "../lib/tauri-commands";

export function useRecordingState() {
  const [state, setState] = useState<RecordingState>("IDLE");
  const [audioLevel, setAudioLevel] = useState(0);

  useEffect(() => {
    invoke<RecordingState>("get_state").then(setState);

    const unsubs: (() => void)[] = [];
    listen<RecordingState>("state-changed", (e) => {
      setState(e.payload);
      if (e.payload !== "RECORDING") {
        setAudioLevel(0);
      }
    }).then((u) => unsubs.push(u));

    listen<number>("audio-level", (e) => {
      setAudioLevel(e.payload);
    }).then((u) => unsubs.push(u));

    return () => unsubs.forEach((u) => u());
  }, []);

  return { state, audioLevel };
}
