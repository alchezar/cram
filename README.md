# cram

A local web app for practising English B2 (FCE) grammar. A small [axum] server
renders quiz pages with [maud] + [htmx] (almost no hand-written JavaScript);
quizzes live in TOML files and per-question progress is stored in SQLite.

## Features

- Topic roadmap on the index page with a mastery bar per topic
- Multiple-choice and free-text questions, reshuffled on every reload
- A question drops out after 5 correct answers in a row; stars show the streak
- Per-topic "reset progress"
- Opens the index in your browser on start and logs a LAN URL to open from a phone
- Keeps the machine awake while serving so it stays reachable

## Requirements

- A recent stable **Rust** toolchain (2024 edition; built with 1.97) and `cargo`
- Optional, only for the database/test `make` targets:
  - `cargo install sqlx-cli` - migrations and offline query metadata
  - `cargo install cargo-nextest` - the test runner used by `make test`

The compiled SQLx query cache (`.sqlx/`) is committed, so a plain build needs no
database connection.

## Setup

```bash
cp cram.toml.example cram.toml
```

By default the database is created under the platform data dir on first run
(macOS `~/Library/Application Support/cram/cram.db`, Linux
`~/.local/share/cram/cram.db`), so no path needs configuring. Set `database_url`
in `cram.toml` only to override that location.

The `make` targets that touch the database (`migrate`, `prepare`, `dev`, `test`)
use compile-time sqlx macros and still need a `.env`. Copy the example; its
`DATABASE_URL` uses `${HOME}`, so no path editing is needed on most machines:

```bash
cp .env.example .env
```

## Build and run

Using the Makefile (run `make help` to list every target):

```bash
make run        # build and run in release mode
make dev        # run in debug mode (loads .env)
make release    # build a release binary and copy it to ./cram
make build      # debug build
make lint       # clippy, warnings treated as errors
make fmt        # check and apply formatting
make migrate    # apply pending migrations
```

Or plain cargo from the project root:

```bash
cargo run
```

On start the server prints a local and a LAN URL and opens the index in your
default browser; open the LAN URL on a phone connected to the same Wi-Fi.

To install the binary system-wide: `cargo install --path .` puts `cram` in
`~/.cargo/bin`. It still reads `cram.toml`, `quizzes/`, `roadmap.toml` and `web/`
relative to the working directory, so run it from the project root.

## Layout

```
src/            server: lib.rs (bootstrap) + main.rs (entry point)
  db/           SQLite pool, migrations, progress queries
  models/       quiz and roadmap TOML models
  render.rs     HTML rendering (maud)
  route.rs      HTTP routes and handlers
quizzes/        one TOML file per quiz
roadmap.toml    index topics grouped into sections
web/            static assets (style.css, htmx.min.js)
migrations/     SQLite schema
```

[axum]: https://github.com/tokio-rs/axum
[maud]: https://maud.lambda.xyz
[htmx]: https://htmx.org
