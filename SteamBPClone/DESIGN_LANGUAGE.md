# Steam Big Picture / Steam Deck UI — Design Language

A reference spec for replicating the **Steam Big Picture mode** (a.k.a. the modern
"Steam Deck UI" / gamepad UI) design language. All values were reverse-engineered
from the **local Steam client** on this machine:

```
~/.steam/debian-installation/steamui/css/chunk~2dcc5aaf7.css   (~4.9 MB compiled)
~/.steam/debian-installation/steamui/css/library.css
~/.steam/debian-installation/steamui/images/
```

The Steam client is a Chromium/CEF app, so its UI *is* HTML/CSS — these are the
real production values, not approximations. This document is written so another
AI (or human) can reproduce the look with the **correct vocabulary**.

> **Asset/IP note.** Steam's brand typeface **Motiva Sans** and Valve's image
> assets are proprietary and are **not** redistributed here. We name them
> correctly and ship the exact fallback stack so the real font drops in cleanly.

---

## 1. Core identity

| Trait | Value |
|---|---|
| Mood | Dark, cinematic, "blue-black", high-contrast, console-like |
| Background | Deep blue-black with a soft top radial vignette; often blurred hero art behind |
| Shape | Soft but tight radii (2–10px); fully-rounded pills (22px) for nav & keys |
| Density | Generous padding, large touch/controller targets, big focus states |
| Signature move | **The "Focusable" lift** — focused element scales up, lifts toward the viewer, casts a soft shadow, and (on gamepad) **inverts to white surface / dark text** |

---

## 2. Color

### Surface ramp (deepest → lightest)
| Token (ours) | Hex | Role | Source frequency |
|---|---|---|---|
| `--steam-surface-0` | `#0e141b` | deepest panel / dialog body | 453× |
| `--steam-surface-1` | `#171d25` | app background base | 15× |
| `--steam-surface-2` | `#1b2838` | classic store blue surface | 5× |
| `--steam-surface-3` | `#23262e` | standard panel / card | **1089×** |
| `--steam-surface-4` | `#3d4450` | raised surface / hairline border | **1234×** |

### Text ramp (muted → bright)
| Token | Hex | Role | Frequency |
|---|---|---|---|
| `--steam-text-muted` | `#67707b` | disabled / tertiary | 779× |
| `--steam-text-secondary` | `#8b929a` | secondary body text | **1590× (most used)** |
| `--steam-text-default` | `#b8bcbf` | default body text | 307× |
| `--steam-text-bright` | `#dcdedf` | high-emphasis | 227× |
| `--steam-text-white` | `#ffffff` | headings / focused label | — |

### Accents
| Token | Hex | Role |
|---|---|---|
| `--steam-accent` | `#1a9fff` | **THE primary accent** (240×) — actions, progress, selection underline |
| `--steam-accent-bright` | `#47bfff` | blue gradient start (20×) |
| `--steam-accent-deep` | `#1a43c0` | blue gradient end |
| `--steam-online` | `#66c0f4` | online status / links (legacy store blue) |
| `--steam-destructive` | `#de3618` | destructive actions / errors |

### Signature gradients (verbatim from source)
- **Green "Play"/Launch CTA:** `linear-gradient(to right, #70d61d 0%, #01a75c 160%)`
- **Blue Install/Download CTA:** `linear-gradient(to right, #47bfff 0%, #1a43c0 160%)`

**Rule of thumb:** surfaces are desaturated blue-black; anything *interactive or
in-progress* trends to `#1a9fff`; the only saturated greens/reds are the Play and
Destructive CTAs respectively.

---

## 3. Typography

```css
font-family: "Motiva Sans", Arial, Helvetica, sans-serif;   /* UI text */
font-family: "Roboto Mono", "Consolas", monospace;          /* numeric / code */
```

- **Brand face:** *Motiva Sans* — a geometric-humanist sans, shipped by Steam as
  `clientui.uifont` at weights **100, 300, 400, 500, 700, 800, 900** (+ italics).
  Proprietary; substitute with a close free face (e.g. *Mulish*, *Nunito Sans*,
  or just the Arial fallback) if you cannot license it.
- Headings are heavy (700–900) and slightly letter-spaced; body is 400–500.
- Numerals (clock, timers, %) often use tabular/mono figures.

| Scale token | px | Use |
|---|---|---|
| `--steam-fs-hero` | 48 | clock / hero numerals (`--clock-font-size`) |
| `--steam-fs-h1` | 32 | hero titles |
| `--steam-fs-h2` | 24 | section titles |
| `--steam-fs-h3` | 18 | sub-headers, nav |
| `--steam-fs-body` | 15 | body |
| `--steam-fs-small` | 13 | secondary |
| `--steam-fs-tiny` | 11 | badges / hints |

---

## 4. Shape & spacing

**Radii (frequency-ranked in source):** `2px` (306×, inputs/chips) · `3px` (192×,
buttons) · `4px` (128×, cards) · `8px` (104×, panels/tiles) · `10px` (102×, feature
tiles) · `22px` (148×, pills & on-screen-keyboard keys).

**Spacing:** 4/8 base grid — `4, 8, 12, 16, 20, 24, 32, 48`.

**Layout tokens (real Steam variables):**
- `--gamepad-page-content-max-width: 1100px` → our `--steam-content-max`
- `--capsule-width: 320px` → our `--steam-capsule-w` (vertical **2:3** game tile)

---

## 5. Motion & the Focus model ⭐ (the most important part)

Big Picture is **controller-first**. Every navigable element carries the class
**`.Focusable`**, and the currently-focused one gets **`.gpfocus`** ("gamepad
focus"). Reproduce these exactly — this is what makes it *feel* like Steam:

```css
/* verbatim from .Focusable:focus in the source */
transition: filter, box-shadow, transform .3s cubic-bezier(0.16, 0.86, 0.43, 0.99);
box-shadow: 0px 16px 24px 0px rgba(0,0,0,.5);
transform:  translateZ(15px);     /* tiles also scale 1.05–1.1 */
z-index: 12;
```

| Token | Value |
|---|---|
| `--steam-ease` | `cubic-bezier(0.16, 0.86, 0.43, 0.99)` |
| `--steam-dur` | `0.3s` |
| `--steam-focus-lift` | `scale(1.06)` (tiles range 1.05–1.2) |
| `--steam-focus-shadow` | `0px 16px 24px 0px rgba(0,0,0,.5)` |

**Gamepad-focus color inversion:** a focused default control flips to
**white background, dark text** (`.gpfocus { background:#fff; color:#23262e }`).
Primary keeps blue (`#1a9fff`), Destructive keeps red (`#de3618`).

Navigation is **spatial**: D-pad/arrow keys move focus to the nearest focusable in
that direction (implemented in `app.js`).

---

## 6. Component vocabulary (use these names)

| Steam internal name | What it is | Our class |
|---|---|---|
| `Focusable` / `gpfocus` | navigable element + its focused state | `.bpm-focusable` / `.gpfocus` |
| `DialogButton` | button; variants `Primary`, `Destructive`, `BarButton`, `Glyph` | `.bpm-button` (+ `--primary/--destructive/--bar/--glyph`) |
| `capsule` | vertical 2:3 library game tile | `.bpm-capsule` |
| `Field` | a settings row (label + control) | `.bpm-field` |
| `library_hero` / `library_capsule` / `library_logo` | hero art / grid art / transparent logo image slots | hero section |
| `DialogHeader` | section/dialog title (letter-spaced) | `.bpm-section__title` |
| footer button-hint bar | A/B/X/Y controller glyph hints | `.bpm-footerbar` + `.bpm-glyph` |

Controller glyph colors: **A** green, **B** red, **X** blue, **Y** yellow.

---

## 7. Layout anatomy of a Big Picture screen

```
┌───────────────────────────────────────────────────────────┐
│  NAV: [logo] Store  Library  Community … clock  [avatar]   │  sticky, dark gradient
├───────────────────────────────────────────────────────────┤
│  HERO: blurred game art, logo, [▶ Play] [Achievements]…    │  radius 10, 2:3.. cinematic
│                                                             │
│  RAIL: ◧ ◧ ◧ ◧ ◧  ← horizontal capsule shelf, scroll-snap   │
│                                                             │
│  content max-width 1100px, centered                         │
├───────────────────────────────────────────────────────────┤
│  FOOTER: (A)Select  (B)Back  (X)Filter  (Y)Details          │  fixed, blurred
└───────────────────────────────────────────────────────────┘
```

---

## 8. Files in this reference

| File | Purpose |
|---|---|
| `styles/tokens.css` | all design tokens as CSS custom properties (the source of truth) |
| `styles/components.css` | buttons, capsules, fields, toggles, sliders, tabs, badges, glyphs |
| `styles/app.css` | app shell: nav, hero, rails, grids, footer bar |
| `index.html` | navigable showcase = component library |
| `app.js` | live clock, generated tiles/swatches, **spatial gamepad focus nav** |

## 9. How to view

```bash
cd ~/SteamBPClone
python3 -m http.server 8000   # then open http://localhost:8000
```

Move focus with **Tab** or the **arrow keys** to feel the Focusable lift and the
gamepad color-inversion that define Big Picture mode.

---

## 10. Source locations (for re-extraction / verification)

- Compiled UI CSS: `~/.steam/debian-installation/steamui/css/chunk~2dcc5aaf7.css`
- Library CSS: `~/.steam/debian-installation/steamui/css/library.css`
- Fonts (proprietary): `steamui/` served as `/custom_fonts/clientui.uifont?MotivaSans-*`
- Image slots: `steamui/images/` (`library_hero.png`, `library_capsule.png`,
  `library_logo_transparent.png`, controller glyphs, `steam_spinner.png`, …)
