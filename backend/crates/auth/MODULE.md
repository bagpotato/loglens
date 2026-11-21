# loglens-auth

Login, password hashing, JWT and the axum extractor that guards API routes.

- `hash_password` / `verify_password`: Argon2id default params, PHC strings.
- `JwtKeys::new(secret)`, `.issue(&User)` (HS256, 12 h), `.verify(token) -> Claims`
  (`Claims { sub, uid, tid, role, exp }`).
- `AuthUser`: extractor (`JwtKeys: FromRef<S>`). Token from `Authorization: Bearer <t>`
  or `Sec-WebSocket-Protocol: bearer, <t>`. Rejects with 401 `{"error":"unauthorized"}`.
- `ensure_admin(pool, user, password)`: creates the first admin when tenant 1 has no users.
- `login(pool, keys, user, password) -> LoginResponse { token, username, role }`.
  Unknown user and wrong password give the same error with similar timing.
- `AuthError`: `IntoResponse`, 401 for credentials/token, 500 otherwise.
- Depends on: loglens-storage, argon2, jsonwebtoken, axum, chrono, serde.
- Env vars (read by the api via loglens-config, passed in): `LOGLENS_ADMIN_USER`,
  `LOGLENS_ADMIN_PASSWORD`, `LOGLENS_JWT_SECRET`.
