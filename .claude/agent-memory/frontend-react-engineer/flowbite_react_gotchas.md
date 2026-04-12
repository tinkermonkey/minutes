---
name: Flowbite React Component Gotchas
description: Version-specific API differences in flowbite-react that differ from documentation examples
type: project
---

## flowbite-react v0.12.x installed in this project

### Drawer sub-components are named exports, NOT dot-notation

Use:
```tsx
import { Drawer, DrawerHeader, DrawerItems } from 'flowbite-react';
<DrawerHeader title="Settings" />
<DrawerItems>...</DrawerItems>
```

NOT `<Drawer.Header>` or `<Drawer.Items>` — those properties do not exist on the component and will cause TS errors.

### No Skeleton component

`Skeleton` is not exported from `flowbite-react` in this version. Use Tailwind `animate-pulse` with plain `div` elements instead.

### Button has no isProcessing prop

The `Button` component does not have an `isProcessing` prop. To show a loading spinner inside a button, import `Spinner` separately and render it as a child:

```tsx
import { Button, Spinner } from 'flowbite-react';

<Button disabled={isLoading} onClick={handler}>
  {isLoading && <Spinner size="sm" className="mr-2" />}
  {isLoading ? 'Loading…' : 'Click me'}
</Button>
```

**Why:** These APIs differ from the documentation and will cause TypeScript errors if used incorrectly. Confirmed against the installed package (v0.12.17).

**How to apply:** Any time a new Flowbite component is used, verify the exact export name and prop shape matches v0.12.x before writing the code.
