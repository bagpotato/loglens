# features/auth

Login screen and session state for the single admin user (JWT in localStorage).

Public items (`index.ts`):

- `AuthProvider` (`{ children, onLogout? }`) — holds the session; restores it from the stored token (decoded JWT `sub`/`role`, expired tokens dropped); logs out on the `loglens:unauthorized` window event.
- `useAuth()` -> `{ user, isAuthenticated, login(username, password), logout() }`.
- `RequireAuth` — redirects to `/login` (keeps the original URL in `state.from`) when there is no session.
- `LoginPage` — mobile-first form; error state for bad credentials (401) and network failures. In the native
  app (Tauri) it also asks for the server URL (stored via `@/shared/platform`).
- `login(username, password)` — `POST /api/auth/login` -> `{ token, username, role }`.

Dependencies: `@/shared/api/client` (token storage, ApiError), `@/shared/ui`, react-router.
Env vars: none.
