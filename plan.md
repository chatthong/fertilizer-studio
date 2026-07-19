# CropBinary → Desktop Rewrite Plan (Tauri + Rust)

> **Status:** Planning · **Started:** 2026-07-19 · **Owner:** chatthong
> Living document. Check off acceptance criteria (AC) as each phase completes.

---

## 1. Vision

Rewrite the current cloud SaaS ("Fertilizer Studio" / CropBinary / AssayGrid — a
Next.js 16 + Postgres + Stripe app) into a **free, open-source desktop application**
distributed as `.dmg` / `.exe` / Linux packages. Same look and feel, cleaner
architecture, no servers to host.

**Guiding principles**
- Free software, open source, public repo.
- Host **nothing** — no paid cloud infra.
- Local-first: works fully offline.
- "Nerd geek" clean rewrite — Rust-native backend.

---

## 2. Strategy: new public repo, legacy stays private

- **Legacy** (this repo, the Next.js app) → **stays PRIVATE and untouched.** No history
  rewrite, no key rotation forced. It remains the reference source + the origin of the
  shared library data.
- **Rewrite** → **brand-new, separate GitHub project, public/OSS.** Fresh `git init`, so
  there is **no leaked-secret history to scrub** — the new repo starts clean.
- The new repo never receives `.env`; secrets are provided at runtime by the user's own
  machine (local DB, no cloud creds).

- **Project name:** **Fertilizer Studio** (repo slug e.g. `fertilizer-studio`)
- **License:** **GPL-3.0** (copyleft — derivatives stay free)
- **On-disk location:** `~/Desktop/projects/fertilizer-studio`

---

## 3. Confirmed decisions (from brainstorming, 2026-07-19)

| Topic | Decision |
|-------|----------|
| App model | Local-first + optional sync; **host nothing** |
| Sync scope | GitHub hosts read-only **shared library**, pulled on launch, cached for offline. User's own data is **local**, with manual **export/import** to a file. |
| Repo | **New public OSS repo**; legacy stays private |
| Shell | **Tauri v2** |
| Backend | **Rust-heavy / purist native** — all domain logic + calculation engine in Rust |
| Frontend | **React + Vite + Tailwind + shadcn** in Tauri's webview (keeps the exact same look) |
| Database | **SQLite** (local, in app-data dir), accessed from Rust (`sqlx` or `SeaORM`) |
| Library sync | Rust `reqwest` pulls shared-library file from the public repo; offline = last cached copy |
| Platforms | **macOS (.dmg)**, **Windows (.exe)**, **Linux (AppImage/deb)** |
| Monetization | **Removed entirely** — no Stripe, subscriptions, credits, entitlements, gating |
| Auth | **Removed** — no server auth/sessions/passkeys/TOTP; local single-user app |

---

## 4. Architecture (target)

```
┌────────────────────────────────────────────────────────────────┐
│  Tauri v2 app  →  .dmg / .exe / AppImage · deb  (small binaries)  │
│                                                                  │
│  Frontend (system webview)          Backend (Rust core)          │
│  ─────────────────────────          ────────────────────         │
│  React + Vite + Tailwind + shadcn   Tauri commands (IPC)         │
│  thin UI · invoke('cmd', …)         domain logic + calc engine   │
│                                     SQLite via sqlx/SeaORM       │
│                                     reqwest → GitHub library pull │
│                                     file I/O → export / import    │
└────────────────────────────────────────────────────────────────┘
   local DB in app-data dir  ·  shared library cached from public repo
```

**Data ownership split**
- **Shared library** (fertilizer/product catalog, public formulas): read-only, pulled
  from the public repo, cached locally, refreshed on launch when online.
- **User data** (their formulas, feeding plans, inventory, prefs): local SQLite only;
  manual export/import to a file for backup / moving devices.

---

## 5. What gets CUT vs KEPT

**CUT (removed in the rewrite)**
- Stripe / billing (invoices, refunds, coupons, tax, dunning)
- Subscriptions, plans, prices, credits (calculation + private-book), entitlements
- Gating cards / feature gating
- NextAuth, passkeys/WebAuthn, TOTP, sessions, device fingerprinting
- S3 / rustfs object storage → local files
- Rybbit analytics
- Admin consoles for all of the above
- DB-backed translation admin (i18n UI strings can stay as static frontend copy)
- The whole Next.js server layer (server actions, API routes, SSR)

**KEEP (the actual product — to re-port)**
- Fertilizer library: canonical + variant catalog, pricing, specs, documents
- Formula design: recipes, components, targets, target captures
- **Calculation engine**: nutrient totals (N / NO₃ / NH₄ / P / K / Ca / Mg / S / Si),
  PPM/EC, dissolution / solution simulation, buffer **Auto** picker, target solving,
  the **NNLS blend solver** (`lib/formulas/target-optimizer.ts`)
- **Chemical-incompatibility / safety engine** — pairwise mixing warnings from canonical
  flags (with-phosphates/sulfates/calcium/borate/high-pH); currently tangled in
  `create-formula-builder.tsx` (~L4100–4316) — must be lifted to a pure module
- **Batch-cost estimator + multi-currency (FX)** — live feature in the builder
  (`create-formula-builder.tsx` L3675–3790) + price resolution in `builder-options.ts`
- **`builder-options` assembly layer** — resolves canonical+variant+overrides+pricing
  into the builder/feeding option shape; the Prisma→domain resolver Rust must reimplement
- Feeding plans (week-by-week, stars, veg/flower peak summaries)
- Inventory — **currently mock/hardcoded (no persistence)** → greenfield build in Phase 4,
  not a port
- The shadcn/Tailwind look & UX
- Multi-language UI (as static strings)
- Library browse/search (`lib/search/explore-search.ts`)

**Reference:** detailed port map in the private legacy repo at `docs/port-atlas.md`
(chemistry constants, zod input contracts, PG→SQLite mapping, per-subsystem port notes).

---

## 6. Phased roadmap + acceptance criteria

> Each phase gets its own spec → implementation plan → build. Do **not** start a phase's
> build until its spec is approved. Order below is the agreed build order.

### Phase 0 — New OSS project setup  ⭐ NEXT
Stand up the clean public repo + governance. No app logic yet.

**Acceptance criteria**
- [x] New project directory + fresh `git init` created — `~/Desktop/projects/fertilizer-studio`
- [x] FOSS **LICENSE** chosen and added — GPL-3.0 (canonical text)
- [x] `README.md` written: what it is, build-from-source, contributing, license
- [x] `.gitignore` in place from commit #1 (never track `.env`, build artifacts, DB files)
- [x] `.env.example` template documented — no real secrets
- [x] `CONTRIBUTING.md` added (issue/PR templates deferred — optional)
- [x] Repo pushed to GitHub as **public** — https://github.com/chatthong/fertilizer-studio ; legacy `AssayGrid` confirmed **private**
- [x] Verified: no secret staged in the first commit (only `.env.example`)

### Phase 1 — Walking skeleton (scaffold + SQLite + library read + sync)
Thinnest end-to-end slice that exercises every layer and de-risks the architecture.

**Acceptance criteria**
- [ ] Tauri v2 app builds and runs on macOS (dev)
- [ ] React + Vite + Tailwind + shadcn frontend renders inside the Tauri window
- [ ] SQLite DB created in the OS app-data dir on first run
- [ ] Fertilizer-library schema ported to SQLite (Postgres enums/arrays → SQLite-safe)
- [ ] A Tauri command reads library rows from SQLite and returns them to the UI
- [ ] "Browse library" screen lists catalog items (read-only) with the shadcn look
- [ ] On launch: Rust pulls the shared-library file from the public repo, caches it,
      and populates/refreshes the local library tables
- [ ] Offline behavior verified: with no network, app uses the last cached library
- [ ] Documented seed/export path from the legacy Postgres → shared library file

### Phase 2 — Calculation engine crate (Rust)
Port the crown-jewel math into an isolated, tested crate.

**Acceptance criteria**
- [ ] Standalone Rust crate for the engine (no UI/DB deps)
- [ ] Nutrient totals + PPM/EC computed for a given recipe
- [ ] Dissolution / solution-strength simulation implemented
- [ ] Buffer **Auto** picker implemented
- [ ] Target-solving logic implemented
- [ ] Unit tests validate outputs against **known values captured from the legacy app**
      (parity within an agreed tolerance)
- [ ] Engine exposed to the frontend via Tauri commands

### Phase 3 — Formula design workflow
**Acceptance criteria**
- [ ] Create/edit/delete formula recipes (local SQLite persistence)
- [ ] Add/remove components from the library; ratios editable
- [ ] "Set the target" card works, driven by the Rust engine
- [ ] Buffer **Auto** option selects the best buffer nutrient
- [ ] Nutrient profile panel matches legacy output for sample formulas
- [ ] Star / organize formulas locally

### Phase 4 — Feeding plans + inventory
**Acceptance criteria**
- [ ] Week-by-week feeding plan create/edit/view
- [ ] Vegetative/flowering summary cards read stored peak values
- [ ] Star plans
- [ ] Inventory items + transactions (scope for v1 confirmed)

### Phase 5 — User data export / import + settings / i18n
**Acceptance criteria**
- [ ] "Export my data" writes a portable file (format decided)
- [ ] "Import" restores from that file (merge/replace behavior defined)
- [ ] Preferences persist locally (theme, units, locale)
- [ ] Multi-language UI via static strings (locales chosen)

### Phase 6 — Packaging, signing, auto-update, release
**Acceptance criteria**
- [ ] Tauri bundler produces `.dmg` (Apple Silicon + Intel)
- [ ] Tauri bundler produces Windows `.exe` (NSIS or MSI)
- [ ] Tauri bundler produces Linux AppImage + `.deb`
- [ ] Install + launch verified on each OS
- [ ] Signing strategy decided (macOS notarization? Windows cert? or ship unsigned)
- [ ] GitHub Releases publishes artifacts; auto-updater wired (optional v1)
- [ ] CI builds all three targets on tag

---

## 7. Open decisions (need input before/while building)

- [x] **Project name** — **Fertilizer Studio** (repo slug `fertilizer-studio`)
- [x] **License** — **GPL-3.0**
- [x] **New project location** on disk — `~/Desktop/projects/fertilizer-studio`
- [ ] **Rust DB layer**: `sqlx` (async, compile-checked SQL) vs `SeaORM` (ORM)
- [ ] **Shared-library file format**: SQLite `.db` (directly queryable) vs JSON
- [ ] **Inventory** in v1 scope, or defer to a later release?
- [ ] **Signing**: pay for macOS Developer ID + Windows cert, or ship unsigned first?

---

## 8. Risks & mitigations

**Top engine-port risks (from the Port Atlas):**
- **NNLS blend solver** (`lib/formulas/target-optimizer.ts`) — hand-rolled active-set NNLS
  + Gaussian elimination + constraint bisection with idiosyncratic tie-breaking a library
  crate won't reproduce. Key constants: `P2O5_TO_P=0.4364602764`, `K2O_TO_K=0.8301472547`,
  `EPSILON=1e-9`, tolerance `1e-8`, 12 geometric-escalation attempts ×10 + 40 bisection
  iters, `LIQUID_SCALING_FACTOR=10000`. → **Port verbatim to Rust f64; golden-master** the
  outputs against the legacy app.
- **EC estimator** buried in the 785-line `formula-ppm-analysis-card.tsx` (ION_SPECS,
  PPM_ION_FACTORS, ionic-strength piecewise interpolation). → Lift to a pure Rust module;
  guard the piecewise breakpoints with tests.
- **Buffer Auto-picker + mode scaling + accuracy grading** tangled in
  `create-formula-builder.tsx` (~L1440–2260, 16 solver calls, mode-specific target scaling,
  density fixed-point). → Highest extraction effort; isolate before porting.
- **Decimal-precision parity** — some paths need `rust_decimal`, not `f64`. → Decide
  per-path during Phase 2; golden-master catches drift.

**Cross-cutting:**
- **Engine parity** — capture golden test vectors from the legacy app first (Phase 2 AC).
- **Postgres → SQLite gaps** — enums→TEXT+CHECK, arrays/JSON→JSON columns; PG→SQLite map
  is in `docs/port-atlas.md §9`. Decide during Phase 1 schema port.
- **zod input contracts** (`schemas/fertilizer.ts`, `schemas/formulas.ts`) must be
  reproduced field-for-field in Rust (serde + validator) for every command.
- **Data migration** — legacy has `lib/sql-dump.ts` / `sql-import.ts` to egress Postgres;
  use it to seed the SQLite catalog / GitHub-published library file.
- **Scope creep** — the legacy has huge surface. → Ruthless YAGNI; §5 CUT list is binding.
- **Unsigned binaries** — Gatekeeper/SmartScreen warnings. → Acceptable for a free
  project v1; document the workaround; revisit signing in Phase 6.

---

## 9. Progress log

- **2026-07-19** — Brainstormed direction; chose Tauri+Rust rewrite, new public repo,
  legacy stays private. Plan created. Decided name = **Fertilizer Studio**, license =
  **GPL-3.0**. Launched the "Port Atlas" multi-agent workflow to map the surviving
  domain into `docs/port-atlas.md` (grounds the Phase 1/2 specs).
- **2026-07-19** — **Phase 0 scaffold done** (local commit `8c6079b`): GPL-3.0 LICENSE,
  README, CONTRIBUTING, .gitignore, .env.example, plan.md. Verified legacy `AssayGrid`
  repo is **PRIVATE**.
- **2026-07-19** — **Phase 0 COMPLETE**: repo public at
  https://github.com/chatthong/fertilizer-studio (final secret sweep clean).
- **2026-07-19** — **Port Atlas complete** (10-agent workflow, 0 errors, ~890K tokens) →
  `docs/port-atlas.md` in the private legacy repo. Findings folded into §5/§8: inventory
  is currently **mock** (greenfield, not a port); surfaced the incompatibility engine,
  live multi-currency batch-cost estimator, and `builder-options` assembly layer as
  must-port surviving features; recorded the top engine-port risks (NNLS solver, EC
  estimator, buffer Auto-picker, decimal precision).
- **2026-07-19** — **Cloud data extracted + verified** to a private local backup
  (`~/Desktop/projects/cropbinary-cloud-backup`, 233 MB): full Postgres dump (74 tables,
  .sql + .dump), rustfs images (175) + documents (22) + 8 daily DB backups. Servers safe
  to decommission (user handles via provider panel). Legacy *code* repo stays private.
- **2026-07-19** — **Golden vectors captured** (Phase 2 prep) → `golden/*.json` in this
  repo: `optimizeBlendShares` NNLS solver (9 branch-covering cases) + oxide/ppm
  conversions, at full f64 precision, run from the verbatim legacy TS. Harness kept in the
  private legacy repo `tools/golden-capture/`. Pending: EC estimator + buffer Auto-picker
  (tangled in React; capture during Phase 2).
