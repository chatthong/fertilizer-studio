import type { ComponentType } from "react";

type IconProps = { className?: string };

const Leaf = ({ className }: IconProps) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z" />
    <path d="M2 21c0-3 1.85-5.36 5.08-6" />
  </svg>
);
const Beaker = ({ className }: IconProps) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4.5 3h15M6 3v7.5L3.5 18a2 2 0 0 0 1.8 3h13.4a2 2 0 0 0 1.8-3L18 10.5V3" />
    <path d="M6.5 12h11" />
  </svg>
);
const Calendar = ({ className }: IconProps) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="4" width="18" height="18" rx="2" /><path d="M16 2v4M8 2v4M3 10h18" />
  </svg>
);
const Package = ({ className }: IconProps) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <path d="m7.5 4.27 9 5.15M21 8l-9 5-9-5 9-5 9 5Z" /><path d="M3 8v8l9 5 9-5V8M12 13v9" />
  </svg>
);

type NavKey = "library" | "formulas" | "feeding" | "inventory";

const NAV: { key: NavKey; label: string; icon: ComponentType<IconProps>; ready: boolean }[] = [
  { key: "library", label: "Library", icon: Leaf, ready: true },
  { key: "formulas", label: "Formulas", icon: Beaker, ready: false },
  { key: "feeding", label: "Feeding", icon: Calendar, ready: false },
  { key: "inventory", label: "Inventory", icon: Package, ready: false },
];

export function Sidebar({ active }: { active: NavKey }) {
  return (
    <aside className="w-60 shrink-0 flex flex-col border-r border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
      <div className="flex items-center gap-2.5 px-4 h-14 border-b border-neutral-200 dark:border-neutral-800">
        <div className="grid place-items-center size-8 rounded-lg bg-emerald-600 text-white">
          <Leaf className="size-4.5" />
        </div>
        <div className="leading-tight">
          <div className="text-sm font-semibold tracking-tight">Fertilizer Studio</div>
          <div className="text-[10px] text-neutral-400">v0.1 · local</div>
        </div>
      </div>

      <nav className="flex-1 p-2 space-y-0.5">
        {NAV.map(({ key, label, icon: Icon, ready }) => {
          const isActive = key === active;
          return (
            <button
              key={key}
              disabled={!ready}
              className={[
                "w-full flex items-center gap-2.5 rounded-md px-2.5 py-2 text-sm transition-colors",
                isActive
                  ? "bg-emerald-50 text-emerald-700 font-medium dark:bg-emerald-950/50 dark:text-emerald-300"
                  : ready
                    ? "text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
                    : "text-neutral-400 dark:text-neutral-600 cursor-default",
              ].join(" ")}
            >
              <Icon className="size-4.5 shrink-0" />
              <span className="flex-1 text-left">{label}</span>
              {!ready && (
                <span className="text-[9px] uppercase tracking-wide rounded bg-neutral-100 px-1.5 py-0.5 text-neutral-400 dark:bg-neutral-800">
                  soon
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <div className="p-3 border-t border-neutral-200 dark:border-neutral-800">
        <div className="flex items-center gap-2 text-[11px] text-neutral-400">
          <span className="size-1.5 rounded-full bg-emerald-500" />
          Local · offline · no account
        </div>
      </div>
    </aside>
  );
}
