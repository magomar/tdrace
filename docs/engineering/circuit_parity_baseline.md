---
type: Architecture Spec
title: "Circuit Parity Baseline (Spec 042, step 1)"
description: "Baseline verification confirming that official circuit JSON definitions match legacy Rust circuit generators for Spec 042."
status: active
category: engineering
tags: [circuits, tracks, parity, baseline, spec-042]
---

# Circuit Parity Baseline (Spec 042, step 1)

Date: 2026-09-27. Task: `tdrace-ylew.1`.

## Question

Before the Rust circuit generators are removed, does any official circuit in
Rust hold a change that is not in its `tracks/` JSON?

## Method

1. Export every `GameModule::tracks()` generator (all six modules) to a scratch
   folder, with the same export code that wrote `tracks/`:

   ```bash
   TDRACE_EXPORT_DIR=<dir> cargo test -p tdrace-app --test track_manager_tests \
       test_export_canonical_presets_to_git_repo -- --ignored
   ```

2. Compare each exported file with the file of the same path in `tracks/`,
   field by field, with a relative float tolerance of 1e-3:

   ```bash
   python3 scripts/circuit_parity.py <dir> tracks
   ```

Inputs: `tdrace` at the step 1 commit, `tdrace-tracks` at `badb564`.

Self-check of the compare script: a changed `default_laps`, a removed wall
segment (Monza) and a moved waypoint (Lonato) were each reported.

## Result

**96 of 96 circuits are identical.** No file exists on only one side.

So no Rust-side fix is missing from the JSON, and no JSON re-export is needed.
The GT kerb fix is in `tdrace-tracks` commit `badb564`.

## Not covered here

The export writes only the `Track` data. The `title`, `tag`, `description`
and `default_laps` of each `TrackDefinition` were not compared. Step 2
(`tdrace-ylew.2`) moves them into the JSON and checks them.
