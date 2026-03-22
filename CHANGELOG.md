# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Master password protection** — on first launch you choose a master password (or skip for password-free mode); your vault is encrypted with it via SQLCipher and you're prompted to unlock it each session
- **Lock screen** — when the vault is locked, a clean unlock form is shown; three consecutive wrong attempts surface a warning
- **Auto-lock** — set a timeout (1 / 5 / 10 / 15 / 30 min, or Off) in Settings → Security; the vault locks automatically after the chosen period of inactivity
- **Change Password** — Settings → Security → Change Password lets you rekey to a new password, or remove the password entirely; the new password goes through the same strength requirements and confirmation step
- **Password strength bar** — a live color-coded bar (Weak / Fair / Strong) appears below the password field on both first-run Setup and Change Password screens
- **First-run Setup view** — brand-new welcome screen walks you through securing your vault before you ever see the account list
- **`StrengthBar` component** — shared password-strength indicator extracted to `src/components/strength_bar.rs`, reused across Setup and Change Password
- **`AutoLockPicker` component** — modal sheet for selecting the auto-lock timeout with a checkmark next to the active selection
- **`ChangePasswordModal` component** — modal for rekeying the vault, including current-password verification, new password + confirm, and a "Remove password" escape hatch
- **`Radio` component** — accessible radio button used on the Setup screen for password / no-password selection
- Three new storage constants — `META_PASSWORD_SET`, `META_PASSWORD_HASH`, `META_AUTO_LOCK_SECS` — replace scattered string literals for meta-table keys

### Fixed
- Dragging the app window content no longer reveals a white background behind the WebView — the app now feels fully native with no browser-like rubber-band or drag artifacts

### Changed
- `AppShell` now drives a `LockState` machine (`FirstRun` → `Locked` ↔ `Unlocked`) and renders the appropriate view before reaching the normal routing layer
- Settings → Security → "Change PIN" row renamed to "Change Password"; "Auto-lock" row now opens the new `AutoLockPicker` modal
- Auto-lock interaction tracking uses an atomic timestamp (`LAST_INTERACTION_SECS`) so the background polling loop never causes re-renders

## [0.1.1] - 2026-03-21

### Added
- Right-click context menu on account rows — rename, move up/down within a group, change category (with inline picker), and delete with confirmation
- Long-press (600ms) opens the context menu on touch devices
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
