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

There are no automated tests at this time.
