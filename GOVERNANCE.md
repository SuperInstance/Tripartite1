# SuperInstance AI - Governance and Decision-Making

## Table of Contents

- [Overview](#overview)
- [Project Leadership](#project-leadership)
- [Roles and Responsibilities](#roles-and-responsibilities)
- [Decision-Making Process](#decision-making-process)
- [Release Management](#release-management)
- [Roadmap Prioritization](#roadmap-prioritization)
- [Conflict Resolution](#conflict-resolution)
- [Financial Management](#financial-management)
- [Community Governance](#community-governance)
- [Amendments to This Document](#amendments-to-this-document)

---

## Overview

SuperInstance AI is an open-source project building a privacy-first, tripartite
agentic AI system. This document outlines how the project is governed, how
decisions are made, and how the community can participate.

### Guiding Principles

1. **Open Source First:** Code and decisions are transparent and public
2. **Community-Driven:** Users and contributors shape the project direction
3. **Technical Excellence:** We prioritize quality, security, and performance
4. **Privacy by Design:** User privacy is never an afterthought
5. **Consensus-Oriented:** Decisions are made through discussion and agreement
6. **Iterative Development:** We ship early, ship often, and improve continuously

---

## Project Leadership

### Project Lead

**Geoffrey Huntley** is the project lead and BDFL (Benevolent Dictator For Life)
with the following responsibilities:

- Overall project vision and direction
- Final decision-maker for unresolved conflicts
- Represents the project to the public
- Ensures project adheres to its principles
- Coordinates with maintainers and contributors

### Maintainers

Maintainers are contributors who have demonstrated:

- Consistent, high-quality contributions
- Deep understanding of the codebase
- Active participation in reviews and discussions
- Alignment with project principles

**Current Maintainers:**
- Geoffrey Huntley (Project Lead)
- [Additional maintainers will be added as the project grows]

**Becoming a Maintainer:**
Maintainers are invited by existing maintainers based on sustained contribution.
There is no formal application process. Criteria include:

- At least 20 merged pull requests
- Active participation in code reviews (at least 50 reviews)
- Contributed to at least 3 different areas of the codebase
- Demonstrated good judgment in technical discussions
- Endorsed by at least 2 existing maintainers

**Maintainer Responsibilities:**
- Review pull requests within a reasonable time (target: 1 week)
- Participate in technical discussions
- Mentor new contributors
- Uphold code quality and security standards
- Follow the Code of Conduct
- Make decisions in their area of expertise

**Stepping Down:**
Maintainers may step down at any time by notifying the project lead. If a
maintainer is inactive for 6 months without notice, they may be moved to
"emeritus" status (honorary, no active duties).

---

## Roles and Responsibilities

### Contributors

Contributors are anyone who has contributed to the project (code, docs, tests,
reviews, etc.). All contributors are expected to:

- Follow the Code of Conduct
- Follow contribution guidelines (see CONTRIBUTING.md)
- Be respectful and collaborative
- Test their changes before submitting
- Respond to feedback on their contributions

**Rights:**
- Vote in RFCs (Request for Comments)
- Participate in discussions
- Submit pull requests
- Propose new features
- Report bugs and issues

### Users

Users are people who use SuperInstance AI. We value all users, whether they're
developers, researchers, or end users. Users are encouraged to:

- Report bugs and issues
- Request features
- Participate in discussions
- Ask questions
- Share their use cases

**Rights:**
- All contributor rights (if they contribute)
- File issues and participate in discussions
- Request help and support

### Security Team

The security team is responsible for:

- Handling vulnerability reports (see SECURITY.md)
- Security reviews of pull requests
- Dependency security audits
- Security testing and penetration testing
- Security advisories and announcements

**Composition:** Subset of maintainers with security expertise

### Release Managers

Release managers are responsible for:

- Preparing releases
- Writing release notes
- Coordinating release schedules
- Managing version numbers
- Handling hotfixes

**Composition:** Maintainers with release experience

---

## Decision-Making Process

### Consensus-First Approach

Most decisions are made through discussion and consensus:

1. **Proposal:** Anyone can propose a change (RFC, issue, PR)
2. **Discussion:** Community discusses pros/cons
3. **Revision:** Proposal is refined based on feedback
4. **Consensus:** If no objections, move forward
5. **Escalation:** If deadlock, project lead decides

### Types of Decisions

#### Minor Decisions (Day-to-Day)

**Examples:**
- Bug fixes
- Documentation improvements
- Refactoring
- Test improvements
- Minor features (non-breaking)

**Process:**
- Any contributor can submit a PR
- One maintainer approval required
- No formal discussion needed (unless controversial)

#### Major Decisions (Technical)

**Examples:**
- New features (breaking changes)
- Architecture changes
- Dependency changes
- Performance improvements
- API changes

**Process:**
1. Open an RFC (Request for Comments) issue
2. Discuss for at least 7 days
3. Address concerns and revise
4. Call for final comments (3 days)
5. If consensus, proceed; if deadlock, project lead decides

**RFC Template:**
```markdown
# RFC: [Title]

## Summary
Brief description of the proposal.

## Motivation
Why are we proposing this? What problem does it solve?

## Detailed Design
Technical details, implementation approach, etc.

## Drawbacks
Potential downsides, risks, or concerns.

## Alternatives
What other approaches did we consider?

## Unresolved Questions
What do we still need to figure out?
```

#### Strategic Decisions (Project Direction)

**Examples:**
- New project phases
- Major partnerships
- Funding models
- License changes
- Governance changes

**Process:**
1. Proposal by project lead or maintainer
2. Community discussion (14 days minimum)
3. Formal vote by maintainers
4. Requires 2/3 supermajority of maintainers
5. Project lead has veto power (rarely used)

### Voting

**When We Vote:**
- Resolving deadlocks in RFCs
- Strategic decisions
- Adding/removing maintainers
- Major policy changes

**Who Can Vote:**
- Maintainers vote on strategic decisions
- All contributors can vote on RFCs (advisory, maintainers decide)

**Voting Process:**
- Voting period: 7 days
- Public votes (comment on RFC)
- Simple majority for most decisions
- 2/3 supermajority for major changes
- Project lead breaks ties

---

## Release Management

### Version Numbering

We follow [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH

- MAJOR: Incompatible API changes
- MINOR: New functionality (backwards-compatible)
- PATCH: Bug fixes (backwards-compatible)
```

**Examples:**
- `0.2.0` → `0.2.1`: Bug fix
- `0.2.0` → `0.3.0`: New feature (backwards-compatible)
- `0.2.0` → `1.0.0`: First stable release
- `1.0.0` → `2.0.0`: Breaking changes

### Release Schedule

**Current Phase (0.x):**
- releases every 2-4 weeks
- No strict schedule (ship when ready)
- Focus on completing Phase 2: Cloud Mesh

**After v1.0:**
- Major releases: Every 6-12 months
- Minor releases: Every 1-3 months
- Patch releases: As needed (bug fixes, security)

### Release Process

**Pre-Release:**
1. Feature freeze (no new features)
2. Testing week (focused testing)
3. Release candidate (RC) if major version
4. Bug fixes only (during testing)

**Release:**
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. GitHub release (with notes)
5. Publish to crates.io
6. Announce on social media

**Post-Release:**
1. Monitor for issues
2. Hotfixes if critical bugs
3. Post-mortem if major issues

### Supported Versions

- **Current version:** Full support (features, bug fixes, security)
- **Previous version:** Security fixes only
- **Older versions:** Unsupported

See SECURITY.md for details.

---

## Roadmap Prioritization

### Phase-Based Development

SuperInstance follows a phased development approach:

- **Phase 1: Local Kernel** (Complete ✅)
  - Tripartite council
  - Privacy proxy
  - Knowledge vault
  - CLI interface

- **Phase 2: Cloud Mesh** (In Progress 🔄)
  - QUIC tunnel
  - Cloudflare Workers
  - Billing and metering
  - LoRA upload

- **Phase 3: Marketplace** (Planned)
  - Model marketplace
  - Knowledge credits
  - Collaborator features

- **Phase 4: Utility** (Future)
  - Advanced features
  - Performance optimization
  - Ecosystem expansion

See PROJECT_ROADMAP.md for details.

### Prioritization Criteria

When deciding what to work on next, we consider:

1. **User Impact:** How many users benefit? How significant is the benefit?
2. **Technical Dependencies:** What does this unblock? What depends on this?
3. **Effort:** How much work is required? (we prefer quick wins)
4. **Strategic Value:** Does this advance our vision and goals?
5. **Community Interest:** Is there demand for this feature?
6. **Security:** Does this improve security or fix a vulnerability?

**Priority Formula (Informal):**
```
Priority = (User Impact × Strategic Value) / Effort
```

High priority = High impact, high value, low effort
Low priority = Low impact, low value, high effort

### Roadmap Updates

The roadmap is updated:
- After each phase completion
- When major technical changes are needed
- Based on community feedback
- At least quarterly (even if no changes)

**How to Influence the Roadmap:**
- Open an RFC proposing a new phase or feature
- Participate in roadmap discussions
- Vote on RFC proposals
- Share your use cases and priorities

---

## Conflict Resolution

### Types of Conflicts

#### Technical Disagreements

**Example:** Which library to use, how to implement a feature

**Resolution:**
1. Discussion in RFC or issue
2. Present evidence and benchmarks
3. Seek compromise or hybrid approach
4. If deadlock, project lead decides

#### Code Review Disagreements

**Example:** Reviewer requests changes, contributor disagrees

**Resolution:**
1. Discuss in PR comments
2. Seek third-party opinion (another maintainer)
3. Contributor must either make changes or withdraw PR
4. Reviewer has final say (but should be reasonable)

#### Conduct Issues

**Example:** Code of Conduct violation

**Resolution:**
See CODE_OF_CONDUCT.md for enforcement process.

#### Governance Disputes

**Example:** Disagreement about project direction

**Resolution:**
1. Discussion in maintainer forum
2. Vote by maintainers (2/3 supermajority)
3. Project lead has final decision

### Escalation Path

1. **Peer Discussion:** Try to resolve directly
2. **Third Party:** Ask another maintainer to mediate
3. **Project Lead:** Final decision-maker
4. **Community Vote:** For major changes (rare)

---

## Financial Management

### Funding Sources

SuperInstance AI is currently funded by:

- **Contributor Time:** Volunteer effort by contributors
- **Future:** We're exploring sustainable funding models

### Planned Funding Models

We're considering several funding models:

1. **Grants:** Apply for research and open-source grants
2. **Sponsorships:** Corporate sponsorships (with independence guarantees)
3. **Service Fees:** Managed hosting, support contracts
4. **Knowledge Credits:** Contributor rewards system

**Principles:**
- No venture capital (unless strategic)
- Maintain project independence
- Transparent about funding sources
- Reinvest in the project

### Financial Transparency

If we accept funding, we commit to:

- Publish financial reports quarterly
- Disclose all funding sources
- Explain how funds are used
- Avoid conflicts of interest
- Community input on major financial decisions

---

## Community Governance

### Communication Channels

**Official Channels:**
- **GitHub:** Issues, PRs, Discussions
- **Discord:** (Coming soon) Real-time chat
- **Email:** dev@superinstance.ai

**Decision-Making Forums:**
- **RFCs:** Technical proposals
- **Issues:** Bugs and feature requests
- **Discussions:** General questions and ideas
- **Maintainer Meetings:** Private discussions (logged)

### Community Standards

All community spaces are governed by:

- **Code of Conduct:** See CODE_OF_CONDUCT.md
- **Contributing Guidelines:** See CONTRIBUTING.md
- **Security Policy:** See SECURITY.md

### Community Growth

**Recruitment:**
- Welcome newcomers warmly
- Provide good first issues
- Mentor contributors
- Recognize contributions

**Retention:**
- Respond to questions promptly
- Review PRs quickly (target: 1 week)
- Recognize and thank contributors
- Provide growth opportunities

**Diversity and Inclusion:**
- Actively welcome underrepresented groups
- Use inclusive language
- Be accommodating of different time zones
- Provide multiple ways to contribute

---

## Amendments to This Document

### Changing Governance

This governance document can be amended by:

1. **Proposal:** RFC proposing changes
2. **Discussion:** At least 14 days discussion
3. **Vote:** 2/3 supermajority of maintainers
4. **Approval:** Project lead approval

### Triggering Amendments

This document should be reviewed and updated:
- Annually (at minimum)
- When major changes occur
- When community requests it
- If it no longer reflects reality

---

## Contact and Feedback

### Governance Questions

For questions about governance:

- **GitHub Discussion:** https://github.com/SuperInstance/Tripartite1/discussions
- **Email:** governance@superinstance.ai
- **Project Lead:** Via GitHub @geoffreyhuntley

### Feedback

We welcome feedback on this governance document:

- Open an issue or discussion
- Suggest improvements
- Ask for clarification
- Propose amendments

---

## Acknowledgments

This governance document is inspired by:

- **Rust Governance:** https://www.rust-lang.org/governance
- **TensorFlow Governance:** https://www.tensorflow.org/governance
- **Mozilla Governance:** https://www.mozilla.org/about/governance/
- ** Contributor Covenant:** Code of Conduct

---

*Last Updated: 2026-01-07*
*Version: 1.0*
*Next Review: 2026-01-07 (annual)*

This document is a living document and will evolve as the project grows.
