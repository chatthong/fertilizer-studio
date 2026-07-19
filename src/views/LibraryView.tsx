import { useEffect, useMemo, useState, type ReactNode } from "react";
import { loadLibrary, CATEGORY_LABELS, type LibraryRow } from "../lib/library";

const pct = (v: number) => (v ? `${+v.toFixed(1)}` : "·");

function Stat({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="rounded-lg border border-neutral-200 bg-white px-4 py-3 dark:border-neutral-800 dark:bg-neutral-900">
      <div className="text-2xl font-semibold tabular-nums tracking-tight">{value}</div>
      <div className="text-xs text-neutral-400">{label}</div>
    </div>
  );
}

// Small three-bar N-P-K glyph (Total N / P2O5 / K2O), scaled to 60% w/w.
function Npk({ n, p, k }: { n: number; p: number; k: number }) {
  const bar = (v: number, cls: string) => (
    <div className="h-1.5 w-8 rounded-full bg-neutral-100 dark:bg-neutral-800 overflow-hidden">
      <div className={`h-full ${cls}`} style={{ width: `${Math.min(100, (v / 60) * 100)}%` }} />
    </div>
  );
  return (
    <div className="flex items-center gap-1" title={`N ${n} · P₂O₅ ${p} · K₂O ${k}`}>
      {bar(n, "bg-emerald-500")}
      {bar(p, "bg-sky-500")}
      {bar(k, "bg-amber-500")}
    </div>
  );
}

export function LibraryView() {
  const [rows, setRows] = useState<LibraryRow[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [q, setQ] = useState("");
  const [cat, setCat] = useState<string | null>(null);

  useEffect(() => {
    loadLibrary().then(setRows).catch((e) => setError(String(e)));
  }, []);

  const categories = useMemo(() => {
    const set = new Map<string, number>();
    (rows ?? []).forEach((r) => r.category && set.set(r.category, (set.get(r.category) ?? 0) + 1));
    return [...set.entries()].sort((a, b) => b[1] - a[1]);
  }, [rows]);

  const stats = useMemo(() => {
    const list = rows ?? [];
    return {
      products: list.length,
      categories: new Set(list.map((r) => r.category).filter(Boolean)).size,
      countries: new Set(list.map((r) => r.country).filter(Boolean)).size,
      brands: new Set(list.map((r) => r.manufacturer).filter(Boolean)).size,
    };
  }, [rows]);

  const filtered = (rows ?? []).filter((r) => {
    if (cat && r.category !== cat) return false;
    if (!q.trim()) return true;
    const hay = `${r.chemicalName ?? ""} ${r.brandName ?? ""} ${r.manufacturer ?? ""} ${r.code ?? ""}`.toLowerCase();
    return hay.includes(q.toLowerCase());
  });

  return (
    <div className="flex flex-col h-full">
      {/* Top bar */}
      <header className="flex items-center gap-4 px-6 h-14 border-b border-neutral-200 dark:border-neutral-800 shrink-0">
        <div className="flex-1">
          <h1 className="text-base font-semibold tracking-tight">Library</h1>
          <p className="text-xs text-neutral-400">Open fertilizer catalog</p>
        </div>
        <div className="relative">
          <svg className="absolute left-2.5 top-2.5 size-4 text-neutral-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
          </svg>
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Search…"
            className="w-64 rounded-md border border-neutral-300 bg-white pl-8 pr-3 py-2 text-sm outline-none focus:ring-2 focus:ring-emerald-500/40 dark:border-neutral-700 dark:bg-neutral-900"
          />
        </div>
      </header>

      <div className="flex-1 overflow-auto px-6 py-5 space-y-5">
        {error && (
          <div className="rounded-md border border-red-300 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300">
            Failed to load library: {error}
          </div>
        )}

        {/* Stats */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <Stat label="Products" value={rows ? stats.products : "—"} />
          <Stat label="Categories" value={rows ? stats.categories : "—"} />
          <Stat label="Origins" value={rows ? stats.countries : "—"} />
          <Stat label="Sources" value={rows ? stats.brands : "—"} />
        </div>

        {/* Category chips */}
        {categories.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            <Chip active={cat === null} onClick={() => setCat(null)}>All</Chip>
            {categories.map(([c, n]) => (
              <Chip key={c} active={cat === c} onClick={() => setCat(cat === c ? null : c)}>
                {CATEGORY_LABELS[c] ?? c} <span className="opacity-50">{n}</span>
              </Chip>
            ))}
          </div>
        )}

        {/* Table */}
        <div className="overflow-hidden rounded-lg border border-neutral-200 dark:border-neutral-800">
          <table className="w-full text-sm">
            <thead className="bg-neutral-50 text-neutral-400 dark:bg-neutral-900/60">
              <tr className="text-left">
                <th className="px-4 py-2.5 font-medium">Product</th>
                <th className="px-4 py-2.5 font-medium">Brand</th>
                <th className="px-4 py-2.5 font-medium">Origin</th>
                <th className="px-4 py-2.5 font-medium">Category</th>
                <th className="px-4 py-2.5 font-medium">N·P·K</th>
                <th className="px-3 py-2.5 font-medium text-right">N</th>
                <th className="px-3 py-2.5 font-medium text-right">P₂O₅</th>
                <th className="px-3 py-2.5 font-medium text-right">K₂O</th>
                <th className="px-3 py-2.5 font-medium text-right">Ca</th>
                <th className="px-3 py-2.5 font-medium text-right">Mg</th>
                <th className="px-3 py-2.5 font-medium text-right">S</th>
              </tr>
            </thead>
            <tbody>
              {(rows === null && !error) && (
                <tr><td colSpan={11} className="px-4 py-10 text-center text-neutral-400">Loading library…</td></tr>
              )}
              {filtered.map((r) => (
                <tr key={r.variantId} className="border-t border-neutral-100 hover:bg-neutral-50 dark:border-neutral-800/70 dark:hover:bg-neutral-900/40">
                  <td className="px-4 py-2.5">
                    <div className="font-medium text-neutral-800 dark:text-neutral-100">{r.chemicalName ?? "—"}</div>
                    <div className="text-xs text-neutral-400">{r.code ?? ""}</div>
                  </td>
                  <td className="px-4 py-2.5 text-neutral-600 dark:text-neutral-300">{r.brandName ?? "—"}</td>
                  <td className="px-4 py-2.5 text-neutral-500">{r.country ?? "—"}</td>
                  <td className="px-4 py-2.5">
                    <span className="rounded-full bg-neutral-100 px-2 py-0.5 text-xs text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">
                      {CATEGORY_LABELS[r.category ?? ""] ?? r.category ?? "—"}
                    </span>
                  </td>
                  <td className="px-4 py-2.5"><Npk n={r.nNo3 + r.nNh4} p={r.p2o5} k={r.k2o} /></td>
                  <td className="px-3 py-2.5 text-right tabular-nums">{pct(r.nNo3 + r.nNh4)}</td>
                  <td className="px-3 py-2.5 text-right tabular-nums">{pct(r.p2o5)}</td>
                  <td className="px-3 py-2.5 text-right tabular-nums">{pct(r.k2o)}</td>
                  <td className="px-3 py-2.5 text-right tabular-nums text-neutral-500">{pct(r.ca)}</td>
                  <td className="px-3 py-2.5 text-right tabular-nums text-neutral-500">{pct(r.mg)}</td>
                  <td className="px-3 py-2.5 text-right tabular-nums text-neutral-500">{pct(r.s)}</td>
                </tr>
              ))}
              {rows !== null && filtered.length === 0 && (
                <tr><td colSpan={11} className="px-4 py-10 text-center text-neutral-400">No products match your filters.</td></tr>
              )}
            </tbody>
          </table>
        </div>
        <p className="text-xs text-neutral-400">
          Showing {filtered.length} of {rows?.length ?? 0} · values are % w/w · seeded from the open library
        </p>
      </div>
    </div>
  );
}

function Chip({ active, onClick, children }: { active: boolean; onClick: () => void; children: ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={[
        "rounded-full px-3 py-1 text-xs transition-colors",
        active
          ? "bg-emerald-600 text-white"
          : "bg-neutral-100 text-neutral-600 hover:bg-neutral-200 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700",
      ].join(" ")}
    >
      {children}
    </button>
  );
}
