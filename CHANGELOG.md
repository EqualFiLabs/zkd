# Changelog

## Unreleased

- Adopted ADR-001 by deferring official Go, .NET, Java/Kotlin, and Swift bindings to the Ecosystem phase; introduced `docs/bindings-cookbook.md` for DIY integrators.
- Updated roadmap, interfaces, test plan, tasklist, README, and architecture docs to reflect the Phase-0 binding surface (C ABI, Python, Flutter/Dart, WASI) and mark deferred targets as non-normative.
- Added ABI stability coverage: new `zkp_version` export, version metadata in `zkp_prove`/`zkp_verify`, symbol-presence integration tests, and buffer ownership assertions.
