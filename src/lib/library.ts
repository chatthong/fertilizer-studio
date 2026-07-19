import { invoke } from "@tauri-apps/api/core";

export type LibraryRow = {
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

const inTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Load the library. In the Tauri app this reads SQLite via the `list_library`
 * command; in a plain browser (design/preview) it falls back to joining the
 * bundled shared `library.json` client-side so the UI renders the same data.
 */
export async function loadLibrary(): Promise<LibraryRow[]> {
  if (inTauri()) {
    return invoke<LibraryRow[]>("list_library");
  }
  const data = (await import("../../library/library.json")).default as RawLibrary;
  return joinRows(data);
}

type RawLibrary = {
  canonicals: Array<{
    id: string;
    code: string | null;
    chemicalName: string | null;
    category: string | null;
    nutrients: Partial<Record<"nNo3" | "nNh4" | "p2o5" | "k2o" | "ca" | "mg" | "s", number | null>>;
  }>;
  variants: Array<{
    id: string;
    canonicalId: string;
    brandName: string | null;
    manufacturer: string | null;
    country: string | null;
  }>;
};

function joinRows(data: RawLibrary): LibraryRow[] {
  const byId = new Map(data.canonicals.map((c) => [c.id, c]));
  const num = (v: number | null | undefined) => (typeof v === "number" ? v : 0);
  return data.variants
    .map((v): LibraryRow | null => {
      const c = byId.get(v.canonicalId);
      if (!c) return null; // mirror the SQL inner join
      const n = c.nutrients ?? {};
      return {
        variantId: v.id,
        brandName: v.brandName,
        manufacturer: v.manufacturer,
        country: v.country,
        chemicalName: c.chemicalName,
        code: c.code,
        category: c.category,
        nNo3: num(n.nNo3),
        nNh4: num(n.nNh4),
        p2o5: num(n.p2o5),
        k2o: num(n.k2o),
        ca: num(n.ca),
        mg: num(n.mg),
        s: num(n.s),
      };
    })
    .filter((r): r is LibraryRow => r !== null)
    .sort(
      (a, b) =>
        (a.chemicalName ?? "").localeCompare(b.chemicalName ?? "") ||
        (a.brandName ?? "").localeCompare(b.brandName ?? ""),
    );
}

export const CATEGORY_LABELS: Record<string, string> = {
  raw_salt: "Raw salt",
  single_nutrient: "Single nutrient",
  compound: "Compound",
  trace_mix: "Trace mix",
  liquid_concentrate: "Liquid",
  granular_blend: "Granular blend",
  chelate: "Chelate",
};
