# magnetar

Rust fullstack template: **Axum + Askama + SQLx + Postgres + Datastar + NATS**.
Hypermedia CQRS (commands return `204`, UI updates stream as Datastar SSE
patches), with background jobs (JetStream) and scheduled work as separate
processes.

## Prerequisites

- [mise](https://mise.jdx.dev) — manages the toolchain, tasks, and env
- Docker — for Postgres (later phases)

## Setup

```sh
git clone <repo> magnetar && cd magnetar
mise trust && mise install
mise run dev:server
```

Server runs at http://localhost:3000 (`/healthz` for liveness).

## Configuration

Env files are layered by mise (`mise.toml → [env]`):

- `.env` — committed defaults, safe for any machine. **Never put secrets here.**
- `.env.local` — gitignored; machine-specific overrides and all secrets.

mise's env overrides the shell, so `RUST_LOG=debug mise run ...` won't work —
set overrides in `.env.local` instead.

| Variable       | Default                                            | Notes                    |
| -------------- | -------------------------------------------------- | ------------------------ |
| `PORT`         | `3000`                                             | server listen port       |
| `RUST_LOG`     | `info,sqlx=warn`                                   | `tower_http=debug` for request logs |
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/app_db` | used from phase 3        |
| `NATS_URL`     | `nats://localhost:4222`                            | used from phase 4        |

## Tasks

```sh
mise run dev:server   # run server with hot reload (watchexec)
mise tasks            # list everything
```

Tip: run `bacon clippy` in a separate terminal for a live compiler/lint panel
while editing.

## Version control

jj colocated with git:

```sh
jj describe -m \"...\"   # name the current change
jj new                 # start the next one
jj git push --bookmark main
```

---

Built following the course in [`.plans/magnetar`](.plans/magnetar/README.md).
