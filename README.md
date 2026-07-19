# Fertilizer Studio

**Free, open-source, offline-first desktop app for fertilizer formula design and nutrient calculation.**

Built with [Tauri](https://tauri.app/) (Rust core) + React. Runs entirely on your
machine — no account, no server, no subscription. Your data stays local; a shared
fertilizer library is pulled from this public repo on launch and cached for offline use.

> ⚠️ **Early development.** This is a from-scratch rewrite of a former cloud SaaS into a
> native desktop app. It is not yet feature-complete or installable — the roadmap and
> progress live in [`plan.md`](./plan.md). Build steps below land with the Phase 1 scaffold.

---

## Why

Fertilizer formula design and nutrient math shouldn't require a subscription or an
internet connection. Fertilizer Studio puts a high-accuracy calculation engine and an
open fertilizer library on your desktop, for free.

## Features (target)

- **Fertilizer library** — canonical + variant catalog with pricing and specs, shared via
  this repo and browsable offline.
- **Formula design** — build recipes from library products; set nutrient targets; an
  **Auto** buffer picker chooses the best buffer nutrient for the most on-target results.
- **High-accuracy calculation engine** — nutrient totals (N / NO₃ / NH₄ / P / K / Ca / Mg /
  S / Si), PPM/EC, and dissolution / solution-strength simulation, implemented in Rust.
- **Feeding plans** — week-by-week schedules with vegetative/flowering summaries.
- **Local-first** — everything works offline; export/import your data to a file to back up
  or move between machines.
- **Multiple modes** — dry, percent (%), PPM, and liquid/stock workflows.

## Tech stack

| Layer | Choice |
|-------|--------|
| Shell | Tauri v2 (`.dmg` / `.exe` / AppImage · `.deb`) |
| Backend | Rust — domain logic + calculation engine, exposed as Tauri commands |
| Database | SQLite (local, in the OS app-data dir) |
| Frontend | React + Vite + Tailwind + shadcn/ui |
| Sync | Read-only shared library pulled from this public repo on launch |

## Build from source

> Coming with the Phase 1 scaffold. Planned prerequisites:
>
> - [Rust](https://www.rust-lang.org/tools/install) (stable)
> - [Node.js](https://nodejs.org/) 20+
> - [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) for your OS
>
> ```bash
> # once scaffolded:
> npm install
> npm run tauri dev      # run in development
> npm run tauri build    # produce installers for your platform
> ```

## Contributing

Contributions welcome — see [`CONTRIBUTING.md`](./CONTRIBUTING.md). By contributing you
agree your work is licensed under GPL-3.0.

## License

[GPL-3.0](./LICENSE) © Fertilizer Studio contributors. Copyleft — you may use, modify, and
redistribute, but derivatives must remain free and open under the same license.
