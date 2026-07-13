# Frontend

pnpm workspace 안에서 직원용 `operator` 앱과 pywebview용 `launcher` 앱을 분리한다.

```powershell
pnpm install --frozen-lockfile
pnpm dev:operator
pnpm dev:launcher
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```
