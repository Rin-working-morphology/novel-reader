---
name: Novel Reader
description: A discreet local desktop reader for TXT and EPUB novels.
colors:
  light-reading-surface: "#f3ead3"
  light-text: "#5c6a72"
  light-accent: "#8da101"
  light-border: "#ddd8be"
  light-hover: "#e5e6c5"
  dark-reading-surface: "#1a1b26"
  dark-text: "#c0caf5"
  dark-accent: "#bb9af7"
  dark-border: "#414868"
  dark-hover: "#2f354f"
  transparent: "#00000000"
typography:
  title:
    fontFamily: "LXGW Neo XiHei, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0"
  body:
    fontFamily: "LXGW Neo XiHei, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.8
    letterSpacing: "0"
  label:
    fontFamily: "LXGW Neo XiHei, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0"
rounded:
  sm: "6px"
  md: "8px"
  pill: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
  reading-y: "120px"
components:
  button-primary:
    backgroundColor: "{colors.transparent}"
    textColor: "{colors.light-accent}"
    rounded: "{rounded.sm}"
    padding: "0 14px"
    height: "34px"
  reading-body:
    backgroundColor: "{colors.light-reading-surface}"
    textColor: "{colors.light-text}"
    rounded: "{rounded.md}"
    padding: "120px 24px"
  chrome-panel:
    backgroundColor: "{colors.light-reading-surface}"
    textColor: "{colors.light-text}"
    rounded: "{rounded.sm}"
    padding: "16px"
---

# Design System: Novel Reader

## 1. Overview

**Creative North Star: "The Floating Page"**

The interface exists to protect the act of reading. It should feel low-profile, lightweight, and eye-friendly, with chrome that appears only to orient the user and then steps back. The strongest visual signal is the body text; everything else is supporting structure.

This system rejects complex library-management behavior and commercial-reader styling. Current implementation details are not binding design doctrine. If an existing structure does not serve reading, such as a half-collapsed right outline with weak workflow value, remove or redesign it according to product UI best practice.

**Key Characteristics:**
- Discreet application chrome around a calm reading surface.
- Restrained color, used for actions and current state instead of decoration.
- Dense enough for desktop utility, quiet enough for long reading sessions.
- Transparent mode as the lightest state: readable floating text with minimal surrounding UI.

## 2. Colors

The palette is restrained: warm light reading tones, a quiet ink color, a small olive accent, and a dark mode that preserves contrast without turning the app into a decorative dark dashboard.

### Primary
- **Quiet Olive Accent** (`light-accent`): Primary action, selected state, and useful orientation only. It should stay rare.
- **Soft Violet Accent** (`dark-accent`): Dark-mode equivalent for primary actions and selected state. Do not expand it into gradients or decorative glow.

### Neutral
- **Light Reading Surface** (`light-reading-surface`): Main light-mode page and app surface.
- **Quiet Ink** (`light-text`): Primary text on the light reading surface.
- **Soft Reed Border** (`light-border`): Dividers, panel boundaries, and subtle containment.
- **Dark Reading Surface** (`dark-reading-surface`): Main dark-mode page and app surface.
- **Moonlit Text** (`dark-text`): Primary text on the dark reading surface.
- **Night Border** (`dark-border`): Dividers and panel boundaries in dark mode.
- **Transparent Surface** (`transparent`): Used only for transparent mode, where chrome should visually disappear.

### Named Rules
**The Ten Percent Accent Rule.** Accent color is reserved for primary action, active state, and focus. If the page starts to look green or purple, the accent is overused.

**The Transparent Means Lighter Rule.** Transparent mode should remove chrome before it removes legibility. Body text contrast must remain readable even when panels vanish.

## 3. Typography

**Display Font:** LXGW Neo XiHei with system-ui fallback
**Body Font:** LXGW Neo XiHei with system-ui fallback
**Label/Mono Font:** Fira Code only for code-like or debug surfaces, not normal reader UI

**Character:** A single sans family keeps the product familiar and low-friction. Reading typography should be calm, not expressive; hierarchy comes from weight, spacing, and placement.

### Hierarchy
- **Display** (not a default role): Avoid hero-scale display type in the app shell.
- **Headline** (600, 18-20px, 1.3): Use for modal or panel titles when needed.
- **Title** (600, 16px, 1.4): Current book title, section titles, and major controls.
- **Body** (400, 16px, 1.8): Novel content and readable prose. Keep long-form text comfortable and avoid crowding line height.
- **Label** (400-500, 12-14px, 1.4, normal case): Metadata, file size, secondary actions, and compact navigation labels.

### Named Rules
**The Body Text Wins Rule.** When a component competes visually with the novel text, reduce the component before reducing the text.

## 4. Elevation

The system is flat by default and uses tonal layering, borders, and transparency instead of heavy shadows. Shadows are not part of the core vocabulary. Transparent mode may use light backdrop treatment only when it materially improves text readability.

### Named Rules
**The No Decorative Glass Rule.** Blur and translucent panels are allowed only to keep floating text or controls readable. They are not a default card style.

## 5. Components

### Buttons
- **Shape:** Small-radius utility controls (6px) or circular icon buttons where the icon is the affordance.
- **Primary:** Use the accent color for the main folder-selection action and other rare primary actions.
- **Hover / Focus:** Hover should be a subtle tonal shift. Focus must be visible and should use the accent color or a high-contrast outline.
- **Ghost / Icon:** Header controls should remain quiet and rely on recognizable icons with tooltips where labels are hidden.

### Cards / Containers
- **Corner Style:** Lightly rounded surfaces (6-8px). Do not create large decorative cards.
- **Background:** Use the current theme surface. In transparent mode, remove panel backgrounds where possible.
- **Shadow Strategy:** Flat by default. Prefer borders and tonal states.
- **Border:** Thin 1px dividers only.
- **Internal Padding:** 16px for utility panels, 24px for readable content containers.

### Inputs / Fields
- **Style:** Match Naive UI form controls but tune colors to the reading palette.
- **Focus:** Clear accent outline or border shift.
- **Error / Disabled:** Use semantic state colors sparingly; do not decorate normal states as warnings.

### Navigation
- **Style:** File list, chapter picker, and outline are support controls. They should collapse, hide, or simplify when the user is reading.
- **Active State:** Use tonal background plus readable text. Avoid thick side stripes.
- **Transparent Mode:** Navigation should not remain as a visible chrome scaffold unless the user explicitly opens it.

### Reading Body
- **Style:** Long-form text is the signature component. Preserve line-height, text color contrast, and stable scroll behavior.
- **Chapter Transitions:** Previous/next chapter affordances should appear only near boundaries and should not obscure the text.

## 6. Do's and Don'ts

### Do:
- **Do** keep the product register: familiar desktop tool patterns, restrained motion, and predictable controls.
- **Do** make transparent mode the lightest UI state, with floating readable body text as the priority.
- **Do** remove or redesign existing structures when they do not serve reading, including the current half-collapsed outline pattern.
- **Do** keep accent color rare and meaningful.
- **Do** verify contrast for body text in light, dark, and transparent contexts.

### Don't:
- **Don't** make this feel like a complex library-management app.
- **Don't** make it feel like a commercial reading platform, storefront, or recommendation product.
- **Don't** preserve current rough layout decisions as design requirements.
- **Don't** add decorative gradients, glass cards, heavy shadows, or large rounded cards.
- **Don't** let file management, chapter controls, or settings compete with the reading body.
