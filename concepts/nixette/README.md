# Nixette – Declarative, Sourceful, and Unapologetically Herself

A playful concept distro imagined as the transfemme child of **NixOS** and **Gentoo**. Nixette blends the reproducible confidence of flakes with the fine-grained self-expression of USE flags, wrapped in a trans flag palette and a big, affirming hug.

---

## Identity Snapshot

- **Tagline:** _Declarative, sourceful, and unapologetically herself._
- **Mascot:** Chibi penguin “Nixie” with pastel pigtails, Nix snowflake + Gentoo swirl hoodie.
- **Palette:** `#55CDFC` (sky blue), `#F7A8B8` (pink), `#FFFFFF`, plus a deep accent `#7C3AED`.
- **Pronoun Prompt:** The installer asks for name/pronouns and personalises MOTD, systemd messages, and shell prompt.

---

## Feature Mix

| Pillar                | How Nixette expresses it                                                                                 |
|----------------------|-----------------------------------------------------------------------------------------------------------|
| Reproducibility      | Flake-native system definitions with versioned profiles (`comfort-zone`, `diy-princess`, `studio-mode`). |
| Custom compilation   | `nix emerge` bridge turns Gentoo ebuild overlays into reproducible derivations with cached binaries.      |
| Playful polish       | Catppuccin-trans themes, `nixette-style` CLI to sync GTK/Qt/terminal styling, dynamic welcome affirmations.|
| Inclusive defaults   | Flatpak + Steam pre-set for accessibility tools, Fcitx5, Orca, speech-dispatcher, pronoun-friendly docs.  |

---

## Toolchain Concepts

- **`trans-init` installer** – Guided TUI that outputs `flake.nix`, including overlays for the `nix emerge` bridge. Provides story-mode narration for first boot.
- **`nixette-style`** – Syncs wallpapers, SDDM theme, terminal palette, Qt/KDE settings, all sourced from a YAML theme pack.
- **`emerge-optional`** – Spins up Gentoo chroots inside Nix build sandboxes for packages happiest as ebuilds. Output is cached as a Nix store derivation.
- **`affirm-d`** – Small daemon rotating `/etc/motd`, desktop notifications, and TTY colour accents with inclusive affirmations.

---

## Profile Catalogue

| Profile         | Intent                                                                                     |
|-----------------|---------------------------------------------------------------------------------------------|
| Comfort Zone    | KDE Plasma, PipeWire, Wayland, cozy defaults, automatic Catgirl cursor + emoji fonts.       |
| DIY Princess    | Minimal sway-based stack, just the flake scaffolding and overlay hooks for custom builds.   |
| Studio Mode     | Focuses on creative tooling (Krita, Blender, Ardour) and low-latency kernels, GPU tuning.   |

---

## Roadmap Sketch

1. **Moodboard → Brand Pack** (logo, icon, wallpapers, VT boot splash).
2. **Prototype flakes** – `nix flake init --template nixette#comfort-zone` etc.
3. **Gentoo overlay bridge** – Validate `nix emerge` on a handful of ebuilds (mesa, wine, gamescope).
4. **Installer draft** – BubbleTea/ratatui-driven TUI, prompts for pronouns + accessibility preferences.
5. **Community docs** – Write inclusive user guide, contributor covenant, pronoun style guide.
6. **Launch zine** – Release notes styled like a mini-comic introducing Nixie’s origin story.
7. **Accessibility audit** – Keyboard navigation, screen-reader pass, dyslexia-friendly typography options.
8. **Beta cosy jam** – Invite testers via queer sysadmin spaces; collect feedback through anonymous forms.

---

## Affirmations YAML (snippet)

```yaml
- id: bright-morning
  message: "Good morning, {name}! Your system is as valid and custom as you are."
  colour: "#F7A8B8"
- id: compile-hugs
  message: "Kernel rebuilds take time. You deserve rest breaks and gentle music."
  colour: "#55CDFC"
```

---

## Logo & Wallpaper

See `assets/nixette-logo.svg` for the primary wordmark, `assets/nixette-mascot.svg` for Nixie’s badge, and `assets/nixette-wallpaper.svg` for a 4K wallpaper concept.

### Reference Configs

- `concepts/nixette/sample_flake.nix` demonstrates the comfort-zone profile with `nix emerge`, `affirmd`, and theming hooks.

---

## Contributing Idea Seeds

- Write sample flakes showcasing the hybrid build pipeline.
- Mock up the mascot in SVG for use in documentation.
- Design additional wallpapers (night mode, pride variants, low-light).
- Draft inclusive documentation templates (issue/PR forms, community guidelines).
- Publish a community pledge emphasising safety, pronoun respect, and boundaries.
- Host monthly "compile & chill" streams to showcase contributions.

Let Nixette be the distro that compiles joy, not just binaries. 💜
