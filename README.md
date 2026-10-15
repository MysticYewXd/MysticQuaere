<div align="center">

<img src="assets/logo.svg" width="96" alt="MysticQuaere logo">

# MysticQuaere

**Seek anything on your Linux desktop.**

A Spotlight-style launcher for Wayland, written in Rust and GTK4.<br>
It finds apps, files and answers as you type, and takes its colours from your wallpaper.

<sub><i>quaere</i>: Latin for "seek"</sub>

</div>

<!-- Add a screenshot here once you have one:
<p align="center"><img src="assets/screenshot.png" width="680" alt="MysticQuaere open over a wallpaper"></p>
-->

---

## Contents

- [Features](#features)
- [Does it work on my desktop?](#does-it-work-on-my-desktop)
- [Install](#install)
  - [Step 1: Install Rust](#step-1-install-rust)
  - [Step 2: Install the build dependencies](#step-2-install-the-build-dependencies)
  - [Step 3: Build and install MysticQuaere](#step-3-build-and-install-mysticquaere)
  - [Step 4: Bind a hotkey](#step-4-bind-a-hotkey)
  - [Step 5: Start it at login (recommended)](#step-5-start-it-at-login-recommended)
- [Using it](#using-it)
- [Themes and colours](#themes-and-colours)
- [Updating](#updating)
- [Uninstalling](#uninstalling)
- [Troubleshooting](#troubleshooting)
- [License](#license)

---

## Features

| | |
|---|---|
| **Apps** | Every app in your app menu, ranked as you type. The list refreshes by itself when you install or remove apps. |
| **Files** | Searches your home folder by name. Hidden folders and build clutter (`node_modules`, `target`, `__pycache__`, `venv`) are skipped. |
| **Calculator** | Type `2 * (3 + 4) ^ 2` and the answer appears first. Press <kbd>Enter</kbd> to copy it. |
| **Web** | Anything else can go straight to a DuckDuckGo search. |
| **Wallpaper colours** | Reads the palette that matugen or pywal makes from your wallpaper, and recolours itself live when you switch wallpaper. |
| **Themes** | Six built-in looks, and any CSS of your own on top. |
| **Instant** | Runs quietly in the background, so the hotkey opens it immediately. |

---

## Does it work on my desktop?

MysticQuaere floats above your windows using the Wayland **layer-shell** protocol. Most tiling and KDE desktops support it; GNOME does not.

| Desktop | Works | Floats above windows | Blur behind the card |
|---|:---:|:---:|:---:|
| Hyprland | ✅ | ✅ | ✅ with a layer rule |
| Sway / SwayFX | ✅ | ✅ | SwayFX only |
| KDE Plasma 6 (Wayland) | ✅ | ✅ | ➖ translucent, no blur |
| niri, river, Wayfire, labwc | ✅ | ✅ | depends on the compositor |
| GNOME | ✅ | ➖ opens as a normal window | ➖ |
| X11 sessions | ✅ | ➖ opens as a normal window | ➖ |

> [!NOTE]
> Everything except the floating overlay works everywhere. On GNOME and X11 it simply appears as a small borderless window and hides itself when you click away.

---

## Install

There are no ready-made packages yet, so you build MysticQuaere from source. It takes a few minutes and you only do it once.

> [!IMPORTANT]
> **You need Rust 1.92 or newer.** Many distros ship an older Rust, so Step 1 uses **rustup**, the official installer. On Arch and its relatives, the `rust` package from pacman is new enough and you can use that instead.

### Step 1: Install Rust

Check whether you already have a new enough version:

```sh
rustc --version
```

If that prints `rustc 1.92` or higher, skip to Step 2. Otherwise install rustup:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Accept the default options, then **close and reopen your terminal** so `cargo` is on your `PATH`.

<details>
<summary>Using the fish shell?</summary>

rustup sets up bash and zsh automatically. For fish, run this once:

```fish
fish_add_path ~/.cargo/bin
```

</details>

> [!TIP]
> Already have rustup? Run `rustup update` to get the latest Rust.

### Step 2: Install the build dependencies

You need a C compiler, `pkg-config`, **GTK 4** and **gtk4-layer-shell**, including their development files. Pick your distro:

<details open>
<summary><b>Arch Linux, EndeavourOS, CachyOS, Manjaro, Garuda</b></summary>

```sh
sudo pacman -S --needed base-devel git rust gtk4 gtk4-layer-shell
```

On Arch you can use the `rust` package here instead of rustup.

</details>

<details>
<summary><b>Fedora</b></summary>

```sh
sudo dnf install gcc git pkgconf-pkg-config gtk4-devel gtk4-layer-shell-devel
```

</details>

<details>
<summary><b>Debian 13 (Trixie) or newer, Ubuntu 24.10 or newer</b></summary>

```sh
sudo apt install build-essential git pkg-config libgtk-4-dev libgtk4-layer-shell-dev
```

</details>

<details>
<summary><b>Ubuntu 24.04 LTS, Linux Mint 22, Pop!_OS 24.04</b></summary>

These releases have GTK 4 but **no gtk4-layer-shell package**, so you build that one library yourself first. It takes about a minute.

```sh
sudo apt install build-essential git pkg-config libgtk-4-dev \
    meson ninja-build libwayland-dev wayland-protocols

git clone --depth 1 --branch v1.3.0 https://github.com/wmww/gtk4-layer-shell
cd gtk4-layer-shell
meson setup build --prefix=/usr/local -Dintrospection=false -Dvapi=false \
    -Dexamples=false -Ddocs=false -Dtests=false
ninja -C build
sudo ninja -C build install
sudo ldconfig
cd ..
```

Check that it worked. This should print `1.3.0`:

```sh
pkg-config --modversion gtk4-layer-shell-0
```

</details>

<details>
<summary><b>openSUSE Tumbleweed</b></summary>

```sh
sudo zypper install gcc git pkgconf gtk4-devel gtk4-layer-shell-devel
```

</details>

<details>
<summary><b>NixOS or Nix</b></summary>

Open a shell with everything you need, then follow Step 3 inside it:

```sh
nix-shell -p cargo rustc pkg-config gtk4 gtk4-layer-shell git
```

</details>

<details>
<summary><b>Any other distro</b></summary>

Install your distro's packages for:

- a C compiler and linker (often a `base-devel` or `build-essential` group)
- `pkg-config` (or `pkgconf`)
- GTK 4 **with development files** (usually `gtk4-devel`, `libgtk-4-dev` or `gtk4.0-dev`)
- gtk4-layer-shell 1.0 or newer **with development files** (usually `gtk4-layer-shell-devel` or `-dev`)

If there is no gtk4-layer-shell package, build it from source as shown in the Ubuntu 24.04 section above.

You can check that both libraries are found:

```sh
pkg-config --modversion gtk4 gtk4-layer-shell-0
```

</details>

### Step 3: Build and install MysticQuaere

```sh
git clone https://github.com/MysticYewXd/mysticquaere
cd mysticquaere
cargo install --path .
```

The first build downloads and compiles everything, which takes a few minutes. Afterwards the program is at `~/.cargo/bin/mysticQuaere`.

> [!WARNING]
> The command has a **capital Q**: `mysticQuaere`. Typing `mysticquaere` will say "command not found".

Try it straight away:

```sh
mysticQuaere
```

The launcher should appear. Press <kbd>Esc</kbd> to hide it.

<details>
<summary>Prefer a system-wide install in <code>/usr/local/bin</code>?</summary>

```sh
cargo build --release
sudo install -Dm755 target/release/mysticQuaere /usr/local/bin/mysticQuaere
```

</details>

### Step 4: Bind a hotkey

Running `mysticQuaere` shows the launcher if it's hidden and hides it if it's showing. Bind that command to a key. The examples use <kbd>Super</kbd>+<kbd>Space</kbd>.

> [!TIP]
> Some desktops start apps from a hotkey without your terminal's `PATH`. If the key does nothing, use the full path instead, for example `/home/YOU/.cargo/bin/mysticQuaere`.

<details open>
<summary><b>Hyprland</b></summary>

In `~/.config/hypr/hyprland.conf`:

```ini
bind = SUPER, SPACE, exec, mysticQuaere

# Frosted-glass blur behind the launcher
layerrule = blur, launcher
layerrule = ignorezero, launcher
```

Using the Lua config instead? Add this to your keybinds and rules files:

```lua
hl.bind("SUPER + Space", hl.dsp.exec_cmd("mysticQuaere"), { description = "MysticQuaere" })
hl.layer_rule({ match = { namespace = "launcher" }, blur = true })
```

> [!NOTE]
> Hyprland's rule syntax changes between releases. If your version rejects these lines, check the Hyprland wiki for layer rules. The launcher's namespace is **`launcher`**.

</details>

<details>
<summary><b>Sway / SwayFX</b></summary>

In `~/.config/sway/config`:

```
bindsym $mod+space exec mysticQuaere

# SwayFX only: blur behind the launcher
layer_effects "launcher" blur enable
```

Sway's default config already uses `$mod+space` to switch focus between floating and tiled windows. Remove or change that line first.

</details>

<details>
<summary><b>KDE Plasma</b></summary>

1. Open **System Settings → Keyboard → Shortcuts**.
2. Click **Add New → Command or Script…**
3. Command: `mysticQuaere`
4. Set the shortcut to <kbd>Meta</kbd>+<kbd>Space</kbd> and click **Apply**.

</details>

<details>
<summary><b>GNOME</b></summary>

1. Open **Settings → Keyboard → View and Customise Shortcuts → Custom Shortcuts**.
2. Click **+** (Add Shortcut).
3. Name: `MysticQuaere`. Command: `mysticQuaere`.
4. Set the shortcut and click **Add**.

GNOME uses <kbd>Super</kbd>+<kbd>Space</kbd> to switch keyboard layouts, so pick another key or change that one first.

</details>

<details>
<summary><b>niri</b></summary>

In `~/.config/niri/config.kdl`, inside `binds { }`:

```kdl
Mod+Space { spawn "mysticQuaere"; }
```

</details>

### Step 5: Start it at login (recommended)

The first time you press the hotkey, MysticQuaere starts up and builds its file index. To make even the first press instant, start it hidden when you log in:

| Desktop | Add this |
|---|---|
| Hyprland | `exec-once = mysticQuaere --daemon` |
| Sway | `exec mysticQuaere --daemon` |
| niri | `spawn-at-startup "mysticQuaere" "--daemon"` |
| KDE Plasma | **System Settings → Autostart → Add… → Add Application**, command `mysticQuaere --daemon` |
| GNOME | Create `~/.config/autostart/mysticquaere.desktop` (below) |

<details>
<summary>The autostart file for GNOME (it also works on KDE and most other desktops)</summary>

```ini
[Desktop Entry]
Type=Application
Name=MysticQuaere
Exec=mysticQuaere --daemon
NoDisplay=true
X-GNOME-Autostart-enabled=true
```

</details>

---

## Using it

| Key | What it does |
|---|---|
| Just type | Search apps, files, maths and the web at once |
| <kbd>↑</kbd> <kbd>↓</kbd> | Move the highlight |
| <kbd>Enter</kbd> | Open the highlighted result (copies the answer for maths) |
| Click | Open that result |
| <kbd>Esc</kbd> | Hide the launcher |
| Your hotkey | Show or hide the launcher |

Results come in this order: the calculator answer, then up to 6 apps, then up to 4 files and folders, then a web search for exactly what you typed.

> [!NOTE]
> File search starts after **two characters** and looks at names only, not file contents. The index is rebuilt in the background each time the launcher opens, so new files appear on the next open.

---

## Themes and colours

### Your wallpaper's colours (automatic)

Out of the box, MysticQuaere takes its colours from your wallpaper and recolours itself the moment you switch wallpaper. It looks for these files, in this order:

| Tool | File it reads |
|---|---|
| matugen (used by end-4's dots-hyprland, Caelestia and many Hyprland setups) | `~/.local/state/quickshell/user/generated/colors.json` |
| pywal / wallust (pywal format) | `~/.cache/wal/colors.json` |

If neither exists, you get the built-in **Obsidian Veil** look: violet-black glass with a lilac highlight. A light wallpaper scheme gives a light card automatically.

### Using a fixed theme

Six ready-made themes are in the [`themes/`](themes) folder:

| Theme | Look |
|---|---|
| `obsidian-veil.css` | Violet-black glass with a soft lilac selection |
| `astrolabe.css` | Navy glass, a thin gold double ring and serif type |
| `moonstone.css` | Light lavender glass with indigo ink |
| `aurora.css` | Teal blending into violet |
| `ember-rune.css` | Amber monospace on near-black |
| `wraith-glass.css` | Almost clear, with round corners |

To use one, copy it to your personal style file:

```sh
mkdir -p ~/.config/mysticquaere
cp themes/aurora.css ~/.config/mysticquaere/style.css
```

MysticQuaere notices the file straight away; there's no need to restart. To go back to wallpaper colours, delete `style.css` or empty it.

> [!TIP]
> Astrolabe uses the **Cormorant Garamond** font and Ember Rune uses **JetBrains Mono**. Install them for the intended look; without them you get your normal serif and monospace fonts.

### Writing your own

`~/.config/mysticquaere/style.css` is plain GTK 4 CSS, loaded on top of everything else. Start every rule with `window.mystic` so it only affects the launcher. The parts you can style:

| Selector | What it is |
|---|---|
| `window.mystic .card` | The card itself |
| `window.mystic .card entry.search` | The search field |
| `window.mystic .card entry.search text placeholder` | The "Seek apps, files, maths…" hint |
| `window.mystic .card separator` | The line under the search field |
| `window.mystic .card row` / `row:selected` | A result / the highlighted result |
| `window.mystic .card .title` / `.subtitle` | A result's name / its description |
| `window.mystic .card .calc` | The calculator answer |

The wallpaper colours are available to your CSS as `@mq_surface`, `@mq_text`, `@mq_primary` and `@mq_tertiary`. For example, to keep the wallpaper colours but make the card rounder and more solid:

```css
window.mystic .card {
    border-radius: 28px;
    background: alpha(@mq_surface, 0.85);
}
```

> [!WARNING]
> Don't put launcher styles in `~/.config/gtk-4.0/gtk.css`. Wallpaper tools like matugen rewrite that file every time you change wallpaper, and your changes would be lost.

---

## Updating

```sh
cd mysticquaere
git pull
cargo install --path .
pkill mysticQuaere
```

The next hotkey press starts the new version. If you start it at login, run `mysticQuaere --daemon` once as well, or simply log out and back in.

## Uninstalling

```sh
pkill mysticQuaere
cargo uninstall mysticQuaere
rm -rf ~/.config/mysticquaere
```

Then remove the hotkey and autostart lines you added. If you installed to `/usr/local/bin`, use `sudo rm /usr/local/bin/mysticQuaere` instead of `cargo uninstall`.

---

## Troubleshooting

<details>
<summary><b><code>The system library `gtk4-layer-shell-0` required by crate `gtk4-layer-shell-sys` was not found</code></b></summary>

The gtk4-layer-shell development files are missing. Install the package for your distro from [Step 2](#step-2-install-the-build-dependencies). If your distro has no package, build it from source as in the Ubuntu 24.04 section. The same applies to a missing `gtk4` library: install GTK 4's development package.

</details>

<details>
<summary><b><code>error: rustc 1.85.1 is not supported by the following packages: … requires rustc 1.92</code></b></summary>

Your Rust is too old. Install rustup as in [Step 1](#step-1-install-rust), or run `rustup update` if you already have it. Then make sure `rustc --version` shows the new version. If it doesn't, your distro's Rust is coming first on your `PATH`: remove the distro's `rust` or `cargo` package.

</details>

<details>
<summary><b><code>error while loading shared libraries: libgtk4-layer-shell.so.0</code></b></summary>

This happens after building gtk4-layer-shell yourself. Run `sudo ldconfig` so the system finds the new library, then try again.

</details>

<details>
<summary><b><code>mysticQuaere: command not found</code></b></summary>

- Check the capital **Q**.
- Make sure `~/.cargo/bin` is on your `PATH`. Reopen your terminal after installing rustup, or on fish run `fish_add_path ~/.cargo/bin`.
- For hotkeys, use the full path: `~/.cargo/bin/mysticQuaere` (written out as `/home/YOU/.cargo/bin/mysticQuaere`).

</details>

<details>
<summary><b>It opens as a normal window instead of floating</b></summary>

Your desktop doesn't support layer-shell. That's expected on GNOME and X11; see [Does it work on my desktop?](#does-it-work-on-my-desktop). Everything else still works.

</details>

<details>
<summary><b>There's no blur behind the card</b></summary>

Blur comes from your compositor, not the app. On Hyprland, add the `layerrule` lines from [Step 4](#step-4-bind-a-hotkey). On Sway, blur needs SwayFX. KDE Plasma doesn't blur this kind of window, so the card is translucent without blur.

</details>

<details>
<summary><b>The colours don't follow my wallpaper</b></summary>

MysticQuaere only reads the files listed under [Your wallpaper's colours](#your-wallpapers-colours-automatic). Check that one of them exists:

```sh
ls ~/.local/state/quickshell/user/generated/colors.json ~/.cache/wal/colors.json
```

Also check that `~/.config/mysticquaere/style.css` isn't setting its own colours, because it overrides the wallpaper colours.

</details>

<details>
<summary><b>I changed the code or a setting and nothing happened</b></summary>

An older copy is probably still running in the background. Stop it with `pkill mysticQuaere`, then press your hotkey again.

</details>

<details>
<summary><b>It still doesn't work</b></summary>

Stop the background copy and start it from a terminal so you can see its messages:

```sh
pkill mysticQuaere
mysticQuaere
```

Then open an issue on GitHub with that output, your distro, and your desktop.

</details>

---

## License

MysticQuaere is released under the [MIT License](LICENSE).

<div align="center"><sub>Made by Kong. A sibling of MysticJam.</sub></div>
