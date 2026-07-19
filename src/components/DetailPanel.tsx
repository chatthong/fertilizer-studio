import { useEffect, useState } from "react";
import {
  loadDetail, NUTRIENT_KEYS, NUTRIENT_LABELS, CATEGORY_LABELS, type VariantDetail,
} from "../lib/library";

const fmt = (v: number | null | undefined, unit = "") =>
  v === null || v === undefined ? "—" : `${+v.toFixed(3)}${unit}`;

const INCOMPAT: { key: keyof VariantDetail["incompatibility"]; label: string }[] = [
  { key: "phosphates", label: "Avoid mixing with phosphates" },
  { key: "sulfates", label: "Avoid mixing with sulfates" },
  { key: "calcium", label: "Avoid mixing with calcium" },
  { key: "borate", label: "Avoid mixing with borate" },
  { key: "highPhStock", label: "High-pH stock — keep separate" },
];

export function DetailPanel({ variantId, onClose }: { variantId: string | null; onClose: () => void }) {
  const [detail, setDetail] = useState<VariantDetail | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!variantId) return;
    setLoading(true);
    setDetail(null);
    loadDetail(variantId).then((d) => setDetail(d)).finally(() => setLoading(false));
  }, [variantId]);

  if (!variantId) return null;

  const nutrients = detail
    ? NUTRIENT_KEYS.map((k) => ({ k, label: NUTRIENT_LABELS[k], v: detail.nutrients[k] })).filter((n) => n.v > 0)
    : [];
  const warnings = detail ? INCOMPAT.filter((w) => detail.incompatibility[w.key]) : [];

  return (
    <div className="fixed inset-0 z-40 flex justify-end">
      <div className="absolute inset-0 bg-black/30" onClick={onClose} />
      <aside className="relative z-10 w-[440px] max-w-[92vw] h-full overflow-y-auto bg-white shadow-xl dark:bg-neutral-950 border-l border-neutral-200 dark:border-neutral-800">
        <div className="sticky top-0 flex items-start gap-3 border-b border-neutral-200 bg-white/90 px-5 py-4 backdrop-blur dark:border-neutral-800 dark:bg-neutral-950/90">
          <div className="flex-1 min-w-0">
            <h2 className="text-base font-semibold tracking-tight truncate">
              {detail?.chemicalName ?? (loading ? "Loading…" : "—")}
            </h2>
            <p className="text-xs text-neutral-400">
              {[detail?.formula, detail?.code].filter(Boolean).join(" · ") || " "}
            </p>
          </div>
          <button
            onClick={onClose}
            className="grid place-items-center size-7 rounded-md text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800"
            aria-label="Close"
          >
            <svg viewBox="0 0 24 24" className="size-4" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M18 6 6 18M6 6l12 12" /></svg>
          </button>
        </div>

        {detail && (
          <div className="px-5 py-4 space-y-6 text-sm">
            {/* Variant meta */}
            <section className="grid grid-cols-2 gap-y-1.5 gap-x-4">
              <Meta label="Brand" value={detail.brandName} />
              <Meta label="Category" value={detail.category ? CATEGORY_LABELS[detail.category] ?? detail.category : null} />
              <Meta label="Manufacturer" value={detail.manufacturer} />
              <Meta label="Origin" value={detail.country} />
              <Meta label="Form" value={detail.formFactor} />
            </section>

            {detail.description && (
              <p className="text-neutral-500 dark:text-neutral-400 leading-relaxed">{detail.description}</p>
            )}

            {/* Incompatibility warnings */}
            {warnings.length > 0 && (
              <section className="space-y-1.5">
                {warnings.map((w) => (
                  <div key={w.key} className="flex items-center gap-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-1.5 text-xs text-amber-800 dark:border-amber-900/60 dark:bg-amber-950/30 dark:text-amber-300">
                    <svg viewBox="0 0 24 24" className="size-3.5 shrink-0" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z" /><path d="M12 9v4M12 17h.01" /></svg>
                    {w.label}
                  </div>
                ))}
              </section>
            )}

            {/* Nutrient profile */}
            <section>
              <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-neutral-400">Nutrient profile (% w/w)</h3>
              {nutrients.length === 0 ? (
                <p className="text-neutral-400 text-xs">No nutrient content recorded.</p>
              ) : (
                <div className="space-y-1.5">
                  {nutrients.map((n) => (
                    <div key={n.k} className="flex items-center gap-3">
                      <span className="w-16 shrink-0 text-neutral-500">{n.label}</span>
                      <div className="h-2 flex-1 rounded-full bg-neutral-100 dark:bg-neutral-800 overflow-hidden">
                        <div className="h-full rounded-full bg-emerald-500" style={{ width: `${Math.min(100, (n.v / 60) * 100)}%` }} />
                      </div>
                      <span className="w-12 shrink-0 text-right tabular-nums">{+n.v.toFixed(2)}</span>
                    </div>
                  ))}
                </div>
              )}
            </section>

            {/* Physical */}
            <section>
              <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-neutral-400">Physical</h3>
              <div className="grid grid-cols-2 gap-y-1.5 gap-x-4">
                <Meta label="Density" value={fmt(detail.physical.densityGCm3, " g/cm³")} raw />
                <Meta label="Solubility" value={fmt(detail.physical.solubilityGL25c, " g/L")} raw />
                <Meta label="EC (1 g/L)" value={fmt(detail.physical.ecMsCm1gL25c, " mS/cm")} raw />
                <Meta label="pH (1%)" value={fmt(detail.physical.ph1pct25c)} raw />
              </div>
            </section>

            {/* Pricing */}
            {detail.pricing.length > 0 && (
              <section>
                <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-neutral-400">Market pricing</h3>
                <div className="overflow-hidden rounded-md border border-neutral-200 dark:border-neutral-800">
                  <table className="w-full text-xs">
                    <thead className="bg-neutral-50 text-neutral-400 dark:bg-neutral-900/60">
                      <tr className="text-left">
                        <th className="px-3 py-1.5 font-medium">Pack</th>
                        <th className="px-3 py-1.5 font-medium">Market</th>
                        <th className="px-3 py-1.5 font-medium text-right">/ pack</th>
                        <th className="px-3 py-1.5 font-medium text-right">/ kg</th>
                      </tr>
                    </thead>
                    <tbody>
                      {detail.pricing.map((p, i) => (
                        <tr key={i} className="border-t border-neutral-100 dark:border-neutral-800/70">
                          <td className="px-3 py-1.5">{p.packSizeKg ?? "—"} kg</td>
                          <td className="px-3 py-1.5 text-neutral-500">{p.market ?? "—"}</td>
                          <td className="px-3 py-1.5 text-right tabular-nums">{p.pricePerPack != null ? `${p.currency ?? ""} ${p.pricePerPack}` : "—"}</td>
                          <td className="px-3 py-1.5 text-right tabular-nums">{p.pricePerKg != null ? `${p.currency ?? ""} ${+p.pricePerKg.toFixed(2)}` : "—"}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </section>
            )}
          </div>
        )}
      </aside>
    </div>
  );
}

function Meta({ label, value, raw }: { label: string; value: string | null; raw?: boolean }) {
  return (
    <div>
      <div className="text-[11px] text-neutral-400">{label}</div>
      <div className={raw ? "tabular-nums" : ""}>{value ?? "—"}</div>
    </div>
  );
}
