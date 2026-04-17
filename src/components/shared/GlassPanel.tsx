interface GlassPanelProps {
  children: React.ReactNode;
  className?: string;
  elevated?: boolean;
}

export function GlassPanel({
  children,
  className = "",
  elevated = false,
}: GlassPanelProps) {
  return (
    <div
      className={`${elevated ? "glass-panel-elevated" : "glass-panel"} rounded-xl ${className}`}
    >
      {children}
    </div>
  );
}
