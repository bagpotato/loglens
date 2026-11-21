# mobile

Tauri 2 shell (Rust) that packages `frontend/dist` as a native app (Android APK; desktop works too).
The app talks to a LogLens server the user enters on the login screen (`loglens.server` in localStorage).

- `src-tauri/` — standalone Cargo project (own `[workspace]`, not part of `backend/`). `run()` is the only item.
- `src-tauri/tauri.conf.json` — id `app.loglens.mobile`; builds the frontend first (`npm run build --prefix ../frontend`).
- `src-tauri/capabilities/default.json` — `core:default` only (no shell/fs/http plugins).
- `src-tauri/gen/android/` — generated Android project; cleartext HTTP allowed for LAN/emulator servers.
- Icons: `npm run icons` (from `frontend/public/pwa-512x512.png`).

Build: `npm install`, then `npm run android:build` (debug x86_64 APK for the emulator) or `npm run android:release`.
Needs Rust Android targets, Android SDK + NDK, JDK 17+. On Windows without Developer Mode the CLI cannot symlink
the `.so`: copy `src-tauri/target/x86_64-linux-android/debug/libloglens_mobile_lib.so` (llvm-strip it) into
`gen/android/app/src/main/jniLibs/x86_64/`, then `gradlew clean assembleX86_64Debug -x rustBuildX86_64Debug` in
`gen/android` (`clean` matters: incremental packaging keeps the old unstripped `.so` as dead space, 131 MB vs 21 MB).
Env vars: `ANDROID_HOME`, `NDK_HOME`, `JAVA_HOME`. Server side: `LOGLENS_CORS_ORIGINS` must allow `http://tauri.localhost` (default).
