# Review Prompts

> Prompts for quality assurance and verification sessions

---

## Daily Review

### Morning Standup Prompt
```
Good morning. Let's do a quick standup.

1. Read status/BUILD_STATUS.md
2. Read status/CHANGELOG.md (last 3 entries)
3. Check PROJECT_ROADMAP.md current milestone

Answer:
- What was completed yesterday?
- What's planned for today?
- Any blockers?

Keep it brief - 3 sentences max per question.
```

### End of Day Wrap-up
```
End of day wrap-up.

Review what we accomplished today:
1. List completed tasks
2. Note any partial progress
3. Document blockers encountered
4. Plan tomorrow's priority

Update:
- status/BUILD_STATUS.md
- status/CHANGELOG.md

Commit message for today: [generate appropriate message]
```

---

## Code Reviews

### PR Review Checklist
```
Review this PR/change for merge readiness.

Checklist:
□ Follows architecture patterns from docs
□ Includes appropriate tests
□ Error handling is complete
□ No sensitive data exposed
□ Performance is acceptable
□ Documentation updated
□ Breaking changes noted

For each issue found, rate:
- Severity: blocker / major / minor / nit
- Location: file:line
- Suggestion: how to fix

Approve, request changes, or request discussion.
```

### Security-Focused Review
```
Security review for [component/PR].

Check for:

Authentication & Authorization:
□ Auth checks on all protected routes
□ Token validation correct
□ Permissions properly enforced

Data Handling:
□ Input validation complete
□ Output encoding correct
□ Sensitive data encrypted
□ PII properly handled

Common Vulnerabilities:
□ No SQL injection
□ No XSS vectors
□ No path traversal
□ No command injection
□ Rate limiting in place

Secrets:
□ No hardcoded secrets
□ Secrets from env only
□ Rotation supported

Findings format:
[SEVERITY] [CWE-XXX] Description
Location: file:line
Remediation: how to fix
```

### Performance Review
```
Performance review for [component].

Analyze:
1. Time complexity of algorithms
2. Space complexity
3. I/O patterns (disk, network)
4. Caching opportunities
5. Async vs sync choices

Measure (if possible):
- Cold start time
- Warm execution time
- Memory usage
- CPU utilization

Recommendations:
- Must fix (blocking release)
- Should fix (tech debt)
- Nice to have (optimization)
```

---

## Architecture Reviews

### Design Review
```
Review this design/architecture decision:

[Paste design doc or describe decision]

Evaluate against:
1. Does it fit the tripartite model?
2. Does it preserve privacy guarantees?
3. Is it consistent with existing patterns?
4. Does it scale appropriately?
5. Is it maintainable?

Questions to answer:
- What are the tradeoffs?
- What alternatives were considered?
- What are the risks?
- What's the migration path if it fails?

Recommendation: approve / revise / reject
Rationale: [explain]
```

### Integration Review
```
Review integration between [Component A] and [Component B].

Check:
1. Interface contract is clear
2. Data formats match
3. Error handling spans boundary
4. Timeouts/retries configured
5. Monitoring covers integration

Integration diagram:
[Component A] --[protocol]--> [Component B]

Data flow:
- Request: [format, validation]
- Response: [format, errors]
- Async: [if applicable]

Risks identified:
1. [risk] - mitigation: [how]
```

### API Review
```
Review API design for [endpoint/module].

Principles to check:
□ RESTful conventions followed
□ Consistent naming
□ Proper HTTP methods/status codes
□ Versioning strategy clear
□ Pagination implemented
□ Rate limiting specified
□ Authentication documented

Request/Response analysis:
- Are required fields clearly marked?
- Are optional fields sensible defaults?
- Are error responses helpful?
- Is the response shape consistent?

Breaking change assessment:
- What changes would break clients?
- Is there a deprecation path?
```

---

## Documentation Reviews

### Doc Accuracy Review
```
Review documentation accuracy for [doc path].

Check against current implementation:
1. Code examples run correctly
2. API signatures match
3. Configuration options current
4. Screenshots/diagrams accurate

For each issue:
- Location: [section/line]
- Issue: [what's wrong]
- Correction: [what it should say]

Also check:
- Spelling/grammar
- Formatting consistency
- Link validity
- Version references
```

### Onboarding Review
```
Review onboarding documentation from new user perspective.

Pretend you know nothing about the project.

Follow the getting started guide:
1. Are prerequisites clear?
2. Can you complete each step?
3. Do examples work?
4. Are error messages helpful?

Pain points found:
1. [issue] at step [X]
2. [unclear instruction]

Time to first success: [estimate]
Recommended improvements: [list]
```

---

## Testing Reviews

### Test Coverage Review
```
Review test coverage for [module].

Current coverage: [X]%
Target coverage: 80%

Gaps identified:
1. [untested function] - risk: [impact]
2. [missing edge case] - risk: [impact]

Test quality assessment:
- Are tests actually testing behavior?
- Are assertions meaningful?
- Is test data representative?
- Are tests maintainable?

Recommended additions:
1. Test for [scenario]
2. Test for [edge case]
```

### Integration Test Review
```
Review integration tests for [workflow].

Coverage:
□ Happy path
□ Error paths
□ Edge cases
□ Concurrent scenarios
□ Timeout handling

Test environment:
- Is it representative of production?
- Are mocks appropriate?
- Is cleanup reliable?

Flakiness assessment:
- Any timing-dependent tests?
- Any order-dependent tests?
- External dependencies properly handled?

Recommendations:
1. [add/modify test]
```

---

## Release Reviews

### Pre-Release Checklist
```
Pre-release review for v[X.Y.Z].

Code readiness:
□ All planned features complete
□ All tests passing
□ No known critical bugs
□ Performance acceptable

Documentation:
□ Changelog updated
□ API docs current
□ Migration guide (if needed)
□ Release notes drafted

Operations:
□ Deployment runbook ready
□ Rollback plan tested
□ Monitoring in place
□ Alerts configured

Sign-off required from:
□ Development
□ QA
□ Security
□ Operations

Release decision: GO / NO-GO
Blockers if NO-GO: [list]
```

### Post-Release Review
```
Post-release review for v[X.Y.Z].

Metrics (first 24 hours):
- Error rate: [X]%
- Latency P50/P99: [X]ms / [Y]ms
- User complaints: [count]
- Rollback needed: yes/no

What went well:
1. [positive]
2. [positive]

What could improve:
1. [issue] - action: [fix]
2. [issue] - action: [fix]

Lessons learned:
1. [lesson]

Update runbooks: yes/no
Follow-up tasks: [list]
```

---

## Periodic Reviews

### Weekly Architecture Review
```
Weekly architecture review.

This week's changes:
1. [List significant changes]

Architecture coherence:
- Following tripartite pattern? [assessment]
- Privacy maintained everywhere? [assessment]
- Technical debt accumulating? [assessment]

Metrics trend:
- Code coverage: [trend]
- Build time: [trend]
- Test flakiness: [trend]

Upcoming concerns:
1. [potential issue]

Action items:
1. [task] - owner: [who] - due: [when]
```

### Monthly Health Check
```
Monthly project health check.

Progress:
- Phase [X] progress: [Y]%
- On track for deadline? [yes/no]
- Blockers: [list]

Quality metrics:
- Bug count trend: [up/down/stable]
- Tech debt items: [count]
- Documentation coverage: [%]

Team health:
- Velocity trend: [up/down/stable]
- Biggest pain points: [list]

Recommendations:
1. [strategic recommendation]
2. [process improvement]

Escalations needed: [list or none]
```

---

## Emergency Reviews

### Incident Review
```
Incident review for [incident ID].

Timeline:
- [time]: [event]
- [time]: [event]
- [time]: [resolved]

Impact:
- Duration: [X] minutes
- Users affected: [count]
- Data loss: [yes/no]

Root cause:
[Describe actual root cause]

5 Whys:
1. Why? [answer]
2. Why? [answer]
3. Why? [answer]
4. Why? [answer]
5. Why? [root]

Immediate fixes applied:
1. [fix]

Long-term fixes needed:
1. [fix] - priority: [P1/P2/P3] - owner: [who]

Prevention measures:
1. [measure]

Monitoring gaps:
1. [gap] - fix: [what to add]
```

### Security Incident Review
```
Security incident review for [incident ID].

CONFIDENTIAL - limit distribution

Classification: [severity]

Timeline:
- Detection: [time/method]
- Containment: [time]
- Eradication: [time]
- Recovery: [time]

Attack vector:
[Describe how attacker gained access]

Data exposure:
- PII exposed: [yes/no]
- Credentials exposed: [yes/no]
- Scope: [description]

Immediate response:
1. [action taken]

Remediation:
1. [fix required] - status: [done/in progress]

Notification requirements:
- Users: [yes/no]
- Regulators: [yes/no]
- Partners: [yes/no]

Lessons learned:
1. [lesson]
```
