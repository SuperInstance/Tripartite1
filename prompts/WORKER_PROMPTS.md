# Worker Prompts

> Copy-paste prompts for Claude Code sessions

---

## Session Starters

### First Session Ever
```
I'm starting work on SuperInstance AI, a tripartite agentic system with privacy-first local processing.

Please read these files in order:
1. CLAUDE.md - Your role as orchestrator
2. PROJECT_ROADMAP.md - Overall timeline
3. architecture/HIGH_LEVEL.md - What we're building
4. CLAUDE_CODE_BUILD_GUIDE.md - How to build it

After reading, confirm you understand the project and tell me what Phase/Milestone we should start with.
```

### Resume Existing Work
```
I'm resuming work on SuperInstance AI.

1. Read status/BUILD_STATUS.md for current state
2. Read status/CHANGELOG.md for recent changes
3. Check PROJECT_ROADMAP.md for next priority

What's the next task I should work on?
```

### Starting New Phase
```
We're starting Phase [X]: [Name].

1. Read phases/PHASE_[X]_[NAME].md completely
2. Read architecture/MEDIUM_LEVEL.md for context
3. Check what dependencies from previous phases are ready

Create a session plan for the first milestone of this phase.
```

---

## Implementation Prompts

### Create New Crate
```
Create a new Rust crate called synesis-[name] in the workspace.

Requirements:
- Part of the crates/ workspace
- Standard Cargo.toml with workspace dependencies
- Basic lib.rs with module structure from architecture/LOW_LEVEL.md
- Error types using thiserror
- Tracing instrumentation

Don't implement logic yet - just scaffolding.
```

### Implement Feature
```
Implement [feature name] as specified in [doc reference].

Requirements:
1. Follow the architecture in architecture/LOW_LEVEL.md
2. Use the patterns established in existing code
3. Add comprehensive error handling
4. Include tracing for debugging
5. Write unit tests

Show me the implementation plan before writing code.
```

### Add Tests
```
Add tests for [module/feature].

Requirements:
1. Unit tests for all public functions
2. Integration test for happy path
3. Edge case coverage:
   - Empty inputs
   - Invalid inputs
   - Error conditions
4. Use descriptive test names
5. Add test fixtures if needed

Target: 80%+ code coverage for this module.
```

### Fix Bug
```
There's a bug: [description]

Steps to reproduce:
1. [step 1]
2. [step 2]
3. [expected vs actual]

Debug by:
1. Adding tracing to relevant code paths
2. Writing a failing test that reproduces the bug
3. Finding root cause
4. Fixing with minimal changes
5. Verifying test passes
```

---

## Agent-Specific Prompts

### Pathos Agent Work
```
Working on the Pathos (Intent) agent.

Read agents/PATHOS_AGENT.md first.

Current task: [specific task]

Remember:
- Pathos extracts user intent
- Outputs A2A Manifest
- Uses phi-3-mini model
- Confidence calculation matters
```

### Logos Agent Work
```
Working on the Logos (Logic) agent.

Read agents/LOGOS_AGENT.md first.

Current task: [specific task]

Remember:
- Logos synthesizes solutions
- Uses RAG from Knowledge Vault
- Uses llama-3.2-8b model
- Tracks sources for citations
```

### Ethos Agent Work
```
Working on the Ethos (Truth) agent.

Read agents/ETHOS_AGENT.md first.

Current task: [specific task]

Remember:
- Ethos verifies and validates
- Can VETO dangerous operations
- Uses phi-3-mini model
- Provides feedback for revision rounds
```

### Infrastructure Work
```
Working on infrastructure.

Read agents/INFRASTRUCTURE_AGENT.md first.

Current task: [specific task]

Remember:
- Cloudflare Workers for API
- Durable Objects for state
- D1 for persistent data
- R2 for file storage
```

### Frontend Work
```
Working on frontend.

Read agents/FRONTEND_AGENT.md first.

Current task: [specific task]

Remember:
- Next.js 14 with App Router
- Tailwind + shadcn/ui
- Accessibility is required
- Performance budgets matter
```

---

## Review Prompts

### Code Review
```
Please review the following code:

[paste code or file path]

Check for:
1. Architecture alignment with docs
2. Error handling completeness
3. Security issues
4. Performance concerns
5. Test coverage
6. Documentation quality

Be specific about what to fix and why.
```

### Architecture Review
```
Review current implementation against architecture docs.

1. Read architecture/MEDIUM_LEVEL.md
2. Check if implementation matches
3. Identify deviations
4. Flag technical debt
5. Suggest improvements

Focus on: [specific area]
```

### Security Review
```
Perform a security review of [component].

Check for:
1. Input validation
2. Authentication/authorization
3. Data exposure risks
4. Injection vulnerabilities
5. Secrets management
6. Rate limiting
7. Error message leakage

Reference architecture/LOW_LEVEL.md security section.
```

---

## Documentation Prompts

### Update Status
```
Update project status after completing [task].

1. Update status/BUILD_STATUS.md:
   - Mark completed items
   - Update percentages
   - Note blockers

2. Update status/CHANGELOG.md:
   - Add entry with date
   - Describe what changed
   - Link relevant commits

3. Update PROJECT_ROADMAP.md if milestones changed.
```

### Write API Docs
```
Write API documentation for [module/endpoint].

Include:
1. Overview and purpose
2. Request/response schemas
3. Error codes and meanings
4. Usage examples
5. Rate limits if applicable
6. Authentication requirements

Use the format from docs/api-reference.md.
```

### Create Runbook
```
Create an operational runbook for [system/process].

Include:
1. Overview
2. Prerequisites
3. Normal operation steps
4. Troubleshooting common issues
5. Emergency procedures
6. Contact/escalation info

Target audience: On-call engineer at 3am.
```

---

## Debugging Prompts

### Trace Issue
```
Help me debug: [symptom]

Current observations:
- [observation 1]
- [observation 2]

What I've tried:
- [attempt 1]
- [attempt 2]

Add tracing to help identify the root cause.
```

### Performance Issue
```
Performance problem: [description]

Metrics:
- Expected: [X]ms
- Actual: [Y]ms

Help me:
1. Add timing instrumentation
2. Profile the slow path
3. Identify bottleneck
4. Suggest optimizations
```

### Integration Issue
```
Integration failing between [component A] and [component B].

Component A outputs: [format/type]
Component B expects: [format/type]

Debug the interface mismatch and fix.
```

---

## Deployment Prompts

### Prepare Release
```
Prepare release v[X.Y.Z].

1. Update version numbers
2. Generate changelog from commits
3. Run full test suite
4. Build release artifacts
5. Update documentation
6. Create release checklist

This is a [major/minor/patch] release because: [reason]
```

### Deploy to Staging
```
Deploy current main branch to staging.

1. Verify CI passes
2. Run deployment
3. Run smoke tests
4. Check monitoring
5. Document any issues

Staging URL: [url]
```

### Production Incident
```
INCIDENT: [brief description]

Severity: [P1/P2/P3]
Impact: [user impact]
Started: [time]

Help me:
1. Assess the situation
2. Implement immediate mitigation
3. Find root cause
4. Plan permanent fix
5. Draft incident report
```

---

## Context Refresh Prompts

### Long Session Reset
```
This is a long session. Let me refresh context:

1. We're building SuperInstance AI
2. Current phase: [X]
3. Current milestone: [Y]
4. We were working on: [specific task]
5. Last completed: [what we finished]

Here's where we left off: [context]

Continue from there.
```

### Knowledge Check
```
Before we continue, verify your understanding:

1. What is the tripartite council?
2. What does the privacy proxy do?
3. How does consensus work?
4. What's our current phase goal?

Briefly answer each to confirm context is loaded.
```

---

## Emergency Prompts

### Rollback
```
URGENT: Need to rollback [component] to previous version.

Current version: [X]
Target version: [Y]
Reason: [why]

Steps:
1. Verify rollback target is stable
2. Execute rollback
3. Verify functionality
4. Communicate status
```

### Data Recovery
```
Need to recover [data type] that was lost/corrupted.

Last known good state: [time/version]
What happened: [description]

Help me:
1. Assess what's recoverable
2. Plan recovery steps
3. Execute recovery
4. Verify integrity
5. Prevent recurrence
```
