import { invoke } from "@tauri-apps/api/core";

export const NUTRIENT_KEYS = [
  "nNo3", "nNh4", "p2o5", "k2o", "ca", "mg", "s",
  "fe", "mn", "zn", "b", "cu", "si", "mo", "na", "cl",
] as const;
export type NutrientKey = (typeof NUTRIENT_KEYS)[number];
export type Nutrients = Record<NutrientKey, number>;

export type LibraryRow = {
  variantId: string;
  brandName: string | null;
  manufacturer: string | null;
  country: string | null;
  chemicalName: string | null;
  code: string | null;
  category: string | null;
  nNo3: number; nNh4: number; p2o5: number; k2o: number; ca: number; mg: number; s: number;
};

export type PriceRow = {
  packSizeKg: number | null;
  market: string | null;
  currency: string | null;
  pricePerPack: number | null;
  pricePerKg: number | null;
};

export type VariantDetail = {
  variantId: string;
  brandName: string | null;
  manufacturer: string | null;
  country: string | null;
  formFactor: string | null;
  chemicalName: string | null;
  code: string | null;
  description: string | null;
  formula: string | null;
  category: string | null;
  nutrients: Nutrients;
  physical: {
    densityGCm3: number | null;
    solubilityGL25c: number | null;
    ecMsCm1gL25c: number | null;
    ph1pct25c: number | null;
  };
  incompatibility: {
    phosphates: boolean; sulfates: boolean; calcium: boolean; borate: boolean; highPhStock: boolean;
  };
  pricing: PriceRow[];
};

const inTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

let rawCache: RawLibrary | null = null;
async function raw(): Promise<RawLibrary> {
  if (!rawCache) rawCache = ((await import("../../library/library.json")).default as unknown) as RawLibrary;
  return rawCache;
}

/** Browse rows: SQLite via `list_library` in-app, else joined from library.json. */
export async function loadLibrary(): Promise<LibraryRow[]> {
  if (inTauri()) return invoke<LibraryRow[]>("list_library");
  return joinRows(await raw());
}

/** Full detail for one variant: `variant_detail` in-app, else built from library.json. */
export async function loadDetail(variantId: string): Promise<VariantDetail | null> {
  if (inTauri()) return invoke<VariantDetail | null>("variant_detail", { variantId });
  const data = await raw();
  const v = data.variants.find((x) => x.id === variantId);
  if (!v) return null;
  const c = data.canonicals.find((x) => x.id === v.canonicalId);
  if (!c) return null;
  const n = c.nutrients ?? {};
  const nutrients = Object.fromEntries(NUTRIENT_KEYS.map((k) => [k, num(n[k])])) as Nutrients;
  return {
    variantId: v.id,
    brandName: v.brandName, manufacturer: v.manufacturer, country: v.country, formFactor: v.formFactor ?? null,
    chemicalName: c.chemicalName, code: c.code, description: c.description ?? null, formula: c.formula ?? null,
    category: c.category, nutrients,
    physical: {
      densityGCm3: c.physical?.densityGCm3 ?? null,
      solubilityGL25c: c.physical?.solubilityGL25c ?? null,
      ecMsCm1gL25c: c.physical?.ecMsCm1gL25c ?? null,
      ph1pct25c: c.physical?.ph1pct25c ?? null,
    },
    incompatibility: {
      phosphates: !!c.incompatibility?.phosphates, sulfates: !!c.incompatibility?.sulfates,
      calcium: !!c.incompatibility?.calcium, borate: !!c.incompatibility?.borate,
      highPhStock: !!c.incompatibility?.highPhStock,
    },
    pricing: data.pricing.filter((p) => p.variantId === variantId).map((p) => ({
      packSizeKg: p.packSizeKg ?? null, market: p.market ?? null, currency: p.currency ?? null,
      pricePerPack: p.pricePerPack ?? null, pricePerKg: p.pricePerKg ?? null,
    })),
  };
}

const num = (v: number | null | undefined) => (typeof v === "number" ? v : 0);

type RawLibrary = {
  canonicals: Array<{
    id: string; code: string | null; chemicalName: string | null; description?: string | null;
    formula?: string | null; category: string | null;
    nutrients: Partial<Record<NutrientKey, number | null>>;
    physical?: { densityGCm3?: number | null; solubilityGL25c?: number | null; ecMsCm1gL25c?: number | null; ph1pct25c?: number | null };
    incompatibility?: { phosphates?: boolean; sulfates?: boolean; calcium?: boolean; borate?: boolean; highPhStock?: boolean };
  }>;
  variants: Array<{ id: string; canonicalId: string; brandName: string | null; manufacturer: string | null; country: string | null; formFactor?: string | null }>;
  pricing: Array<{ id: string; variantId: string; packSizeKg?: number | null; market?: string | null; currency?: string | null; pricePerPack?: number | null; pricePerKg?: number | null }>;
};

function joinRows(data: RawLibrary): LibraryRow[] {
  const byId = new Map(data.canonicals.map((c) => [c.id, c]));
  return data.variants
    .map((v): LibraryRow | null => {
      const c = byId.get(v.canonicalId);
      if (!c) return null;
      const n = c.nutrients ?? {};
      return {
        variantId: v.id, brandName: v.brandName, manufacturer: v.manufacturer, country: v.country,
        chemicalName: c.chemicalName, code: c.code, category: c.category,
        nNo3: num(n.nNo3), nNh4: num(n.nNh4), p2o5: num(n.p2o5), k2o: num(n.k2o),
        ca: num(n.ca), mg: num(n.mg), s: num(n.s),
      };
    })
    .filter((r): r is LibraryRow => r !== null)
    .sort((a, b) =>
      (a.chemicalName ?? "").localeCompare(b.chemicalName ?? "") ||
      (a.brandName ?? "").localeCompare(b.brandName ?? ""));
}

export const CATEGORY_LABELS: Record<string, string> = {
  raw_salt: "Raw salt", single_nutrient: "Single nutrient", compound: "Compound",
  trace_mix: "Trace mix", liquid_concentrate: "Liquid", granular_blend: "Granular blend", chelate: "Chelate",
};

export const NUTRIENT_LABELS: Record<NutrientKey, string> = {
  nNo3: "N (NO₃)", nNh4: "N (NH₄)", p2o5: "P₂O₅", k2o: "K₂O", ca: "Ca", mg: "Mg", s: "S",
  fe: "Fe", mn: "Mn", zn: "Zn", b: "B", cu: "Cu", si: "Si", mo: "Mo", na: "Na", cl: "Cl",
};
