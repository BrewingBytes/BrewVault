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

Run the full test suite:

```sh
cargo test
```

Run a specific test by name:

```sh
cargo test test_wrong_key_fails
```

Run only the storage or TOTP tests:

```sh
cargo test storage
cargo test totp
```

Tests cover:

- `storage` — schema init, save/load round-trip, delete, and wrong-key rejection
- `totp` — code generation (SHA-1, SHA-256), output format, invalid secrets, and `seconds_remaining` range
