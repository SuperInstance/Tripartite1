# SuperInstance AI - Logo Design Specification

> **Version**: 1.0.0
> **Last Updated**: 2026-01-07
> **Status**: Design Specification

---

## Table of Contents

1. [Logo Concept](#logo-concept)
2. [Design Elements](#design-elements)
3. [Logo Variations](#logo-variations)
4. [Usage Guidelines](#usage-guidelines)
5. [Implementation](#implementation)
6. [File Specifications](#file-specifications)

---

## Logo Concept

### Core Concept: Tripartite Intelligence

The SuperInstance logo represents our core innovation: **three specialized AI agents working together in consensus**. The logo embodies:

1. **Tripartite Architecture**: Three nodes representing Pathos, Logos, and Ethos
2. **Interconnection**: Agents are connected, not isolated
3. **Balance**: Equal importance, collaborative decision-making
4. **Forward Motion**: Triangular shape suggests upward/forward progress
5. **Unity**: Elements form a cohesive whole

### Design Philosophy

**Minimalist yet meaningful**: Every element serves a purpose. The logo should be:
- Immediately recognizable at small sizes (favicon, app icon)
- Scalable without losing meaning (billboard to button)
- Timeless (not tied to fleeting trends)
- Distinctive (stands out in the AI/tech space)

---

## Design Elements

### The Mark (Icon)

#### Geometry

**Primary Shape**: Equilateral triangle composed of three interconnected nodes

```
        Node 1 (Pathos)
           /\
          /  \
         /    \
Node 2 (Logos)------Node 3 (Ethos)
```

**Detailed Specification**:

1. **Three Nodes**: Circular nodes, equally spaced at triangle vertices
2. **Connecting Lines**: Subtle lines connecting nodes, representing communication
3. **Node Size**: Each node is 30% of triangle height
4. **Line Thickness**: 8% of triangle height
5. **Overall Proportions**: Equilateral triangle (60° angles)

#### Styling

**Fill Colors**:
- **Node 1 (Top)**: Cyber Cyan (#00D9FF) - Pathos (Intent)
- **Node 2 (Bottom Left)**: Logic Orange (#F59E0B) - Logos (Logic)
- **Node 3 (Bottom Right)**: Verification Green (#10B981) - Ethos (Truth)

**Stroke/Connecting Lines**:
- Color: White or light gray (#E5E7EB)
- Thickness: 2px (at 512px base size)

**Effects**:
- **Subtle Gradient**: Each node has a radial gradient (lighter center, darker edge)
- **Soft Shadow**: 30% opacity black shadow, 4px blur, 2px offset
- **No glow effects**: Keep it clean and modern

### The Wordmark

#### Typography

**Font**: System sans-serif (see [Brand Guidelines](../../BRAND_GUIDELINES.md#typography))

**Weight**: 600 (Semi-Bold) for "SuperInstance", 400 (Regular) for "AI"

**Arrangement**:

```
SuperInstance™ AI
```

**Spacing**:
- Letter spacing: -0.02em ("SuperInstance"), 0.1em ("AI")
- Space between "SuperInstance" and "AI": 0.5em

**Color**:
- Primary: White on dark background, Deep Space Blue (#0A1628) on light
- Accent: "AI" can use Cyber Cyan (#00D9FF) for emphasis

### Trademark

**™ Symbol**: Use "™" (trademark) after "SuperInstance" in legal/commercial contexts. Position as superscript, small size.

**When to use**:
- Marketing materials
- Website headers
- Product packaging
- Legal documents

**When to skip**:
- Code comments
- Internal documentation
- UI elements (space-constrained)

---

## Logo Variations

### 1. Primary Logo (Horizontal)

**Layout**: Mark (left) + Wordmark (right)

```
[Icon]  SuperInstance™ AI
```

**Proportions**:
- Icon width: 25% of total logo width
- Wordmark width: 75% of total logo width
- Padding: Space equal to icon width between elements

**Usage**:
- Website headers
- Documentation headers
- Presentation titles
- Marketing materials (primary)

**Backgrounds**: White, Deep Space Blue (#0A1628), or very light gray (#F9FAFB)

---

### 2. Icon Only (Mark)

**Layout**: Single icon, no text

```
[Icon]
```

**Proportions**: Square (1:1 aspect ratio)

**Usage**:
- Application icons (desktop, mobile)
- Favicon (browser tab)
- Profile pictures (social media)
- UI elements (buttons, badges)
- Watermarks

**Minimum Size**: 32x32px (16x16px for favicon)

---

### 3. Stacked Logo (Vertical)

**Layout**: Icon (top) + Wordmark (bottom), centered alignment

```
    [Icon]
SuperInstance™ AI
```

**Proportions**:
- Icon width: 50% of wordmark width
- Vertical spacing: 20% of icon height

**Usage**:
- Social media cards
- Business cards (vertical layout)
- App splash screens
- Packaging (vertical orientation)

---

### 4. Monochrome Logo

**Layout**: Any of the above, single color

**Color Options**:
- All white (for dark backgrounds)
- All Deep Space Blue (#0A1628) (for light backgrounds)
- All black (for single-color printing)

**Usage**:
- Single-color printing (merchandise, stickers)
- Fax/photocopy scenarios
- Embossed/debossed applications
- Situations where color reproduction is unreliable

---

### 5. Dark Mode Logo

**Layout**: Primary logo, optimized for dark backgrounds

**Modifications**:
- Wordmark: White or very light gray (#E5E7EB)
- Icon nodes: Same colors (cyan, orange, green) but slightly brighter
- Connecting lines: White or light gray

**Usage**:
- Dark mode UI
- Dark-themed websites
- Presentations (dark background)
- Video overlays

---

### 6. Light Mode Logo

**Layout**: Primary logo, optimized for light backgrounds

**Modifications**:
- Wordmark: Deep Space Blue (#0A1628)
- Icon nodes: Same colors, but with slight shadow for depth
- Connecting lines: Medium gray (#6B7280)

**Usage**:
- Light mode UI
- Light-themed websites
- Printed materials (white paper)

---

## Usage Guidelines

### Clear Space

**Minimum Clear Space**: Height of the letter "S" in "SuperInstance" (at the displayed size)

**Maintain clear space around logo**:
- No text or graphics within clear space zone
- No overlapping with other elements
- Maintain consistent spacing in layouts

**Example**:

```
┌──────────────────────────────┐
│                              │ ← Clear space
│   [Icon] SuperInstance™ AI   │
│                              │ ← Clear space
└──────────────────────────────┘
    ↑                    ↑
  Clear space         Clear space
```

---

### Minimum Size

**Digital**:
- Horizontal logo: 120px wide minimum
- Icon only: 32x32px minimum (16x16px for favicon)
- Stacked: 80px wide minimum

**Print**:
- Horizontal logo: 1.5 inches (38mm) wide
- Icon only: 0.5 inches (13mm) wide
- Stacked: 1 inch (25mm) wide

**Smaller than minimum?** Use icon-only version.

---

### Backgrounds

**Approved Backgrounds**:

1. **White** (#FFFFFF)
   - Use light mode logo
   - High contrast, clean appearance

2. **Deep Space Blue** (#0A1628)
   - Use dark mode logo
   - Brand-aligned, immersive

3. **Very Light Gray** (#F9FAFB)
   - Use light mode logo
   - Subtle differentiation from white

4. **Transparent**
   - Use appropriate logo variation
   - Ensure adequate contrast with background

**Unacceptable Backgrounds**:
- Photos (unless adequate clear space and contrast)
- Patterns (too busy)
- Colors that don't meet contrast requirements
- Gradients that interfere with readability

---

### Don'ts (Common Mistakes)

❌ **DO NOT**:
- Stretch or distort the logo
- Change the relative proportions of icon and wordmark
- Alter the colors (use approved variations only)
- Rotate or skew the logo
- Add drop shadows or effects (unless specified)
- Place logo on busy backgrounds
- Crowd the logo with other elements
- Use low-resolution versions
- Change the font or letter spacing
- Separate the icon and wordmark (they're a unit)

✅ **DO**:
- Use official logo files
- Maintain aspect ratio
- Scale proportionally
- Use appropriate variation for context
- Maintain clear space
- Ensure sufficient contrast
- Test at small sizes

---

## Implementation

### SVG Format (Recommended)

**Advantages**:
- Scalable without quality loss
- Small file size
- Editable (text, colors)
- Supports transparency

**Use SVG for**:
- Websites (retina displays)
- Presentations (vector export)
- Print materials (high resolution)
- Applications (multi-density support)

**Basic SVG Structure**:

```xml
<svg width="512" height="512" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
  <!-- Definitions for gradients and shadows -->
  <defs>
    <!-- Gradients for each node -->
    <!-- Drop shadow filter -->
  </defs>

  <!-- Connecting lines -->
  <line x1="256" y1="100" x2="150" y2="380" stroke="#E5E7EB" stroke-width="8" />
  <line x1="256" y1="100" x2="362" y2="380" stroke="#E5E7EB" stroke-width="8" />
  <line x1="150" y1="380" x2="362" y2="380" stroke="#E5E7EB" stroke-width="8" />

  <!-- Three nodes with gradients -->
  <circle cx="256" cy="100" r="40" fill="url(#cyanGradient)" />
  <circle cx="150" cy="380" r="40" fill="url(#orangeGradient)" />
  <circle cx="362" cy="380" r="40" fill="url(#greenGradient)" />
</svg>
```

---

### PNG Format

**Use PNG for**:
- Applications that don't support SVG
- Email signatures
- Social media profile pictures
- Quick prototypes

**Resolution Guidelines**:
- Standard: 512x512px (2x)
- High-DPI: 1024x1024px (4x)
- Favicon: 32x32px, 16x16px

---

### Responsive Logo Implementation

**HTML/CSS Example**:

```html
<div class="logo-container">
  <img src="logo-horizontal.svg" alt="SuperInstance AI" class="logo">
</div>

<style>
.logo-container {
  display: inline-block;
}

.logo {
  width: 100%;
  max-width: 300px;  /* Adjust as needed */
  height: auto;
}

@media (max-width: 768px) {
  .logo {
    max-width: 200px;  /* Smaller on mobile */
  }
}
</style>
```

---

## File Specifications

### Naming Convention

**Format**: `{type}-{variation}.{ext}`

**Examples**:
- `logo-horizontal.svg` - Primary horizontal logo, SVG
- `logo-icon.svg` - Icon only, SVG
- `logo-stacked.png` - Stacked variation, PNG
- `logo-horizontal-dark.svg` - Dark mode horizontal, SVG

### File Structure

```
assets/logo/
├── logo.md                    (This file)
├── svg/
│   ├── logo-horizontal.svg
│   ├── logo-icon.svg
│   ├── logo-stacked.svg
│   ├── logo-monochrome.svg
│   ├── logo-horizontal-dark.svg
│   └── logo-icon-dark.svg
├── png/
│   ├── logo-horizontal-512.png
│   ├── logo-horizontal-1024.png
│   ├── logo-icon-32.png
│   ├── logo-icon-512.png
│   └── logo-stacked-512.png
└── favicon/
    ├── favicon-16x16.png
    ├── favicon-32x32.png
    └── favicon.ico
```

### Color Values (CSS)

```css
:root {
  /* Brand Colors */
  --color-deep-space-blue: #0A1628;
  --color-cyber-cyan: #00D9FF;
  --color-neural-purple: #8B5CF6;
  --color-logic-orange: #F59E0B;
  --color-verification-green: #10B981;
  --color-white: #FFFFFF;
  --color-light-gray: #E5E7EB;
  --color-medium-gray: #6B7280;
  --color-dark-gray: #1F2937;
}
```

---

## Usage Examples

### Website Header

```html
<header>
  <div class="logo">
    <img src="assets/logo/svg/logo-horizontal.svg"
         alt="SuperInstance AI"
         width="200"
         height="50">
  </div>
  <nav>...</nav>
</header>
```

### Document Title Page

```markdown
<div align="center">

![SuperInstance AI](assets/logo/svg/logo-stacked.svg)

# SuperInstance AI
## Privacy-First Local AI with Tripartite Consensus

</div>
```

### Business Card

**Front**:
```
┌────────────────────────────┐
│                            │
│   [Icon]   SuperInstance   │
│              AI            │
│                            │
│   your.name@example.com    │
│   https://superinstance.ai │
└────────────────────────────┘
```

### Email Signature

```
[Icon] SuperInstance™ AI
Your Name | Title
Email | Phone | Website
```

---

## Logo Animation (Optional)

For digital applications, subtle animation can enhance the logo:

### Subtle Pulse

**Effect**: Nodes gently pulse in sequence (Pathos → Logos → Ethos)

**Duration**: 3 seconds per cycle

**Implementation**: CSS or SVG animations

**Use Cases**:
- Website homepage
- App loading screens
- Presentation transitions

**Note**: Animation should be subtle, not distracting. Use sparingly.

---

## Accessibility

### Alt Text

**For screen readers**, always provide descriptive alt text:

```html
<img src="logo-horizontal.svg"
     alt="SuperInstance AI - Privacy-first AI with tripartite consensus">
```

**Short version** (for repeated use):
```html
<img src="logo-icon.svg" alt="SuperInstance AI logo">
```

### Color Contrast

**Ensure sufficient contrast**:
- Logo on background: Minimum 4.5:1 (WCAG AA)
- Icon nodes on background: Minimum 3:1 (large elements)

### Logo as Link

**When logo links to homepage**:
```html
<a href="/">
  <img src="logo-horizontal.svg" alt="SuperInstance AI - Home">
</a>
```

---

## Brand Consistency

### Logo Lockup with Tagline

**Full lockup** (formal use):

```
[Icon]  SuperInstance™ AI
        Your AI, Your Way, Your Privacy.
```

**Rules**:
- Tagline in smaller, lighter font
- Align text left under wordmark
- Use only in marketing/intro contexts

### Co-Branding

**When using with other logos**:
- Maintain clear space around SuperInstance logo
- Ensure equal visual weight (unless one is clearly primary)
- Use neutral divider (vertical line or spacing)

**Example**:
```
[Partner Logo]    |    [SuperInstance Logo]
```

---

## Quality Control

### Pre-Flight Checklist

Before using the logo in production:

- [ ] Using official logo file (not a screenshot or copy)
- [ ] Appropriate variation for context (horizontal, icon, stacked)
- [ ] Sufficient clear space maintained
- [ ] Contrast meets accessibility standards
- [ ] Size meets minimum requirements
- [ ] File format appropriate for use case
- [ ] Alt text provided (for digital use)
- [ ] Colors match approved palette

---

## Resources

### Logo Files

All official logo files are in:
- `/assets/logo/svg/` - Vector (SVG) files
- `/assets/logo/png/` - Raster (PNG) files
- `/assets/logo/favicon/` - Favicons

### Related Documentation

- [Brand Guidelines](../../BRAND_GUIDELINES.md) - Complete brand system
- [Social Preview Spec](../../SOCIAL_PREVIEW.md) - Social media images

---

## Version History

**v1.0.0** (2026-01-07)
- Initial logo design specification
- Defined all logo variations
- Established usage guidelines

---

## Design Notes

### Future Enhancements

**Potential additions** (when needed):
- Animated logo (loading states, transitions)
- 3D version (for product videos)
- Sticker/merchandise variations

### Design Rationale

**Why a triangle?**
- Represents stability and strength
- Three vertices = three agents
- Points upward = progress, aspiration
- One of the strongest structural shapes

**Why three colors?**
- Each agent has distinct identity
- Colors map to agent personalities
- Visual diversity while maintaining harmony
- Easy to remember and recognize

**Why simple geometry?**
- Scalable to any size
- Timeless, not tied to trends
- Easy to reproduce (print, digital, merchandise)
- Accessible and clear

---

**SuperInstance AI** - *Your AI, Your Way, Your Privacy.*

*For the latest logo assets and guidelines, check the GitHub repository.*
