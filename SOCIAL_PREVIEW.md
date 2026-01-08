# SuperInstance AI - Social Preview Image Specification

> **Version**: 1.0.0
> **Last Updated**: 2026-01-07
> **Status**: Design Specification
> **Purpose**: GitHub social preview, Twitter cards, Open Graph images

---

## Table of Contents

1. [Overview](#overview)
2. [Design Specifications](#design-specifications)
3. [Layout Grid](#layout-grid)
4. [Content Elements](#content-elements)
5. [Visual Style](#visual-style)
6. [Variations](#variations)
7. [File Formats](#file-formats)
8. [Implementation](#implementation)

---

## Overview

### Purpose

The social preview image is the **first impression** for SuperInstance AI on social media platforms, GitHub repository previews, and link sharing. It must communicate:

1. **Project Identity**: What is SuperInstance AI?
2. **Key Value Proposition**: Why should someone care?
3. **Trust & Credibility**: Professional, polished appearance
4. **Call to Action**: What should the viewer do?

### Platforms

**Primary**:
- GitHub repository preview (when shared on Twitter, LinkedIn, etc.)
- Twitter card (when linking to repo)
- LinkedIn article preview
- Discord embed previews

**Secondary**:
- Open Graph image (general link sharing)
- Email newsletter headers
- Presentation title slides

---

## Design Specifications

### Dimensions

**Primary Size**: **1280 x 640 pixels** (2:1 aspect ratio)

**Why this size**:
- Twitter card recommendation
- GitHub social preview standard
- Open Graph preferred ratio
- Good balance for most platforms

**Alternative Sizes**:
- Square (1:1): 1200 x 1200px (Instagram, some LinkedIn contexts)
- Vertical (16:9): 1280 x 720px (YouTube thumbnail)
- Story (9:16): 1080 x 1920px (Instagram Stories, TikTok)

### File Format

**Recommended**: **PNG** for lossless quality and transparency support

**Alternatives**:
- **JPEG**: Smaller file size, but no transparency
- **WebP**: Modern format, smaller size, but wider platform support varies

**File Size**: Under 500KB for optimal loading

### Color Scheme

**Background**: Gradient from Deep Space Blue to darker shade

```
Start (Top): #0A1628 (Deep Space Blue)
End (Bottom): #050A12 (Darker, 50% darker)
```

**Gradient Direction**: Diagonal (top-left to bottom-right)

**Rationale**: Dark background conveys sophistication, makes content pop, and aligns with brand colors.

---

## Layout Grid

### Grid System

**Columns**: 12 columns, 80px each, 20px gutters

**Total Content Width**: 1200px (40px margins on sides)

**Rows**: 6 rows, approximately 100px each

### Content Areas

```
┌──────────────────────────────────────────────────────┐
│  Row 1: Logo + Tagline (Top 20%)                      │
├──────────────────────────────────────────────────────┤
│  Row 2-4: Hero Content (Middle 50%)                   │
│  - Project Name (Large)                               │
│  - Tagline/Subtitle                                   │
│  - Key Feature/Value Prop                             │
├──────────────────────────────────────────────────────┤
│  Row 5-6: Footer (Bottom 30%)                         │
│  - Website URL                                        │
│  - GitHub URL                                         │
│  - Badge/Status                                       │
└──────────────────────────────────────────────────────┘
```

---

## Content Elements

### Top Section (Logo + Tagline)

**Left Side (20%)**: Icon only logo
- Logo size: 80x80px
- Position: 40px from left, 40px from top

**Right Side (80%)**: Tagline
- Text: "Privacy-First AI with Tripartite Consensus"
- Font: Semi-bold (600 weight)
- Size: 24px
- Color: White (#FFFFFF)
- Position: Centered vertically, 40px right padding

**Visual**:
```
[Icon]  Privacy-First AI with Tripartite Consensus
```

---

### Hero Section (Middle 50%)

**Primary Title**: "SuperInstance AI"
- Font: Bold (700 weight)
- Size: 72px
- Color: White (#FFFFFF)
- Position: Centered horizontally
- Spacing: 40px from top section

**Subtitle/Tagline**: "Your AI, Your Way, Your Privacy."
- Font: Regular (400 weight)
- Size: 32px
- Color: Cyber Cyan (#00D9FF) for emphasis
- Position: Centered, 20px below title

**Key Feature** (Value Proposition):
- Text: "Three AI Agents. One Consensus. Zero Compromises."
- Font: Semi-bold (600 weight)
- Size: 28px
- Color: White (#FFFFFF)
- Position: Centered, 40px below subtitle
- Optional: Add small icons for visual interest

**Visual Hierarchy**:
```
        SuperInstance AI
    Your AI, Your Way, Your Privacy.

   Three AI Agents. One Consensus. Zero Compromises.
```

---

### Bottom Section (Footer)

**Left Side**: Website URL
- Text: "superinstance.ai"
- Font: Monospace (for technical feel)
- Size: 18px
- Color: Light gray (#E5E7EB)
- Position: 40px from left, 40px from bottom

**Right Side**: GitHub URL + Badge
- Text: "github.com/SuperInstance/Tripartite1"
- Font: Monospace
- Size: 18px
- Color: Light gray (#E5E7EB)
- Position: 40px from right, 40px from bottom

**Badge** (Optional):
- Text: "v0.2.0 | 250+ Tests | Production Ready"
- Font: Regular
- Size: 14px
- Color: White
- Background: Cyber Cyan (#00D9FF) or Logic Orange (#F59E0B)
- Shape: Rounded rectangle (pill shape)
- Position: Centered, 20px from bottom

**Visual**:
```
superinstance.ai                v0.2.0 | Production Ready
                        github.com/SuperInstance/Tripartite1
```

---

### Visual Elements

#### Background Pattern (Optional)

**Subtle geometric pattern**: Faint triangular grid overlay

**Implementation**:
- SVG pattern or PNG overlay
- Opacity: 5-10% (very subtle)
- Color: White
- Purpose: Add depth without clutter

**Alternative**: Gradient only (cleaner, simpler)

---

#### Decorative Elements

**Tripartite Icon** (Large, subtle):
- Position: Centered, behind hero text
- Size: 400x400px
- Opacity: 10-15%
- Color: White or Cyber Cyan
- Purpose: Reinforce tripartite concept

**Connecting Lines** (Subtle):
- Three nodes arranged in triangle
- Connected with lines
- Fading edges (gradient transparency)

---

## Visual Style

### Typography

**Font Family**: System sans-serif (see [Brand Guidelines](../BRAND_GUIDELINES.md))

**Font Weights**:
- Title: 700 (Bold)
- Subtitle: 400 (Regular)
- URLs: Monospace

**Alignment**: Centered (for main content), Left/Right (for footer)

---

### Color Usage

**Primary**: White (#FFFFFF) for main text
- High contrast on dark background
- Readable, professional

**Accent**: Cyber Cyan (#00D9FF) for emphasis
- Tagline, key words, badges
- Pops without overwhelming

**Secondary**: Light gray (#E5E7EB) for metadata
- URLs, less important info
- Lower visual hierarchy

**Background**: Gradient (#0A1628 → #050A12)
- Creates depth
- Professional, modern feel

---

### Visual Hierarchy

1. **Title** (SuperInstance AI) - Most important (72px, bold)
2. **Subtitle** (Your AI...) - Secondary (32px, cyan accent)
3. **Key Feature** - Supporting (28px, semi-bold)
4. **URLs** - Least important (18px, monospace, gray)

---

### Spacing

**Margins**: 40px on all sides
**Line Spacing**: 1.2x for title, 1.4x for body
**Section Spacing**: 40px between major sections

---

## Variations

### Variation 1: Feature Focus

**Emphasis**: Highlight a specific feature (e.g., Privacy, Consensus, Local-First)

**Changes**:
- Replace "Three AI Agents..." with specific feature
- Add relevant icon or illustration
- Adjust color accent based on feature

**Example** (Privacy Focus):
```
    SuperInstance AI
Your AI, Your Way, Your Privacy.

  🔒 Your Data Never Leaves Your Device
     18 Redaction Patterns. Zero Leaks.
```

---

### Variation 2: Release Announcement

**Emphasis**: New version or milestone

**Changes**:
- Add "NEW" or "v0.2.0" badge (prominent)
- Highlight key features of release
- Add excitement/confetti elements (subtle)

**Example**:
```
    ✨ SuperInstance AI v0.2.0
  Phase 2: Cloud Mesh Now Available!

  • QUIC Tunnel with mTLS
  • Device Telemetry & Heartbeat
  • Cloud Escalation Client
```

---

### Variation 3: Minimalist

**Emphasis**: Clean, simple, brand-focused

**Changes**:
- Remove subtitle
- Larger title
- More negative space
- Simple geometric elements only

**Example**:
```
         SuperInstance AI
    Privacy-First. Local-First.
```

---

### Variation 4: Action-Oriented

**Emphasis**: Call to action (star, fork, contribute)

**Changes**:
- Add CTA button (visual only)
- Highlight community aspect
- Show contribution stats

**Example**:
```
    SuperInstance AI
  Join the Privacy-First AI Revolution

        ⭐ Star on GitHub
        🍴 Fork to Contribute
```

---

## File Formats

### PNG (Primary)

**Use Cases**: General social sharing, GitHub previews

**Specifications**:
- Size: 1280x640px
- Format: PNG-24 (lossless)
- Transparency: Not required (solid background)
- Color: sRGB color profile

**Export Settings**:
- Resolution: 72 DPI (digital)
- Compression: None (PNG is lossless)
- Optimize: Use TinyPNG or similar for file size reduction

---

### JPEG (Alternative)

**Use Cases**: Email marketing, platforms that prefer JPEG

**Specifications**:
- Size: 1280x640px
- Quality: 90% (high quality, reasonable file size)
- Color: sRGB color profile

**Note**: No transparency in JPEG

---

### SVG (Ideal, but Limited Support)

**Use Cases**: Future-proofing, scaling

**Specifications**:
- ViewBox: 0 0 1280 640
- Vector text (outlined for compatibility)
- Embedded base64 images (if any)

**Limitation**: Not all platforms support SVG for social previews

---

## Implementation

### HTML Meta Tags (For Social Sharing)

Add to your HTML `<head>` section:

```html
<!-- Open Graph / Facebook -->
<meta property="og:type" content="website">
<meta property="og:url" content="https://github.com/SuperInstance/Tripartite1">
<meta property="og:title" content="SuperInstance AI - Privacy-First AI with Tripartite Consensus">
<meta property="og:description" content="Your AI, Your Way, Your Privacy. Three specialized AI agents collaborate on every query for accuracy, safety, and relevance.">
<meta property="og:image" content="https://github.com/SuperInstance/Tripartite1/raw/main/assets/social-preview.png">

<!-- Twitter -->
<meta property="twitter:card" content="summary_large_image">
<meta property="twitter:url" content="https://github.com/SuperInstance/Tripartite1">
<meta property="twitter:title" content="SuperInstance AI - Privacy-First AI with Tripartite Consensus">
<meta property="twitter:description" content="Your AI, Your Way, Your Privacy. Three specialized AI agents collaborate on every query for accuracy, safety, and relevance.">
<meta property="twitter:image" content="https://github.com/SuperInstance/Tripartite1/raw/main/assets/social-preview.png">
```

---

### File Location

**Place social preview image** in repository root:

```
/mnt/c/claudesuperinstance/
├── assets/
│   └── social-preview.png  (1280x640px)
├── SOCIAL_PREVIEW.md       (This file)
└── README.md
```

**Why root level?**: GitHub automatically looks for `social-preview.png` or `og-image.png` in the repository root.

---

### Figma Design File (Template)

**For easy creation**, use a Figma template:

**Frame Setup**:
- Desktop frame: 1280x640px
- Grid: 12 columns, 80px wide, 20px gutters
- Margins: 40px

**Layers** (bottom to top):
1. Background (rectangle with gradient)
2. Optional pattern overlay
3. Optional decorative icon (subtle)
4. Logo (top left)
5. Title (centered, large)
6. Subtitle (centered)
7. Key feature (centered)
8. Footer URLs (bottom left/right)
9. Badge (bottom center, optional)

**Export**:
- Format: PNG
- Scale: 1x (1280x640px)
- Prefix: social-preview

---

## Quality Checklist

Before deploying, verify:

- [ ] Correct dimensions (1280x640px)
- [ ] File size under 500KB
- [ ] All text is legible at small sizes
- [ ] Color contrast meets accessibility standards
- [ ] No typos or grammatical errors
- [ ] URL is correct
- [ ] Version number is accurate (if shown)
- [ ] Brand colors are accurate
- [ ] Logo is crisp and properly aligned
- [ ] Meta tags are configured correctly

---

## Testing

### Preview Tools

**Test your social preview** before deploying:

1. **Twitter Card Validator**: https://cards-dev.twitter.com/validator
2. **LinkedIn Post Inspector**: https://www.linkedin.com/post-inspector/
3. **Open Graph Debugger**: https://www.opengraph.xyz/
4. **Facebook Sharing Debugger**: https://developers.facebook.com/tools/debug/

**What to test**:
- Image loads correctly
- Title, description, image appear as expected
- No caching issues (clear cache if needed)

---

### A/B Testing

**If you have traffic**, test variations:

**Metrics to track**:
- Click-through rate (CTR)
- Engagement (likes, shares, comments)
- Conversion (stars, forks, visits)

**Variations to test**:
- Different taglines
- Different feature highlights
- Different color accents
- Minimalist vs. detailed

---

## Examples

### Example 1: General Purpose (Recommended)

```
┌─────────────────────────────────────────────────────┐
│ [Icon]  Privacy-First AI with Tripartite Consensus │
│                                                     │
│              SuperInstance AI                       │
│        Your AI, Your Way, Your Privacy.            │
│                                                     │
│    Three AI Agents. One Consensus. Zero Compromises│
│                                                     │
│superinstance.ai    v0.2.0 | Production Ready       │
│             github.com/SuperInstance/Tripartite1   │
└─────────────────────────────────────────────────────┘
```

### Example 2: Feature Focus (Privacy)

```
┌─────────────────────────────────────────────────────┐
│ [Icon]         🔒 Privacy-First AI                  │
│                                                     │
│              SuperInstance AI                       │
│      Your Data Never Leaves Your Device            │
│                                                     │
│           18 Redaction Patterns. Zero Leaks.       │
│                                                     │
│superinstance.ai    🔒 100% Local Processing        │
│             github.com/SuperInstance/Tripartite1   │
└─────────────────────────────────────────────────────┘
```

### Example 3: Minimalist

```
┌─────────────────────────────────────────────────────┐
│ [Icon]                                             │
│                                                     │
│              SuperInstance AI                       │
│        Privacy-First. Local-First. Consensus.       │
│                                                     │
│                                                     │
│superinstance.ai                                     │
│             github.com/SuperInstance/Tripartite1   │
└─────────────────────────────────────────────────────┘
```

---

## Resources

### Design Tools

- **Figma**: https://www.figma.com/ (free, collaborative)
- **Canva**: https://www.canva.com/ (templates available)
- **Adobe Express**: https://www.adobe.com/express/ (free)

### Image Optimization

- **TinyPNG**: https://tinypng.com/ (PNG compression)
- **Squoosh**: https://squoosh.app/ (Google's image optimizer)
- **ImageOptim**: https://imageoptim.com/ (Mac app)

### Related Documentation

- [Brand Guidelines](../BRAND_GUIDELINES.md) - Complete brand system
- [Logo Specification](logo/logo.md) - Logo design and usage

---

## Version History

**v1.0.0** (2026-01-07)
- Initial social preview specification
- Defined layout grid and content elements
- Provided implementation guidelines

---

## Next Steps

1. **Create the social preview image** using Figma, Canva, or similar tool
2. **Test** using Twitter Card Validator and other debuggers
3. **Optimize** file size (under 500KB)
4. **Deploy** to repository root (`assets/social-preview.png`)
5. **Configure** meta tags in HTML head
6. **Monitor** analytics and iterate based on performance

---

**SuperInstance AI** - *Your AI, Your Way, Your Privacy.*

*For the latest branding assets, check the GitHub repository.*
