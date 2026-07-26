# IdleScreen (idle)

Core daemon, CLI, plugin API, and presentation host for **IdleScreen**.

IdleScreen is a modular Wayland idle manager: the daemon watches compositor idle signals, loads screensaver plugins, and presents their frames on a layer-shell surface. Control it with the CLI, optional TUI, or optional COSMIC panel applet (separate packages).

Website: [https://idlescreen.github.io](https://idlescreen.github.io)

---

## Quick install

**install.sh supports:** Fedora / RHEL-family (**DNF**) and Debian / Ubuntu-family (**APT**).  
**Arch:** experimental PKGBUILD in [idlescreen/packages](https://github.com/idlescreen/packages) `arch/` — not covered by the one-line installer.

```bash
curl -fsSL https://idlescreen.github.io/packages/install.sh | sh
# or after repo setup:  sudo dnf install idlescreen   /   sudo apt install idlescreen
```

Installs the product metapackage **`idlescreen`** (depends on `idle-daemon`, `idle-cli`, `idle-savers`, `idle-tui`), and on COSMIC also **`idle-cosmic`**. Remove with `sudo dnf remove idlescreen` / `sudo apt remove idlescreen`.

---

## Repository layout (this monorepo)

| Crate / area | Role |
|--------------|------|
| `idle-daemon` | Idle policy, plugin host, Wayland presentation, user systemd unit |
| `idle-cli` | CLI binaries `idlescreen` / `idle` |
| `idle-api` | Plugin trait / cell-grid ABI for savers |
| `idle-runner` | Shared host helpers (cell raster, sys info, plugin session) |
| `crates/wayland-present` | Layer-shell overlay presentation |

Related org repos (not always in this tree): `idle-tui`, `idle-cosmic`, `idle-saver-*`, `packages`, `render`, `idle-studio`.

---

## Official screensavers

Ten procedural **cell-grid plugins** (separate packages / repos). The host rasterizes cells to pixels (optional wgpu cell path, CPU fallback).

| Module | Description | Preview |
|--------|-------------|---------|
| **Beams** | Crossing vector beams | `idlescreen preview beams` |
| **Cosmos** | Starfield / nebula-style motion | `idlescreen preview cosmos` |
| **Bursts** | Expanding burst patterns | `idlescreen preview bursts` |
| **Storm** | Dense particles with flash accents | `idlescreen preview storm` |
| **Chaos** | Strange-attractor style curves | `idlescreen preview chaos` |
| **Hearth** | Warm ember / fire-like ambient | `idlescreen preview hearth` |
| **Ripple** | Expanding wave patterns | `idlescreen preview ripple` |
| **Radar** | Sweeping radar arc with blips | `idlescreen preview radar` |
| **Glyphs** | Falling character cascade | `idlescreen preview glyphs` |
| **Gnats** | Swarming agent motion | `idlescreen preview gnats` |

---

## Features (as shipped)

- **Wayland presentation** — requires compositor support for idle-notify and layer-shell (or equivalent). Strongest on COSMIC, Hyprland, and Sway; GNOME and KDE vary by protocol coverage. See `docs/BOUNDARIES.md`.
- **Cell-grid plugins + host raster** — savers implement `idle-api`; host rasterizes (optional wgpu, CPU fallback).
- **COSMIC panel applet** — optional package `idle-cosmic` (separate repo).
- **CLI and TUI** — `idlescreen` / `idle`; `idlescreen tui` runs the `idle-tui` binary when installed.
- **Inhibit and battery** — logind + MPRIS2 media inhibit; on battery, present/sim targets capped to 30 FPS/Hz.

---

## Manual package installation

<details>
<summary><b>Fedora / RHEL (DNF)</b></summary>

```bash
sudo curl -fsSL https://idlescreen.github.io/packages/rpm/idlescreen.repo \
  -o /etc/yum.repos.d/idlescreen.repo
sudo dnf check-update
sudo dnf install idlescreen
# COSMIC: sudo dnf install idle-cosmic
# Remove: sudo dnf remove idlescreen
```
</details>

<details>
<summary><b>Debian / Ubuntu (APT)</b></summary>

```bash
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://idlescreen.github.io/packages/apt/idlescreen-keyring.gpg \
  | sudo tee /etc/apt/keyrings/idlescreen-keyring.gpg >/dev/null
echo "deb [signed-by=/etc/apt/keyrings/idlescreen-keyring.gpg] https://idlescreen.github.io/packages/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/idlescreen.list >/dev/null
sudo apt update
sudo apt install idlescreen
# COSMIC: sudo apt install idle-cosmic
# Remove: sudo apt remove idlescreen
```
</details>

<details>
<summary><b>Arch Linux (experimental)</b></summary>

```bash
git clone https://github.com/idlescreen/packages.git
cd packages/arch
makepkg -si
```
</details>

---

## CLI commands

```bash
idlescreen tui              # Launch idle-tui dashboard
idlescreen status           # Daemon and saver state
idlescreen on               # Enable idle screensaver (alias: enable)
idlescreen off              # Disable idle screensaver (alias: disable)
idlescreen preview <name>   # Preview a saver now
idlescreen stop             # Stop preview / presentation
idlescreen doctor           # Diagnostics
idlescreen --help           # Full command list
```

Alias: `idle` is the same binary as `idlescreen`.

---

## Terminal UI

```bash
idlescreen tui
# or: idle-tui
```

| Key | Action |
|-----|--------|
| `Tab` | Switch panes |
| `Space` / `Enter` | Toggle or activate (depends on pane) |
| `p` | Preview selected saver (Savers pane) |
| `c` | Install `idle-cosmic` when COSMIC is detected and applet is missing |
| `q` / `Esc` | Quit |

---

## Links

- Website: [https://idlescreen.github.io](https://idlescreen.github.io)
- Packages: [github.com/idlescreen/packages](https://github.com/idlescreen/packages)
- Org: [github.com/idlescreen](https://github.com/idlescreen)
