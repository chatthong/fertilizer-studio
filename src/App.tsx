import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type LibraryRow = {
  variantId: string;
  brandName: string | null;
  manufacturer: string | null;
  country: string | null;
  chemicalName: string | null;
  code: string | null;
  category: string | null;
  nNo3: number;
  nNh4: number;
  p2o5: number;
  k2o: number;
  ca: number;
  mg: number;
  s: number;
};

const pct = (v: number) => (v ? `${(+v.toFixed(2)).toString()}%` : "—");

function App() {
  const [rows, setRows] = useState<LibraryRow[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [q, setQ] = useState("");

  useEffect(() => {
    invoke<LibraryRow[]>("list_library").then(setRows).catch((e) => setError(String(e)));
  }, []);

  const filtered = (rows ?? []).filter((r) => {
    if (!q.trim()) return true;
    const hay = `${r.chemicalName ?? ""} ${r.brandName ?? ""} ${r.manufacturer ?? ""} ${r.code ?? ""} ${r.category ?? ""}`.toLowerCase();
    return hay.includes(q.toLowerCase());
  });

  return (
    <div className="min-h-full bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <header className="border-b border-neutral-200 dark:border-neutral-800 px-6 py-4 flex items-center gap-4">
        <div className="flex-1">
          <h1 className="text-lg font-semibold tracking-tight">Fertilizer Studio</h1>
          <p className="text-xs text-neutral-500 dark:text-neutral-400">
            Open fertilizer library · {rows ? `${rows.length} products` : "loading…"}
          </p>
        </div>
        <span className="text-[10px] uppercase tracking-wider rounded-full px-2 py-1 bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300">
          Local · offline
        </span>
      </header>

      <div className="px-6 py-4">
        <input
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="Search products, brands, categories…"
          className="w-full max-w-md rounded-md border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-emerald-500/40"
        />
      </div>

      <main className="px-6 pb-10">
        {error && (
          <div className="rounded-md border border-red-300 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300 px-4 py-3 text-sm">
            Failed to load library: {error}
          </div>
        )}
        {!error && rows === null && (
          <p className="text-sm text-neutral-500">Loading library…</p>
        )}
        {!error && rows !== null && (
          <div className="overflow-x-auto rounded-lg border border-neutral-200 dark:border-neutral-800">
            <table className="w-full text-sm border-collapse">
              <thead className="bg-neutral-100 dark:bg-neutral-900 text-neutral-500 dark:text-neutral-400">
                <tr className="text-left">
                  <th className="px-4 py-2 font-medium">Product</th>
                  <th className="px-4 py-2 font-medium">Brand</th>
                  <th className="px-4 py-2 font-medium">Origin</th>
                  <th className="px-4 py-2 font-medium">Category</th>
                  <th className="px-3 py-2 font-medium text-right">Total N</th>
                  <th className="px-3 py-2 font-medium text-right">P₂O₅</th>
                  <th className="px-3 py-2 font-medium text-right">K₂O</th>
                  <th className="px-3 py-2 font-medium text-right">Ca</th>
                  <th className="px-3 py-2 font-medium text-right">Mg</th>
                  <th className="px-3 py-2 font-medium text-right">S</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((r) => (
                  <tr
                    key={r.variantId}
                    className="border-t border-neutral-100 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-900/50"
                  >
                    <td className="px-4 py-2">
                      <div className="font-medium">{r.chemicalName ?? "—"}</div>
                      <div className="text-xs text-neutral-400">{r.code ?? ""}</div>
                    </td>
                    <td className="px-4 py-2">{r.brandName ?? "—"}</td>
                    <td className="px-4 py-2 text-neutral-500">{r.country ?? r.manufacturer ?? "—"}</td>
                    <td className="px-4 py-2">
                      <span className="rounded bg-neutral-100 dark:bg-neutral-800 px-2 py-0.5 text-xs">
                        {r.category ?? "—"}
                      </span>
                    </td>
                    <td className="px-3 py-2 text-right tabular-nums">{pct(r.nNo3 + r.nNh4)}</td>
                    <td className="px-3 py-2 text-right tabular-nums">{pct(r.p2o5)}</td>
                    <td className="px-3 py-2 text-right tabular-nums">{pct(r.k2o)}</td>
                    <td className="px-3 py-2 text-right tabular-nums">{pct(r.ca)}</td>
                    <td className="px-3 py-2 text-right tabular-nums">{pct(r.mg)}</td>
                    <td className="px-3 py-2 text-right tabular-nums">{pct(r.s)}</td>
                  </tr>
                ))}
                {filtered.length === 0 && (
                  <tr>
                    <td colSpan={10} className="px-4 py-8 text-center text-neutral-400">
                      No products match “{q}”.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
