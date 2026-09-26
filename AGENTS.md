# Agent Instructions

This project follows the **Keel Spec-Driven Development (SDD)** methodology. All AI agents must strictly align with the guidelines below to maintain codebase integrity and prevent architectural drift.

## 🎯 1. Alignment Philosophy

In a Keel-managed project, **Specifications are the absolute contracts of execution**.
- Read the project constitution (`specs/constitution/MISSION.md`, `specs/constitution/TECH_STACK.md`, and `specs/constitution/ROADMAP.md`) at the start of every session to align with the core architecture and guidelines.
- Check the project backlog (`BACKLOG.md`) for prioritized tasks.
- Operate strictly through plan-driven gates before proposing or committing code.

## 🚦 2. Execution Gate Rules

1. **Align on Tech Stack:** Never use or propose libraries, languages, or frameworks not explicitly approved in `specs/constitution/TECH_STACK.md`.
2. **Comprehensive Preflight Diagnostics & RDD Verification:** Run `keel doctor` to diagnose environment, issue tracker, OKF compliance, and RDD integrity; run `keel validate` to verify spec integrity and acceptance criteria before requesting approval on changes. Receipt-Driven Development (RDD) is active in advisory mode (`keel.json`). For completed specs, record durable verification receipts via `keel receipt <spec-number> --cmd "<test-command>"`.
3. **Surgical Changes:** Touch only what you must. Do not improve, format, or refactor unrelated adjacent code. Match the existing style exactly.
4. **Zero Residual Dead Code:** Prune all imports, variables, or functions rendered unused/obsolete by your changes.
5. **Atomic Conventional Commits:** Structure git commits atomically. Following [Conventional Commits](https://www.conventionalcommits.org/) format with the associated task ID suffix (e.g., `feat(auth): add session validator (keel-abc2)`) is recommended.

## 🔄 3. SDD Lifecycle Mapping

- **Specifications (`specs/*.md`)**: Formal blueprints defining requirements and Pseudo-Gherkin acceptance criteria.
- **Task Tracking**: Task state is managed natively (e.g., via the Beads issue tracker `br` or platform issue board). Do not rely on loose markdown task lists.
- **Atomic Commits**: Individual tasks map to discrete commits following the recommended [Conventional Commits](https://www.conventionalcommits.org/) format.
- **Diagnostics & Remediation**: Run `keel doctor` (and `keel doctor --fix`) to maintain environment, issue tracker health, and OKF alignment.
- **Walkthroughs & Verification Receipts**: Before completing a task, record a verification receipt via `keel receipt <spec-number> --cmd "<test-command>"` and check pre-closure integrity with `keel validate --completion <spec-number>`, providing concrete test evidence in the closure walkthrough.
