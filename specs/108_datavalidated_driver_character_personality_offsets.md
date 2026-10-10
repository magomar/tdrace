---
type: Architecture Spec
template: architecture
title: "Data-Validated Driver Character Personality Offsets"
description: "Measure every named driver character's personality offsets on the spec 083 pipeline, and shrink only the offsets that cause steering chatter, stalls or a broken tier ladder."
status: draft
created: 2026-10-11
generated: { by: agent/claude, at: 2026-10-11T00:00:00Z }
depends_on:
  - "083"
---

# Architecture Spec 108: Data-Validated Driver Character Personality Offsets 🧑‍✈️

This spec was split out of the first draft of spec 083 on 2026-10-11. It checks the named driver
characters against the calibrated baseline from spec 083.

---

## 🔍 Context

- `DriverCharacter::resolve_profile` (`crates/tdrace-app/src/ai/driver.rs`) adds 7
  `DriverPersonalityOffsets` deltas to the style x tier profile and clamps the result.
- `all_across_modules()` returns 72 characters: Classic 12, GT 12, NASCAR 12, Rally 12, Kart 12 and
  Extreme Off-Road 12. Autocross has 8 more characters in `module/autocross.rs`.
- The deltas were authored by hand. No test checks that a character drives cleanly or keeps its tier
  position.
- **Launch scope first:** Classic, Kart, Rally (Rallycross) and Autocross characters come first. GT,
  NASCAR and Extreme Off-Road characters are checked in a second pass, because those modules are
  post-launch DLC.

---

## 🗺️ Current vs. Proposed System Architecture

1. **Audit (blaze):** for each character, at T2 and T4, on its module's spec 083 training circuits with
   its preferred car, run 8 seeds x 3 laps. Record:
   - pace delta against the plain style profile
   - steering chatter: the sign changes of steering per second on straights
   - spins, offs and the longest no-progress time
2. **Flag** a character when any of these is true:
   - steering chatter is more than 2x the plain style profile
   - the pace delta is outside +-1.5 %
   - any no-progress time is above 10 s
   - the character breaks the tier order against the plain profiles
3. **Fix the flagged characters only.** Scale the character's offsets toward zero (one scale factor per
   character, found by bisection) until the flags clear. Keep the offset signs, so personality
   direction survives. Unflagged characters are not changed.
4. **Guard:** a fast test checks a fixed sample of characters for chatter and stalls.

No new parameters and no new dependencies. The change is only to the constant tables in `driver.rs`
and `module/autocross.rs`.

---

## 🗄️ Database & Storage Migration Plan
- No schema or save-data change. Only the constant offset tables change.

---

## 🔑 Security, Compliance, & IAM Roles
- No secrets, keys or new network services. blaze is reached over the existing Tailscale SSH setup.
- The tuning tools run offline. Nothing in the shipped game reads their output at runtime.

---

## 🛡️ Disaster Recovery, Monitoring, & Fallbacks
- Rollback: revert the fix commit. The old offsets come back unchanged.
- No bot-only rescue (same rule as spec 083).

---

## 🧪 Verification & Acceptance Criteria

### Automated tests
```bash
cargo test -p tdrace-app --test driver_character_tests
cargo test -p tdrace-app --release --test driver_offset_stability_tests
```

### Manual Acceptance Criteria (Pseudo-Gherkin)

#### Scenario: Every launch character is audited
- [ ] **Given** the Classic, Kart, Rally and Autocross characters
- [ ] **When** the audit runs on blaze
- [ ] **Then** `reports/driver_offsets_report.md` lists each character's pace delta, chatter, spins, offs and longest no-progress time at T2 and T4.

#### Scenario: Flagged characters are fixed by shrinking offsets
- [ ] **Given** a flagged character
- [ ] **When** its offsets are scaled toward zero
- [ ] **Then** all its flags clear
- [ ] **And** every non-zero offset keeps its sign.

#### Scenario: Unflagged characters do not change
- [ ] **Given** a character with no flags
- [ ] **When** the fix commit is applied
- [ ] **Then** its `DriverPersonalityOffsets` are byte-identical to before.

#### Scenario: Stability guard
- [ ] **Given** `driver_offset_stability_tests` with a fixed sample of 2 characters per launch module
- [ ] **When** it runs
- [ ] **Then** no sampled character exceeds the chatter limit or 10 s without progress
- [ ] **And** the test finishes in under 120 s on the Mac.

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[ ]` `crates/tdrace-app/src/ai/driver.rs` -> Adjusted offsets for flagged characters.
- `[ ]` `crates/tdrace-app/src/module/autocross.rs` -> Adjusted offsets for flagged Autocross characters.
- `[ ]` `crates/tdrace-app/tests/driver_offset_stability_tests.rs` -> Stability guard.
- `[ ]` `tools/bot_tuning/characters.py` -> Audit and bisection scripts.
- `[ ]` `reports/driver_offsets_report.md` -> Audit and fix results.

### Verification Assertions
- `driver_offset_stability_tests.rs` references `specs/108_datavalidated_driver_character_personality_offsets.md` in its header comment.
