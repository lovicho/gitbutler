# Dependencies

JavaScript dependencies are sourced from pnpm. Commands are surfaced via pnpm.

# Automation

In dev the app is accessible for agent automation on port 9222.

# Typechecking

Typechecking is the fastest way to validate that everything is okay. Always run this **exact** command to typecheck:

```console
$ pnpm -F @gitbutler/lite check
```

# Components

Memoization utilities such as `useMemo`, `useCallback`, and `React.memo` are redundant as we use React Compiler.

Component definitions should follow this pattern, optionally destructuring `p`:

```tsx
type Props = {
  ...
};

export const MyComponent: FC<Props> = (p) => {
  // [...]
};
```

# Concluding your work

Once the work is functionally complete, lint and format it with Oxlint, Oxfmt,
Prettier, and Knip. Oxfmt only formats TypeScript; CI runs Prettier over the
whole repo, including the CSS and Markdown that Oxfmt leaves untouched, so run
it too or those files fail CI:

```console
$ pnpm oxlint:fix && pnpm exec oxfmt apps/lite && pnpm exec prettier --write . && pnpm knip:prod && pnpm knip:non-prod
```
