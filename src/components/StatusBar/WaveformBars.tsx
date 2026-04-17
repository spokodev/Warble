import { useRef, useEffect } from "react";

interface WaveformBarsProps {
  level: number; // 0.0 - 1.0
  barCount?: number;
}

export function WaveformBars({ level, barCount = 5 }: WaveformBarsProps) {
  const barsRef = useRef<number[]>(Array(barCount).fill(0.15));

  useEffect(() => {
    // Distribute the level across bars with some variation
    const bars = barsRef.current;
    for (let i = 0; i < barCount; i++) {
      const centerWeight = 1 - Math.abs(i - (barCount - 1) / 2) / (barCount / 2);
      const target = Math.max(0.15, level * (0.6 + centerWeight * 0.4) + (Math.random() - 0.5) * level * 0.3);
      bars[i] = bars[i] * 0.6 + target * 0.4; // smooth
    }
  }, [level, barCount]);

  return (
    <div className="flex items-center gap-[2px] h-4">
      {barsRef.current.map((h, i) => (
        <div
          key={i}
          className="w-[2px] rounded-full bg-white/60 transition-[height] duration-75 ease-out"
          style={{ height: `${Math.max(15, h * 100)}%` }}
        />
      ))}
    </div>
  );
}
