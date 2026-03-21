# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Right-click context menu on account rows — rename, move up/down within a group, change category (with inline picker), and delete with confirmation
- Long-press (500ms) opens the context menu on touch devices
- `sort_order` column on every entry — lets you arrange accounts in any order you like; existing databases are migrated automatically on first launch
- Move Up / Move Down actions keep entries within their group; boundary items are shown greyed-out so you always know where you are
- Category picker with inline "New category…" input and "No group" option to move an account out of any group
- Rename modal pre-fills the current issuer and account; Confirm stays disabled until the issuer field is non-empty
- Delete confirmation modal calls out the specific entry name so there's no ambiguity about what you're removing
- `RenameModal` and `DeleteConfirmModal` are now standalone public components — reusable anywhere in the app, not just from the context menu
- New design tokens: `--color-disabled` (`#2e2e2e`) for greyed-out items, `--color-overlay` for modal backdrops, `--shadow-menu` for floating surfaces
- **Delete All Accounts** in Settings → Danger Zone now works — confirm once and every entry is wiped from the vault; a toast confirms success or surfaces any error

### Changed
- Default WebView right-click menu ("Inspect Element" etc.) is suppressed app-wide; only BrewVault's own context menu appears on right-click
- Storage layer: `save_entry` (INSERT OR REPLACE) replaced by `insert_entry` (plain INSERT) — new entries must have a `sort_order` assigned before saving
- `delete_entry` now returns an error if the entry doesn't exist, instead of silently succeeding

## [0.1.0] - 2026-03-20
### Added
- Settings screen with profile card, Security, Backup & Sync, Preferences, About, and Danger Zone sections
- Accounts view with grouped TOTP entries (Dev / Work / Personal / other / ungrouped)
- Add account page with manual entry form
- Live TOTP code generation with countdown ring
- Click-to-copy TOTP codes with toast feedback
- Bottom navigation (Accounts / Settings tabs)
- Encrypted local storage via SQLite/SQLCipher
