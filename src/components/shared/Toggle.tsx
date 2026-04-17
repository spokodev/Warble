interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
}

export function Toggle({ checked, onChange, label }: ToggleProps) {
  return (
    <label className="flex items-center gap-3 cursor-pointer">
      <div
        className="toggle-switch"
        data-checked={checked}
        onClick={(e) => {
          e.preventDefault();
          onChange(!checked);
        }}
        role="switch"
        aria-checked={checked}
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault();
            onChange(!checked);
          }
        }}
      />
      <span className="text-sm text-text-primary">{label}</span>
    </label>
  );
}
