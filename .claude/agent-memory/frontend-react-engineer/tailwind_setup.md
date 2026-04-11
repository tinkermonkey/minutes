---
name: Tailwind + Flowbite Setup
description: How Tailwind v4 and flowbite-react are configured in this project
type: project
---

## Tailwind Version

Using Tailwind v4 via `@tailwindcss/vite` plugin (not the v3 PostCSS plugin). There is no `tailwind.config.js` — v4 uses CSS-first configuration.

## CSS Entry Point

`src/index.css` contains only:
```css
@import "tailwindcss";
```

This single import is sufficient for v4 + the Vite plugin. No content path array needed.

## Vite Config

`@tailwindcss/vite` is the first plugin in the array (before `@vitejs/plugin-react`):
```ts
plugins: [tailwindcss(), react()]
```

## Flowbite React

`flowbite-react` is installed. For Tailwind v4 compatibility, no additional `flowbite` plugin registration in CSS was needed — the import above covers it for Stage 0.

**Why:** If Flowbite components stop picking up styles, check flowbite-react docs for v4 — may need `@plugin "flowbite-react/tailwind"` in index.css.

**How to apply:** Import Flowbite React components directly from `flowbite-react`. Tailwind utility classes work alongside them without any extra config.
