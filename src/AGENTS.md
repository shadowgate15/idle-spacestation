# FRONTEND KNOWLEDGE BASE

## OVERVIEW

SvelteKit SPA frontend using Svelte 5 runes, Tailwind v4, and shadcn-svelte primitives.

## WHERE TO LOOK

| Task                        | Location                                                          | Notes                                                  |
| --------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------ |
| App shell and global styles | `routes/+layout.svelte`, `routes/+layout.ts`, `routes/layout.css` | Global CSS import, SPA mode, theme tokens              |
| Home page behavior          | `routes/+page.svelte`                                             | Tauri `invoke('greet')` demo                           |
| Shared primitives           | `lib/components/ui/`                                              | Button, card, input; each component has its own folder |
| Shared helpers              | `lib/utils.ts`                                                    | `cn()` and element/type helper utilities               |
| Route tests                 | `routes/*.spec.ts`, `routes/*.e2e.ts`                             | Vitest browser tests + Playwright E2E                  |
| Storybook examples          | `../.storybook/`, `stories/`                                      | Stories are examples, not app runtime                  |

## STRUCTURE

```text
src/
├── routes/              # App routes, layouts, colocated route tests
├── lib/
│   ├── components/ui/   # shadcn-svelte style primitives
│   ├── utils.ts         # shared class/type helpers
│   └── vitest-examples/ # example tests, low-authority reference
└── stories/             # Storybook example components and stories
```

## CONVENTIONS

- For `.svelte`, `.svelte.ts`, and `.svelte.js` implementation work, use `task(subagent_type="svelte-file-editor", load_skills=[], ...)`.
- For Svelte/SvelteKit questions, use Svelte MCP docs tools directly: call `svelte_list-sections` first, then `svelte_get-documentation` for only the relevant sections.
- Always run `svelte_svelte-autofixer` before returning Svelte code.
- Avoid loading `svelte-code-writer` and `svelte-core-bestpractices` until the runtime skill-registry issue is fixed.
- Treat shadcn-svelte docs as the canonical source for component APIs, composition, and theming patterns; use local `components.json` for this repo's aliases and style settings.
- Prefer existing aliases from `components.json`: `components`, `ui`, `utils`, `hooks`, `lib`.
- Keep UI primitives in their component folder with an `index.ts` barrel and `.svelte` implementation.
- Match current naming: kebab-case filenames, PascalCase component exports, camelCase helpers.
- For class composition, use `cn()` from `$lib/utils.ts`; for variants, follow the existing `tailwind-variants` pattern.
- Colocate tests with the route/component they cover.

## ANTI-PATTERNS

- Do not copy template Storybook components from `stories/` into production UI without adapting them to repo conventions.
- Do not treat `lib/vitest-examples/` as app architecture; it is example coverage only.
- Do not add SSR-dependent code unless you also revisit the SPA/Tauri setup; frontend is intentionally `ssr = false`.
- Do not bypass shadcn-svelte composition when a matching primitive already exists.
- Do not write raw class concatenation when `cn()` already covers the case.

## UNIQUE STYLES

- `routes/layout.css` owns Tailwind imports, shadcn theme tokens, and app-wide color variables.
- Browser component tests use `vitest-browser-svelte` plus `@tauri-apps/api/mocks` for frontend/Tauri boundary tests.
- Playwright E2E runs against a generated build + preview web server on port 4173; see `playwright.config.ts` for the exact command.
- Current frontend is still close to template scale; prefer small, local changes over new abstractions.

## NOTES

- `routes/demo/` is demo-only unless the task explicitly says otherwise.
- `lib/hooks` alias exists in config but there are no real hooks yet; verify before building around it.
