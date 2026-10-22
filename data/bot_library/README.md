# Bot Behaviour Library 📚🏎️🤖

This directory houses the persistent, versioned **Bot Behaviour Library** for TDRace (Spec 137).

## Architecture

- `behaviours/*.toml`: Individual declarative behavior specification files with full creation provenance, tactical driving personas, controller parameters, and performance summaries.
- `benchmark_matrix.db`: SQLite database storing detailed multi-scenario evaluation matrix runs (time trials, pack race simulations across varied modalities/cars/tires, and real-race human player telemetry).

## Lifecycle Statuses

- `active`: Validated behavior available for general career and race grid selection.
- `candidate`: Experimental behavior undergoing simulation matrix evaluation or tuning.
- `baseline`: Golden standard reference behavior (immutable).
- `archived`: Preserved historical behavior retained for scenario or backwards compatibility.
- `deprecated`: Obsolete behavior kept for regression testing.
