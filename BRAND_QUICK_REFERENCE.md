# SuperInstance AI - Brand Quick Reference

> **Version**: 1.0.0
> **Last Updated**: 2026-01-07

---

## 🎨 Colors

### Primary
```
Deep Space Blue:  #0A1628  (Backgrounds, headers)
Cyber Cyan:       #00D9FF  (Accents, CTAs, highlights)
```

### Secondary
```
Neural Purple:    #8B5CF6  (Pathos agent)
Logic Orange:     #F59E0B  (Logos agent, warnings)
Verification Grn: #10B981  (Ethos agent, success)
```

### Neutrals
```
White:            #FFFFFF  (Light mode)
Light Gray:       #E5E7EB  (Borders)
Medium Gray:      #6B7280  (Secondary text)
Dark Gray:        #1F2937  (Dark mode surfaces)
```

### Semantic
```
Error Red:        #EF4444  (Errors)
Info Blue:        #3B82F6  (Information)
```

---

## ✏️ Typography

### Font Stack
**Sans-serif** (Primary):
```css
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
             "Helvetica Neue", Arial, sans-serif;
```

**Monospace** (Code):
```css
font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Mono",
             "Source Code Pro", monospace;
```

### Size Scale
```
H1: 48px / 3rem     (700 Bold)
H2: 36px / 2.25rem  (600 Semi-Bold)
H3: 24px / 1.5rem   (600 Semi-Bold)
H4: 20px / 1.25rem  (600 Semi-Bold)

Body: 16px / 1rem   (400 Regular)
Small: 14px / 0.875rem
Caption: 12px / 0.75rem

Code: 14px (inline), 13px (block)
```

---

## 🎯 Logo Usage

### Variations
1. **Primary** (horizontal) - Main use
2. **Icon only** - Applications, favicon
3. **Stacked** - Social media, cards
4. **Monochrome** - Single-color printing

### Clear Space
Minimum: Height of "S" in "SuperInstance"

### Minimum Size
- **Digital**: 120px wide (horizontal), 32x32px (icon)
- **Print**: 1.5 inches wide

### Backgrounds
✅ White, Deep Space Blue, transparent
❌ Busy patterns, photos (unless clear space)

---

## 📊 Logo Colors (Three Agents)

```
Node 1 (Top):      #00D9FF  (Cyan)    → Pathos (Intent)
Node 2 (Bot Left): #F59E0B  (Orange)  → Logos (Logic)
Node 3 (Bot Right):#10B981  (Green)   → Ethos (Truth)
```

---

## 🎭 Voice & Tone

### Characteristics
- Expert but approachable
- Precise but not pedantic
- Confident but not arrogant
- Human but professional

### Guidelines
- ✅ Use active voice: "Run this command"
- ✅ Be direct: "Install the package"
- ✅ Use second person: "Your data"
- ✅ Include why: "Redact to protect privacy"
- ❌ Don't be wordy: "In order to" → "To"
- ❌ No jargon without explanation
- ❌ No weasel words: "basically", "actually"

---

## 📐 Visual Hierarchy

### Priority Order
1. Critical actions (CTAs)
2. Key information (what/why)
3. Supporting details (how)
4. Supplementary info (examples, tips)
5. Metadata (dates, versions)

### Size Matters
Larger = More important
Top-left = Most important (LTR languages)
Brighter = More important
More space around = More important

---

## 🎪 Icon & Emoji Usage

### Strategic Emojis
| Concept | Emoji |
|---------|-------|
| Success | ✅ |
| Progress | 🔄 |
| Pending | ⏳ |
| Warning | ⚠️ |
| Info | 💡 |
| Bug | 🐛 |
| Feature | ✨ |
| Security | 🔒 |
| Performance | ⚡ |
| Docs | 📚 |
| Code | 💻 |

### Guidelines
- Use consistently (same emoji = same meaning)
- Don't overuse (not every element needs one)
- Test recognition (is it clear without labels?)
- Max 1-2 per paragraph

---

## 📏 Image Specifications

### Diagrams
- **Format**: SVG (preferred) or PNG
- **Size**: Min 1200px wide (2x for retina)
- **Colors**: Brand palette only
- **Font**: System sans-serif, min 14px
- **File Size**: Under 500KB

### Screenshots
- **Format**: PNG (lossless)
- **Size**: 1920x1080px (Full HD)
- **Theme**: Consistent dark mode
- **Optimization**: Under 1MB

### Social Preview
- **Size**: 1280x640px (2:1 ratio)
- **Format**: PNG
- **File Size**: Under 500KB
- **Location**: Repository root

---

## 🚫 Don'ts (Common Mistakes)

### Logo
❌ Stretch or distort
❌ Change colors (use approved variations)
❌ Rotate or skew
❌ Crowd with other elements
❌ Use low-res versions

### Colors
❌ Use colors outside palette
❌ Insufficient contrast (test with checker)
❌ Too many colors in one design

### Typography
❌ Skip heading levels (H1 → H3)
❌ Use ALL CAPS for emphasis
❌ Multiple fonts in one document

### Content
❌ Bury important information
❌ Overuse formatting (bold, italic, code)
❌ Include unnecessary details

---

## ✅ Dos (Best Practices)

### General
✅ Follow guidelines consistently
✅ Test accessibility (contrast, screen readers)
✅ Use high-quality images
✅ Maintain clear hierarchy
✅ Keep layouts clean

### Documentation
✅ Start with clear title/purpose
✅ Use code blocks for commands
✅ Include examples
✅ Add diagrams for architecture
✅ Link to related resources

### Social Media
✅ Use brand colors in graphics
✅ Include clear CTAs
✅ Use relevant hashtags (max 3-5)
✅ Tag contributors (with permission)
✅ Proofread before posting

---

## 📱 File Naming

### Format
`{category}-{description}.{variation}.{ext}`

### Examples
```
logo-horizontal.svg
logo-icon-32.png
architecture-overview.png
screenshot-cli-init.png
feature-privacy-showcase.png
```

### Categories
- `architecture` - System diagrams
- `feature` - Feature showcases
- `screenshot` - CLI/app screenshots
- `logo` - Logo variations
- `social` - Social media images

---

## 🔗 Accessibility

### Color Contrast
Minimum ratios:
- Normal text: 4.5:1
- Large text: 3:1
- UI components: 3:1

**Test at**: https://contrast-ratio.com/

### Alt Text
Always provide descriptive alt text:
```html
<img src="logo.svg" alt="SuperInstance AI logo">
```

### Inclusive Language
- Use person-first language
- Avoid gendered pronouns (use "they")
- Avoid idioms (non-native speakers)
- Provide context (don't assume prior knowledge)

---

## 📦 Quick Checklist

### Before Publishing
- [ ] Colors match palette
- [ ] Typography uses correct sizes
- [ ] Contrast meets WCAG AA
- [ ] Logo follows guidelines
- [ ] No typos
- [ ] File size optimized
- [ ] Alt text provided
- [ ] Tested on platform

### For Images
- [ ] Correct dimensions
- [ ] Under file size limit
- [ ] Text is legible
- [ ] No sensitive info
- [ ] Colors are accurate
- [ ] Properly named

---

## 🛠️ Tools

### Design
- **Figma**: https://www.figma.com/
- **Canva**: https://www.canva.com/
- **Excalidraw**: https://excalidraw.com/
- **Mermaid**: https://mermaid.live/

### Testing
- **Color Contrast**: https://contrast-ratio.com/
- **Twitter Card**: https://cards-dev.twitter.com/validator
- **Open Graph**: https://www.opengraph.xyz/

### Optimization
- **TinyPNG**: https://tinypng.com/
- **Squoosh**: https://squoosh.app/

---

## 📚 Full Documentation

For complete brand guidelines:
- [Brand Guidelines](BRAND_GUIDELINES.md) - Complete system
- [Logo Specification](assets/logo/logo.md) - Logo design
- [Social Preview](SOCIAL_PREVIEW.md) - Social media
- [Image Assets](assets/images/README.md) - Image inventory
- [Branding Summary](BRANDING_SUMMARY.md) - Full overview

---

## 💡 Remember

**Brand Essence**: Privacy-first, local-first AI with tripartite consensus

**Tagline**: "Your AI, Your Way, Your Privacy."

**Key Values**: Privacy, Transparency, Collaboration, Innovation

**Visual Style**: Clean, technical, modern, approachable

---

**SuperInstance AI** - *Your AI, Your Way, Your Privacy.*

*Quick Reference v1.0.0 | 2026-01-07*
