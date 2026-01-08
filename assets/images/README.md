# SuperInstance AI - Image Assets

> **Version**: 1.0.0
> **Last Updated**: 2026-01-07
> **Status**: Asset Inventory & Requirements

---

## Table of Contents

1. [Overview](#overview)
2. [Current Images](#current-images)
3. [Needed Images](#needed-images)
4. [Image Specifications](#image-specifications)
5. [Naming Conventions](#naming-conventions)
6. [Creation Guidelines](#creation-guidelines)
7. [Diagram Standards](#diagram-standards)
8. [Screenshots](#screenshots)

---

## Overview

This directory contains all visual assets for SuperInstance AI, including:
- Architecture diagrams
- Feature showcase images
- Screenshots
- Conceptual illustrations
- Technical diagrams

**Purpose**: Provide high-quality, consistent visuals for documentation, marketing, and presentations.

---

## Current Images

### Architecture Diagrams

**Status**: To be created based on ASCII art in documentation

**Planned Diagrams**:

1. **High-Level Architecture** (`architecture-overview.png`)
   - Source: [ARCHITECTURE.md](../../ARCHITECTURE.md)
   - Shows: User → Local Hub → Privacy Proxy → Cloud
   - Style: Clean, layered architecture

2. **Tripartite Council** (`tripartite-council.png`)
   - Source: [README.md](../../README.md)
   - Shows: Pathos, Logos, Ethos collaboration
   - Style: Three agents with consensus flow

3. **Privacy Proxy** (`privacy-proxy.png`)
   - Source: [ARCHITECTURE.md](../../ARCHITECTURE.md)
   - Shows: Redaction and re-inflation flow
   - Style: Data flow diagram with before/after

4. **Knowledge Vault** (`knowledge-vault.png`)
   - Source: [README.md](../../README.md)
   - Shows: RAG pipeline, vector search
   - Style: Flowchart with document processing

5. **Cloud Mesh (Phase 2)** (`cloud-mesh.png`)
   - Source: [Phase 2 Roadmap](../../phases/PHASE_2_DETAILED_ROADMAP.md)
   - Shows: QUIC tunnel, Cloudflare integration
   - Style: Network diagram with secure tunnel

---

### Feature Showcase Images

**Status**: Conceptual only, awaiting creation

**Planned Images**:

1. **Privacy First** (`feature-privacy.png`)
   - Shows: Redaction patterns in action
   - Style: Before/after comparison
   - Use Case: Marketing, README

2. **Local-First Processing** (`feature-local.png`)
   - Shows: Local inference vs. cloud
   - Style: Side-by-side comparison
   - Use Case: Marketing, README

3. **Tripartite Consensus** (`feature-consensus.png`)
   - Shows: Three agents deliberating
   - Style: Animated or illustrated
   - Use Case: Explainer video, README

4. **Knowledge Vault RAG** (`feature-rag.png`)
   - Shows: Document retrieval and answer generation
   - Style: Flowchart with real document example
   - Use Case: Documentation, tutorial

5. **Hardware Detection** (`feature-hardware.png`)
   - Shows: Automatic hardware recognition
   - Style: System UI mockup
   - Use Case: Documentation, tutorial

---

### Screenshots

**Status**: To be captured

**Needed Screenshots**:

1. **CLI - Init Command** (`screenshot-init.png`)
   - Shows: `synesis init` output
   - Terminal theme: Dark with good contrast
   - Resolution: 1200x800px minimum

2. **CLI - Status Command** (`screenshot-status.png`)
   - Shows: `synesis status` table output
   - Terminal theme: Dark
   - Resolution: 1200x800px

3. **CLI - Query Example** (`screenshot-query.png`)
   - Shows: `synesis ask` with consensus output
   - Terminal theme: Dark
   - Resolution: 1200x800px

4. **CLI - Knowledge Commands** (`screenshot-knowledge.png`)
   - Shows: Knowledge vault operations
   - Terminal theme: Dark
   - Resolution: 1200x800px

5. **Desktop App (Future)** (`screenshot-desktop.png`)
   - Shows: Desktop GUI (Phase 4)
   - Style: Modern, clean UI
   - Resolution: 1920x1080px

---

## Needed Images

### High Priority

1. **Architecture Overview** - Essential for understanding system
2. **Tripartite Council Diagram** - Core innovation
3. **Privacy Flow Diagram** - Key differentiator
4. **CLI Screenshots** - Show, don't just tell

### Medium Priority

5. **Knowledge Vault RAG** - Explain how RAG works
6. **Feature Showcase Images** - Marketing materials
7. **Phase 2 Cloud Mesh** - Show progress and direction

### Low Priority

8. **Animated Explainer** - Video/gif for social media
9. **Infographic** - Single-page overview
10. **Comparison Charts** - Benchmarks, performance

---

## Image Specifications

### Technical Specifications

**Format**:
- **Diagrams**: SVG (preferred) or PNG (minimum 1200px wide)
- **Screenshots**: PNG (lossless)
- **Photos/Presentations**: JPEG (high quality)

**Resolution**:
- **Diagrams**: Minimum 1200px wide, 2x for retina (2400px)
- **Screenshots**: 1920x1080px (Full HD) or higher
- **Thumbnails**: 400px wide (for previews)

**Color**:
- Use brand colors (see [Brand Guidelines](../../BRAND_GUIDELINES.md))
- Dark mode preferred for technical diagrams
- High contrast for readability

**Typography**:
- System fonts for consistency
- Minimum 14px for legibility
- Monospace for code/commands

**File Size**:
- Diagrams: Under 500KB each
- Screenshots: Under 1MB each
- Optimize with TinyPNG or similar

---

## Naming Conventions

### Format

`{category}-{description}.{variation}.{ext}`

**Examples**:
- `architecture-overview.png` (Main architecture diagram)
- `architecture-tripartite-flow.svg` (Tripartite flow diagram)
- `feature-privacy-showcase.png` (Privacy feature showcase)
- `screenshot-cli-init.png` (Init command screenshot)
- `screenshot-cli-query-dark.png` (Query screenshot, dark mode)

### Categories

- **architecture**: System architecture, technical diagrams
- **feature**: Feature showcases, marketing images
- **screenshot**: CLI or app screenshots
- **concept**: Conceptual illustrations, not tied to specific features
- **logo**: Logo variations (see [logo/](logo/))
- **social**: Social media images (see root level)

---

## Creation Guidelines

### Diagram Tools

**Recommended Tools**:

1. **Mermaid.js** (Code-to-diagram)
   - Integrated with Markdown
   - Version control friendly
   - Simple, clean diagrams
   - Example: [ARCHITECTURE.md](../../ARCHITECTURE.md)

2. **Excalidraw** (Hand-drawn style)
   - Web-based, free
   - Hand-drawn aesthetic (approachable)
   - Export to SVG/PNG
   - URL: https://excalidraw.com/

3. **draw.io / diagrams.net**
   - Desktop or web
   - Professional diagrams
   - Export to SVG/PNG
   - URL: https://app.diagrams.net/

4. **Figma** (Professional design)
   - Collaborative design tool
   - Advanced features
   - Export to SVG/PNG
   - URL: https://www.figma.com/

5. **PlantUML** (Code-to-UML)
   - Text-based diagrams
   - Version control friendly
   - Good for sequence diagrams
   - URL: https://plantuml.com/

---

### Diagram Best Practices

**DO**:
- Use consistent colors (brand palette)
- Maintain clear visual hierarchy
- Label all components clearly
- Use arrows to show flow/direction
- Include legend if needed
- Keep it simple (avoid clutter)
- Use consistent stroke widths
- Align elements precisely

**DON'T**:
- Overcrowd with too many elements
- Use too many colors (stick to brand palette)
- Make text too small (minimum 14px)
- Use inconsistent styles
- Forget to label abbreviations
- Mix diagram types within one diagram

---

## Diagram Standards

### Architecture Diagrams

**Purpose**: Show system components and relationships

**Elements**:
- **Components**: Rectangles with rounded corners
- **Databases**: Cylinders
- **External Systems**: Cloud shape
- **Data Flow**: Arrows with labels
- **Grouping**: Dotted rectangles for subsystems

**Colors**:
- **Local Components**: Deep Space Blue (#0A1628) background
- **Cloud Components**: Cyber Cyan (#00D9FF) accents
- **Data Flow**: White or light gray (#E5E7EB) arrows
- **Privacy Elements**: Verification Green (#10B981)

**Example Structure**:
```
┌────────────────────────────────────────┐
│         User Interface Layer           │
├────────────────────────────────────────┤
│                                        │
│  ┌────────┐    ┌─────────┐            │
│  │ Pathos │───→│ Consensus│           │
│  └────────┘    │  Engine  │           │
│  ┌────────┐    └────┬─────┘           │
│  │  Logos │────────┤                  │
│  └────────┘       │                  │
│  ┌────────┐       │                  │
│  │  Ethos │───────┘                  │
│  └────────┘                          │
└────────────────────────────────────────┘
```

---

### Flow Diagrams

**Purpose**: Show process flow, data transformations

**Elements**:
- **Start/End**: Rounded rectangles (pill shape)
- **Process**: Rectangles
- **Decision**: Diamonds
- **Data**: Parallelograms
- **Arrows**: Solid lines with arrowheads

**Colors**:
- **Standard Process**: Deep Space Blue background
- **Decision Points**: Logic Orange (#F59E0B) border
- **Success States**: Verification Green (#10B981)
- **Error States**: Error Red (#EF4444)

**Example: Privacy Flow**
```
┌──────────┐
│  User    │
│  Query   │
└─────┬────┘
      │
      ▼
┌─────────────┐
│ Privacy     │
│ Proxy       │
│ (Redact)    │
└─────┬───────┘
      │
      ▼
┌─────────────┐
│  Tripartite │
│  Council    │
└─────┬───────┘
      │
      ▼
┌─────────────┐
│  Local or   │
│  Cloud      │
└─────┬───────┘
      │
      ▼
┌─────────────┐
│  Response   │
│  Re-inflate │
└─────────────┘
```

---

### Sequence Diagrams

**Purpose**: Show interaction between components over time

**Elements**:
- **Actors**: Top boxes (components)
- **Lifelines**: Vertical dashed lines
- **Messages**: Horizontal arrows with labels
- **Activation**: Rectangles on lifelines

**Tools**: PlantUML, Mermaid.js

**Example**:
```
User      Pathos     Logos     Ethos    Consensus
 │           │          │         │          │
 ├──────────>│          │         │          │
 │ Process   │          │         │          │
 │           │          │         │          │
 │           ├──────────>│         │          │
 │           │ Retrieve  │         │          │
 │           │          │         │          │
 │           ├───────────>│        │          │
 │           │          │ Verify  │          │
 │           │          │         │          │
 │           ├───────────┼─────────>│          │
 │           │          │         │ Vote     │
 │           │          │         │          │
```

---

## Screenshots

### Capture Guidelines

**Preparation**:
1. **Clean terminal**: Clear previous commands, history
2. **Consistent theme**: Use same terminal theme for all screenshots
3. **Window size**: Standardize (e.g., 1200x800px)
4. **Font**: Use clear, readable monospace font (14-16px)
5. **Background**: Clean desktop background or solid color

**Terminal Setup**:
```bash
# Set terminal colors (example for GNOME Terminal)
# Background: #0A1628 (Deep Space Blue)
# Text: #FFFFFF (White)
# Font: Monospace 14px
```

**Capture Commands**:
```bash
# macOS
screencapture -T 2 -x screenshot.png

# Linux (gnome-screenshot)
gnome-screenshot -a -f screenshot.png

# Windows (PowerShell)
Take-ScreenCapture -Path screenshot.png
```

**Editing**:
- Crop to relevant content
- Add subtle drop shadow (optional)
- Add annotations (arrows, highlights) if needed
- Ensure text is legible
- Optimize file size

---

### Screenshot Checklist

Before using a screenshot:

- [ ] Terminal/app is in consistent theme
- [ ] Text is legible at full resolution
- [ ] Window is appropriately sized
- [ ] No sensitive information (API keys, tokens)
- [ ] File size is reasonable (under 1MB)
- [ ] Resolution is adequate (1200px wide minimum)
- [ ] Content is relevant and up-to-date
- [ ] Annotations are clear and necessary

---

## ASCII Art Enhancements

**Note**: Some diagrams in documentation are ASCII art. These can be enhanced to professional diagrams while maintaining the same structure.

**Example ASCII Art** (from README.md):

```
User Query
     │
     ▼
┌───────────────────────────────────┐
│         Privacy Proxy             │
└─────────────┬─────────────────────┘
              │
┌─────────────▼─────────────────────┐
│      Tripartite Council           │
│  ┌────────┐ ┌────────┐ ┌────────┐│
│  │ Pathos │ │  Logos │ │  Ethos ││
│  └───┬────┘ └───┬────┘ └───┬────┘│
└───────────────────────────────────┘
```

**Enhanced Diagram**: Convert to SVG using same structure, but with:
- Rounded rectangles with proper fills
- Color-coded agents (Pathos: cyan, Logos: orange, Ethos: green)
- Smooth connecting lines
- Proper typography

---

## Asset Inventory Template

Use this template to track created images:

```markdown
### Image Name

**File**: `filename.png`
**Size**: 1200x800px
**Format**: PNG
**Created**: 2026-01-07
**Creator**: [Name/Tool]
**Purpose**: [Brief description]
**Used In**: [README.md, ARCHITECTURE.md, etc.]
**Status**: [Ready/Needs Review/Outdated]

**Changelog**:
- 2026-01-07: Initial creation
- 2026-01-15: Updated colors to match brand guidelines
```

---

## Next Steps

1. **Create architecture diagrams** from existing ASCII art
2. **Capture CLI screenshots** with consistent theming
3. **Design feature showcase images** for marketing
4. **Optimize all images** for file size and quality
5. **Document each image** using inventory template
6. **Create animated diagrams** (optional, for advanced use cases)

---

## Resources

### Design Tools

- **Mermaid Live Editor**: https://mermaid.live/
- **Excalidraw**: https://excalidraw.com/
- **draw.io**: https://app.diagrams.net/
- **Figma**: https://www.figma.com/
- **PlantUML**: https://plantuml.com/

### Image Optimization

- **TinyPNG**: https://tinypng.com/
- **Squoosh**: https://squoosh.app/
- **ImageOptim**: https://imageoptim.com/

### Inspiration

- **Diagrams as Code**: https://diagrams.mingrammer.com/
- **Awesome Diagrams**: https://github.com/streetturtle/awesome-diagrams

### Related Documentation

- [Brand Guidelines](../../BRAND_GUIDELINES.md) - Visual identity
- [Social Preview Spec](../../SOCIAL_PREVIEW.md) - Social media images
- [Logo Specification](logo/logo.md) - Logo design

---

## Version History

**v1.0.0** (2026-01-07)
- Initial image inventory and requirements
- Defined diagram standards
- Created naming conventions
- Established creation guidelines

---

**SuperInstance AI** - *Your AI, Your Way, Your Privacy.*

*For the latest branding assets, check the GitHub repository.*
