---
type: Project Constitution
title: "Project Mission & Agent Constitution"
description: "Constitutional governance, coding laws, and architectural boundaries."
status: active
---

# Project Mission & Agent Constitution 📜

A template for the project-wide constitution, establishing the operational laws for developers and AI coding agents.

---

## 🎯 Primary Mission

Provide a one-paragraph description of the project's primary mission, the target audience, the core problems it resolves, and the expected standard of polish.

---

## 📜 Core Laws of Development

All developers and AI coding agents must unconditionally adhere to the following principles:

### 1. Hybrid Pragmatic Keel Methodology (Spec-Driven Development)
- **Major Features & Architectural Changes**: Must be plan-driven. A formal specification (or modification to an existing spec) must be drafted in `/specs/` and approved before code changes.
- **Minor Fixes & Local Adjustments**: Bug fixes, style tweaks, and test cases can be updated directly in code without requiring a formal spec update.
- **Strict Major Boundaries**: A formal specification MUST be created or modified if a change:
  1. Adds, renames, or removes a database field, collection, or schema.
  2. Modifies an API signature (query parameters, path parameters, or request/response payloads).
  3. Adds or removes an external dependency or library.
  4. Aligns with a milestone checklist item in `roadmap.md`.
- **File Naming & Casing Standard**: To establish a clear visual hierarchy and maintain cross-platform filesystem compatibility:
  - **UPPERCASE** is strictly used for high-priority constitutional and policy sheets (e.g., `MISSION.md`, `TECH_STACK.md`, `ROADMAP.md`, `BACKLOG.md`).
  - **lowercase_snake_case** is strictly used for active, numbered feature or technical specifications (e.g., `001_permissions.md`, `002_route_storage.md`).
- **Implementation Plans**: Always draft a detailed, approved implementation plan before making complex code changes.
- **Issue Tracking & Commits**: Engineering tasks are decomposed into discrete, tracked units rather than ephemeral task files. Commits implementing tasks are recommended to follow [Conventional Commits](https://www.conventionalcommits.org/) format with the associated task ID suffix (e.g., `(keel-a1b2)`).

### 2. Simplicity First (KISS/DRY)
- Write the minimum amount of code required to resolve the problem. No speculative abstractions or unrequested features.
- Avoid introducing redundant external packages or library dependencies when standard tools exist.

### 3. Surgical Changes
- Touch only what is required to satisfy the goal. Match surrounding code conventions, and do not make unrelated refactors or cleanups.
- Remove any imports, variables, or functions that are made unused by your modifications.

### 4. Goal-Driven Verification
- Enforce the Plan-Build-Verify loop: know your testing and validation criteria before writing any code.
- Ensure all quality checks (linting, tests) compile and pass with zero warnings prior to merge.

---

## 🤝 Codebase Architecture & Ownership

Detail the software layers and folder responsibilities:
- **Frontend Layer**: (e.g. Next.js, React, Astro) - Single source of truth for the user interface.
- **Backend Layer**: (e.g. FastAPI, Express, Django) - Single source of truth for database and business logic.
- **Specs Directory**: Location of active design, database, and feature contracts.
