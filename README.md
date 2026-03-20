# BrewVault

A desktop TOTP authenticator app built with [Dioxus](https://dioxuslabs.com/) and Rust.

## Prerequisites

- **Rust** — install via [rustup](https://rustup.rs/)
- **Dioxus CLI** — `cargo install dioxus-cli`
- **Tailwind CSS CLI**
  - macOS: `brew install tailwindcss`
  - Windows / Linux: see the [Tailwind CSS installation docs](https://tailwindcss.com/docs/installation)

## Dev workflow

**Build the app in dev**

```sh
tailwindcss -i assets/tailwind.css -o assets/main.css --watch & dx serve --desktop
```

## Release build

```sh
dx bundle --desktop
```

The bundled app will be placed under `target/dx/brew-vault/bundle/<platform>`.

## Releases

Tagged releases are built automatically via GitHub Actions for macOS, Linux, and Windows.

To publish a new release:

1. Update the version in `Cargo.toml`
2. Add a `## [x.y.z]` section with content to `CHANGELOG.md`
3. Tag and push:

```sh
git tag vx.y.z
git push --tags
```

The workflow validates that the tag matches `Cargo.toml`, builds installers on all three platforms, and publishes a GitHub Release with the CHANGELOG notes attached. Pre-release tags (e.g. `v1.0.0-beta.1`) are automatically marked as pre-releases.

## Project structure

```
brew-vault/
├── assets/
│   ├── tailwind.css   # Tailwind source (edit this)
│   └── main.css       # Generated output (do not edit)
├── src/
│   ├── main.rs        # App entry point
│   ├── components/    # Dioxus UI components
│   └── models/        # Data models and app state
├── Cargo.toml
└── Dioxus.toml
```

## Tests

Run the full test suite (unit + integration):

```sh
cargo test
```

Run only unit tests:

```sh
cargo test --lib
```

Run only integration tests:

```sh
cargo test --test storage_roundtrip
```

Run a specific test by name:

```sh
cargo test test_wrong_key_fails
```

Filter by module:

```sh
cargo test storage
cargo test totp
```

Tests cover:

- `storage` (unit) — schema init, save/load round-trip, upsert, delete, and wrong-key rejection
- `totp` (unit) — code generation (SHA-1, SHA-256), output format, invalid secrets, and `seconds_remaining` range
- `storage_roundtrip` (integration) — persists 3 entries to a real encrypted file, reopens with the same key, and asserts all fields survive the round-trip
