#!/usr/bin/env node
/**
 * Extract the shared fertilizer library from a CropBinary Postgres plain-SQL dump
 * into library/library.json — the app seed AND the GitHub-published open library.
 *
 * Usage: node tools/extract-library.cjs <dump.sql> [out.json]
 *
 * Fidelity rules (see docs/audit-legacy-vs-new.md):
 *  - Variant nutrients = canonical base with nutrient-scope overrides APPLIED, matching
 *    legacy buildResolvedNutrients() which applies overrides regardless of status.
 *  - Archived variants (archived_at set) are excluded.
 *  - Carries aliases, CAS, assay, hazard, packaging, and full pricing provenance.
 */
const fs = require("fs");

const DUMP = process.argv[2];
const OUT = process.argv[3] || require("path").join(__dirname, "..", "library", "library.json");
if (!DUMP) {
  console.error("usage: node tools/extract-library.cjs <dump.sql> [out.json]");
  process.exit(1);
}
const lines = fs.readFileSync(DUMP, "utf8").split("\n");

const unesc = (f) =>
  f === "\\N" ? null : f.replace(/\\t/g, "\t").replace(/\\n/g, "\n").replace(/\\r/g, "\r").replace(/\\\\/g, "\\");

function table(name) {
  const rows = [];
  let cols = null, inBlock = false;
  const re = new RegExp(`^COPY (?:public\\.)?"?${name}"?\\s*\\(([^)]+)\\) FROM stdin;`);
  for (const line of lines) {
    if (!inBlock) {
      const m = line.match(re);
      if (m) { cols = m[1].split(",").map((c) => c.trim().replace(/^"|"$/g, "")); inBlock = true; }
      continue;
    }
    if (line === "\\.") break;
    const f = line.split("\t").map(unesc);
    const o = {};
    cols.forEach((c, i) => (o[c] = f[i]));
    rows.push(o);
  }
  return rows;
}

const num = (v) => (v === null || v === undefined || v === "" ? null : Number(v));
const bool = (v) => v === "t" || v === "true";
// Postgres text array literal -> JS array, e.g. {"OMRI",EU} -> ["OMRI","EU"]
const arr = (v) => {
  if (!v || v === "{}" || v === "\\N") return [];
  const inner = v.replace(/^\{|\}$/g, "");
  if (!inner) return [];
  return inner.split(",").map((s) => s.replace(/^"|"$/g, "").trim()).filter(Boolean);
};

// legacy NutrientKey -> our library.json camelCase key
const NKEY = {
  N_NO3: "nNo3", N_NH4: "nNh4", P2O5: "p2o5", K2O: "k2o", Mg: "mg", Ca: "ca", S: "s",
  Fe: "fe", Mn: "mn", Zn: "zn", B: "b", Cu: "cu", Si: "si", Mo: "mo", Na: "na", Cl: "cl",
};
const NUT_KEYS = Object.values(NKEY);

const canonicalRaw = table("FertilizerCanonical");
const variantRaw = table("FertilizerVariant");
const overrideRaw = table("FertilizerVariantOverride");
const pricingRaw = table("FertilizerVariantPricing");
const aliasRaw = table("FertilizerCanonicalAlias");

// canonical base nutrients keyed by canonical id
const baseNutrients = (r) => ({
  nNo3: num(r.nutrient_n_no3) ?? 0, nNh4: num(r.nutrient_n_nh4) ?? 0,
  p2o5: num(r.nutrient_p2o5) ?? 0, k2o: num(r.nutrient_k2o) ?? 0,
  ca: num(r.nutrient_ca) ?? 0, mg: num(r.nutrient_mg) ?? 0, s: num(r.nutrient_s) ?? 0,
  fe: num(r.nutrient_fe) ?? 0, mn: num(r.nutrient_mn) ?? 0, zn: num(r.nutrient_zn) ?? 0,
  b: num(r.nutrient_b) ?? 0, cu: num(r.nutrient_cu) ?? 0, si: num(r.nutrient_si) ?? 0,
  mo: num(r.nutrient_mo) ?? 0, na: num(r.nutrient_na) ?? 0, cl: num(r.nutrient_cl) ?? 0,
});
const canonicalById = new Map(canonicalRaw.map((r) => [r.id, r]));

const canonicals = canonicalRaw.map((r) => ({
  id: r.id, code: r.canonical_code, chemicalName: r.chemical_name, description: r.description,
  formula: r.formula, cas: r.cas, category: r.category, formFactor: r.default_form_factor,
  status: r.status, imageUrl: r.image_url,
  nutrients: baseNutrients(r),
  physical: {
    densityGCm3: num(r.physical_density_g_per_cm3),
    bulkDensityKgM3: num(r.physical_bulk_density_kg_per_m3),
    solubilityGL25c: num(r.physical_solubility_g_per_l_25c),
    ecMsCm1gL25c: num(r.physical_ec_ms_per_cm_1g_per_l_25c),
    ph1pct25c: num(r.physical_ph_1pct_solution_25c),
  },
  assay: {
    purityPct: num(r.assay_purity_pct), moisturePct: num(r.assay_moisture_pct),
    insolublePct: num(r.assay_insoluble_pct),
  },
  incompatibility: {
    phosphates: bool(r.incompatibility_with_phosphates), sulfates: bool(r.incompatibility_with_sulfates),
    calcium: bool(r.incompatibility_with_calcium), borate: bool(r.incompatibility_with_borate),
    highPhStock: bool(r.incompatibility_high_ph_stock),
  },
  compatibilityNotes: r.compatibility_notes,
  certsLabels: arr(r.certs_labels), hazardGhsCodes: arr(r.hazard_ghs_codes), hazardNotes: r.hazard_notes,
}));

// overrides grouped by variant, nutrient-scope only, applied like legacy buildResolvedNutrients
const overridesByVariant = new Map();
let appliedCount = 0;
for (const o of overrideRaw) {
  if (o.scope !== "nutrient") continue;
  const key = NKEY[o.target_field];
  if (!key) continue;
  const val = num(o.override_value);
  if (val === null) continue;
  if (!overridesByVariant.has(o.variant_id)) overridesByVariant.set(o.variant_id, []);
  overridesByVariant.get(o.variant_id).push({ key, val });
}

let archived = 0;
const variants = variantRaw
  .filter((r) => {
    if (r.archived_at && r.archived_at !== "\\N") { archived++; return false; }
    return true;
  })
  .map((r) => {
    const canon = canonicalById.get(r.canonical_id);
    const nutrients = canon ? baseNutrients(canon) : Object.fromEntries(NUT_KEYS.map((k) => [k, 0]));
    const ovs = overridesByVariant.get(r.id) || [];
    ovs.forEach((o) => { nutrients[o.key] = o.val; appliedCount++; });
    return {
      id: r.id, canonicalId: r.canonical_id, status: r.status,
      manufacturer: r.manufacturer, brandName: r.brand_name, modelSku: r.model_sku,
      country: r.country, gradeLabel: r.grade_label, formFactor: r.form_factor, imageUrl: r.image_url,
      packagingPackSizesKg: arr(r.packaging_pack_sizes_kg).map(Number),
      packagingShelfLifeMonths: num(r.packaging_shelf_life_months),
      certsLabels: arr(r.certs_labels), hazardLabels: arr(r.hazard_labels),
      nutrients,
      hasOverrides: ovs.length > 0,
    };
  });

const keptVariantIds = new Set(variants.map((v) => v.id));

const pricing = pricingRaw
  .filter((r) => keptVariantIds.has(r.variant_id))
  .map((r) => ({
    id: r.id, variantId: r.variant_id, packSizeKg: num(r.pack_size_kg),
    market: r.market, marketRegion: r.market_region, currency: r.currency,
    pricePerPack: num(r.price_per_pack), pricePerKg: num(r.price_per_kg),
    incoterm: r.incoterm, minOrderPack: num(r.min_order_pack), leadTimeDays: num(r.lead_time_days),
    lastUpdate: r.last_update, priceReference: r.price_reference, dataSource: r.data_source,
  }));

const aliases = aliasRaw.map((r) => ({
  canonicalId: r.canonical_id, alias: r.alias, aliasType: r.alias_type, langCode: r.lang_code,
}));

const out = {
  schemaVersion: 2,
  source: "Extracted from CropBinary production DB dump.",
  overridePolicy: "nutrient overrides applied at variant level (matches legacy buildResolvedNutrients, status-ignoring)",
  counts: {
    canonicals: canonicals.length, variants: variants.length, pricing: pricing.length,
    aliases: aliases.length, overridesApplied: appliedCount, archivedVariantsExcluded: archived,
  },
  canonicals, aliases, variants, pricing,
};

fs.writeFileSync(OUT, JSON.stringify(out, null, 2) + "\n");
console.log("Wrote", OUT);
console.log(out.counts);
const agsil = variants.find((v) => (v.brandName || "").includes("AgSil"));
if (agsil) console.log("AgSil 16H resolved K2O:", agsil.nutrients.k2o, "Si:", agsil.nutrients.si, "(overrides:", agsil.hasOverrides + ")");
