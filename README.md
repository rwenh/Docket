# task-manager (Rust port)

Rust port of a FastAPI task-management API — JWT auth, Postgres-backed CRUD,
same routes and JSON shapes as the Python original.

## Stack

| Python                            | Rust                                                |
|------------------------------------|------------------------------------------------------|
| FastAPI                            | Axum 0.7                                              |
| SQLAlchemy (sync `Session`)         | Diesel 2.1 (sync), wrapped in `deadpool-diesel` so it can be called from async handlers via `.interact()` on a blocking worker thread |
| Pydantic                           | `serde` + `validator`                                 |
| python-jose                        | `jsonwebtoken`                                        |
| passlib (bcrypt)                   | `bcrypt` crate                                        |
| pydantic-settings (env vars)        | hand-rolled `Settings` struct reading `std::env`       |
| Alembic / implicit table creation   | DDL run directly via `sql_query` at startup — see note below |

## Project layout

Mirrors the Python package structure 1:1:

```
src/
  core/     config.rs, security.rs, deps.rs   (≈ app/core)
  db/       schema.rs, session.rs             (≈ app/db)
  models/   user.rs, task.rs                  (≈ app/models)
  schemas/  user.rs, task.rs                  (≈ app/schemas)
  routers/  auth.rs, tasks.rs                 (≈ app/routers)
  lib.rs    module root (new — see below)
  main.rs   thin binary entry point (≈ main.py)
  error.rs  new — see below
migrations/
  001_create_users.sql
  002_create_tasks.sql
tests/
  e2e.rs
```

`lib.rs` + a thin `main.rs` isn't in the Python original — it exists so
`cargo test` can run doctests (they only execute against a library target,
not a bare binary crate). `error.rs` is also new: Rust has no exception
mechanism to piggyback on the way `raise HTTPException(...)` does, so
there's one central `ApiError` type that every handler returns through `?`,
producing the same `{"detail": "..."}` JSON body FastAPI's default error
handler does.

## Setup

Needs a Postgres server and `libpq` to link against:

```bash
apt-get install -y libpq-dev postgresql   # or your platform's equivalent
```

Configuration is read from the environment (falling back to the defaults
below if unset — there's no `.env` file included, set these however your
environment normally does, or export them directly):

| Variable                       | Default                                              |
|---------------------------------|-------------------------------------------------------|
| `DATABASE_URL`                  | `postgresql://user:password@localhost:5432/taskdb`     |
| `SECRET_KEY`                    | `change-me-in-production`                              |
| `ALGORITHM`                     | `HS256`                                                |
| `ACCESS_TOKEN_EXPIRE_MINUTES`   | `30`                                                   |

## Running

```bash
cargo run
```

Tables are created automatically on startup (`CREATE TABLE IF NOT EXISTS`,
safe on every boot — see `src/db/session.rs`). The Python original imports
the model modules "to ensure tables are created" but never actually calls
`Base.metadata.create_all`, so on a genuinely fresh database it wouldn't
have created them either; this does it for real, executing the DDL in
`migrations/001_create_users.sql` and `migrations/002_create_tasks.sql`
directly (no migration-runner crate involved — see the comment in
`session.rs` for why).

Server listens on `0.0.0.0:8000`, same routes as the original:
`POST /auth/register`, `POST /auth/login`, `GET|POST /tasks`,
`GET|PATCH|DELETE /tasks/{id}`, `GET /health`.

## Testing

```bash
cargo test
```

Runs unit tests (password hashing, JWT round-trip), doctests, and an
end-to-end smoke test (`tests/e2e.rs`) that spawns the real compiled binary
and drives the whole API over actual HTTP against your local Postgres —
registration, duplicate-email rejection, login success/failure, full task
CRUD, pagination, filtering, query-param validation, and the PATCH
null-clearing semantics described below. Needs Postgres reachable at the
`DATABASE_URL` in your environment (defaults to the same `taskdb` used by
`cargo run`).

## A note on the fixed toolchain

`Cargo.toml` declares `rust-version = "1.75"` because that's a real,
enforced constraint here, not a suggestion — the environment this was built
in has rustc pinned at 1.75 with no upgrade path, which sits below the MSRV
of a lot of current crate releases. `Cargo.lock` is committed with every
dependency already pinned to a version that builds cleanly under 1.75, so a
plain `cargo build` should just work as-is.

If you add a new dependency later and hit an MSRV error (typically
`feature 'edition2024' is required` or `requires rustc X.YY or newer`),
you're looking for an older release of the offending crate: check its
version history on crates.io for one whose declared MSRV is ≤ 1.75, then
`cargo update -p <crate> --precise <version>`. Occasionally the version you
want gets rejected by the resolver because some other crate's requirement
is too tight — in that case the same trick applied to *that* crate (found
via `grep -B15 '"<crate>",' Cargo.lock`) usually unblocks it.

## Notable deliberate deviations from the Python original

Each of these is also called out as a comment at the relevant spot in the
code:

- **`status`/`priority` stored as `TEXT` + `CHECK`, not a native Postgres
  enum.** SQLAlchemy's `Enum(...)` creates a native enum type by default;
  mapping that cleanly in Diesel needs either a native-enum SQL type dance
  or an extra crate. Storing as text (with hand-written `ToSql`/`FromSql`
  on the `Status`/`Priority` enums, serializing to the exact same strings)
  is simpler and behaves identically from the API's point of view.
- **`ON DELETE CASCADE` added at the DB level** on `tasks.owner_id`. The
  Python model's `cascade="all, delete"` is SQLAlchemy-ORM-session-level
  only (no `ondelete=` on the actual `ForeignKey`), so it would only fire
  when deleting a `User` through the ORM. There's no user-delete endpoint
  in this API surface either way, so this is a no-op today, but it matches
  the model's evident intent if that endpoint shows up later.
- **CORS: `CorsLayer::very_permissive()` instead of a literal
  `allow_origins=["*"]` + `allow_credentials=True`.** That combination is
  spec-invalid CORS (a wildcard origin can't be paired with credentialed
  requests — browsers reject it), and tower-http's `CorsLayer` refuses to
  build it at all rather than ship something that silently breaks in the
  browser. `very_permissive()` reflects the requesting origin instead of a
  literal `*`, which is the spec-legal version of what the original config
  seems to have been reaching for. Since this API authenticates via a
  Bearer token header rather than cookies, this doesn't change how any real
  client talks to it.
- **PATCH null-clearing.** `TaskUpdate.description`/`due_date` use
  `Option<Option<T>>` with a small custom deserializer, to preserve the
  same three-state distinction Pydantic's `exclude_unset=True` draws:
  field omitted (leave column alone) vs. field explicitly `null` (clear the
  column) vs. field present with a value.
- **Request validation errors return 422**, matching FastAPI/Pydantic's
  default, rather than Axum's default 400 — `ValidatedJson`/`ValidatedQuery`
  in `core/deps.rs` wrap the built-in extractors to remap this.
