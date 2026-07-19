# Contributing to Fertilizer Studio

Thanks for your interest! This is a free, open-source (GPL-3.0) desktop app built with
Tauri + Rust + React. Contributions of code, fertilizer-library data, bug reports, and
docs are all welcome.

> The project is in early development. Check [`plan.md`](./plan.md) for the current phase
> and roadmap before starting substantial work, and open an issue to discuss big changes
> first.

## Getting set up

Prerequisites (finalized with the Phase 1 scaffold):

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Node.js](https://nodejs.org/) 20+
- [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) for your OS

```bash
git clone <this-repo>
cd fertilizer-studio
npm install
npm run tauri dev
```

## Coding conventions

- **Rust** — format with `cargo fmt`; lint with `cargo clippy` (no warnings). Keep the
  calculation engine in its own crate with unit tests.
- **TypeScript/React** — follow the shadcn/ui + Tailwind conventions; avoid bespoke CSS.
- **Commits** — clear, imperative subject lines. Small, focused PRs.
- **Tests** — new calculation logic must ship with tests, validated against known values.

## Contributing library data

The shared fertilizer library is open. To add or correct product data, open a PR against
the library data file (location documented once Phase 1 lands) with sources for the
values where possible.

## License

By submitting a contribution, you agree it is licensed under the project's
[GPL-3.0](./LICENSE) license.
