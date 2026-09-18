---
type: Tech Stack
title: "Tech Stack & Styling Guidelines"
description: "Approved technical stack, framework runtimes, styling guidelines, and commands."
status: active
---

# Tech Stack & Styling Guidelines 🛠️

A template detailing the approved technologies, library rules, styling guidelines, and strict command environments for this project.

---

## 💻 Tech Stack Specification

List the frameworks, database backends, payment models, and specific target runtimes:

### Frontend
| Technology | Version | Purpose / Description |
| :--- | :--- | :--- |
| **Framework** | e.g. `15.x` | Modern web framework |

### Backend
| Technology | Version | Purpose / Description |
| :--- | :--- | :--- |
| **Language** | e.g. `3.13+` | Main backend runtime |

---

## 🎨 Design & Styling Principles (Aesthetic Excellence)

Define the styling standards to ensure visual excellence and UI consistency:
- **Design Tokens**: Specify primary, secondary, surface, and alert colors.
- **Dark Mode**: Detail how dark mode backgrounds are supported (e.g. Slate, HSL Hues).
- **Typography**: Declare Heading and Body font preferences (e.g. Outfit, Inter).
- **Micro-Animations**: Rules for hovers, scales, and transition states.

---

## 🚀 Execution & Command Enforcements

Codify the specific command-line utilities and runtime managers developers/agents must use:

### Frontend Execution (e.g. bun, npm, pnpm)
- Command to install: `bun install`
- Command to run development server: `bun run dev`
- Command to run tests: `bun test`

### Backend Execution (e.g. uv, poetry, pipenv)
- Command to install: `uv sync`
- Command to run server: `uv run uvicorn ...`
- Command to run tests: `uv run pytest`

---

## 🛡️ Coding & Schema Constraints

- **Type Safety**: Guidelines for strongly typed API contracts and zero use of open/any parameters.
- **Validation Schemas**: Enforced validators (e.g. Pydantic, Zod) for parsing API requests and responses.
- **Dependency Auditing**: Prohibitions against downloading redundant third-party libraries.
