# zero2prod

Going through the book From Zero to Production with Rust and documenting my learnings.

## Notes

### Chapter 1

- inner development loop:
  - make changes
  - compile
  - run tests
  - execute the application
- -> the faster the inner development loop, the more iterations fit into the same unit of time (eg. 1h)
- IDE setup
  - `rust-analyzer` for rust lang-server
  - adjust `.cargo/config.toml` to use `zld` for linking (faster than default linker)
    - do not use `zld`, because it has been deprecated end of 2022
- project setup
  - `cargo watch`: watch changes
  - `cargo test`: run tests
  - `cargo clippy`: linter
  - `cargo fmt`: format
  - `cargo audit`: security audits for dependencies
- CI setup
  - audit -> format -> lints -> compile -> tests

### Chapter 2

- user stories: as an X I want to Y so I can Z

### Chapter 3

#### choose web framework

`actix-web`

- a big reason is that it runs on `tokio`
- what is tokio? -> an asynchronous Rust runtime
- useful links:

  - website [Link](https://actix.rs/)
  - docs [Link](https://docs.rs/actix-web/4.0.1/actix_web/index.html)
  - examples [Link](https://github.com/actix/examples)

#### basic health check

`/health_check`

We use

```rust
HttpServer::new(|| {
    App::new()
        .route("/", web::get().to(greet))
        .route("/{name}", web::get().to(greet))
})
```

to:

- setup a new `HttpServer` - handles _transport level_, IP addresses, number of concurrent connections, TLS enabled? etc.
- it instantiates an `App` - it works on app level logic like routing, middleware, request handlers; uses fluent API / builder pattern;
- `||` is a [closure](https://doc.rust-lang.org/book/ch13-01-closures.html); that is like a function handler, but it can capture values from current context
- `web::get()` is a short cut for `Route::new().guard(guard::Get())`; a guard is a trait with `Guard::check`
- the first parameter of `route` is a path (possibly templated string); the second argument is a `route`, instance of the `Route` struct
- `greet` is a handler that gets wrapped by the guard; it returns something that implements `Responder` trait:

  ```rust
  async fn greet(req: HttpRequest) -> impl Responder {
      let name = req.match_info().get("name").unwrap_or("World");
      format!("Hello {}!", &name)
  }
  ```

- The `HttpServer` statement is wrapped with

  ```rust
  #[tokio::main]
  async fn main() -> Result<(), std::io::Error> {
    //
  }
  ```

  This instantiates an async function.

- async in Rust is implemented using `Future`; however, in constrast to other languages, Rust implements pull-based futures; futures can be polled to check if there is a result available
- however, by design and ootb, Rust does not come with async. needs to be provided by an extra cargo package. so Rust by itself does not know what `async` is, because there's nothing to call `poll`
- this is why we add the macro `#[tokio::main]` - what does it do? -> we can use `cargo expand` to check it out!
- `cargo install cargo-expand` to expand macros into their actual output

#### integration tests

- we choose to add integration test, no unit tests needed (testing only calling the `health_check` function wouldn't ensure that a HTTP `GET` on `/health_check` is succeeding)
- three locations for tests in Rust:
  - next to the code (embedded), behind a `#[cfg(test)]` flag -> gets direct access to private fields etc.
  - in doc-comments
  - in a separate `./tests` folder parallel to `./src` -> gets compiled to its own binary
- follow Arrange -> Act -> Assert pattern

- preparing `./tests`

  - we need to make our `main` function exportable; right now it's simply compiled into a binary
  - using `tokio::spawn` to spawn background app; it will be shutdown when the surrounding `tokio` runtime stops; in our case, when test ends
  - spawning on port 0 lets OS find an available port

- the `?` operator - can be used on functions with `Result<T, E>`; equivalent to a pattern match, where errors result in an early `return Err(From::from(err))`; `From` is a error conversation to standard error; for success, matched on `Ok(T)`

Notes:

- naming a parameter with an underscore, eg. `_req`, will signal to the compiler that it's an unused parameter
- running `cargo doc --open` generates html docs offline and opens them in the browser

#### subscribing new users to the newsletter

TODO: definitely revise this part of the chapter, a lot is happening behind ` #[derive(Serialize)]` and `#[derive(Deserialize)]`!

- using parametrized tests `Vec<(&str, &str)>`
- serde

  - Check Understanding Serde by Josh Mcguigan [Link](https://www.joshmcguigan.com/blog/understanding-serde/)
  - monomorphization
    - Rust compiler replaces all generics at runtime with the concrete types; then optimizes > no runtime costs for generics
    - this is known as a zero-cost abstraction: at the same time, easier readable for humans & no performance loss
    - Rust does not provide runtime reflection; all reflection work needs to be done at compile time

#### storing data: databases

- as of August 2020, three big PostgreSQL packages in Rust:
  - `tokio-postgres`
  - `sqlx`
  - `diesel`
- pick according to
  - compile-time safety
    - `sqlx` and `diesel` provide some kind of compile-time checks for SQL queries:
      - `diesel`: code generation using a CLI to generate a representation of the data schema in Rust
      - `sqlx`: usage of macros to connect to a database at compile-time and check if the queries are sound
  - SQL-first vs. DSL for query building
    - `diesel`: provides their own query builder
  - async vs. sync interface
    - `sqlx` and `tokio-postgres` are async
    - `diesel` is sync, no async support planned

-> here we pick `sqlx`

#### integration tests with side-effects

- simple setup script for database setup
- using `sqlx` cli to create postgres database; simple checks if `sqlx` and `psql` are installed; use `psql` to poll until postgres db is up and running
- `sqlx` needs `DATABASE_URL` to be set; set it in fish:

```sh
set DATABASE_URL postgres://postgres:password@127.0.0.1:5432/newsletter
```

- create new (empty) migration: `sqlx migrate add create_subscriptions_table`
- primary key: use _natural key_ (business meaning) vs. _surrogate key_ (synthethic, id)
- run `set -x SKIP_DOCKER true; ./script/init_db.sh` in fish shell

- `sqlx` with feature flag `postgres` exposes `PgConnection` [Link](https://docs.rs/sqlx/latest/sqlx/struct.PgConnection.html)

- configuration management with the crate `config`
  - eg., different constants for different environments (local, dev, staging)
- define modules in `mod.rs`, expose with

```rust
mod subscriptions;

pub use subscriptions::*;
```

#### persisting a new subscriber

- `actix-web` provides _application state_ using the `app_data` method on `App`
- this can be used to attach stateful things to request handlers; like database connection!
- we need a database connection in the `subscribe` handler, so adding `connection: PgConnection` to the run method;
  - compiler error, `HttpServer` expects the returned `App` to be cloneable (i.e., satisfy the `Clone` trait), but `PgConnection` does not!
  - this is because `HttpServer::new` accepts a closure returning an `App` struct; not an `App` struct directly!
  - `actix-web` spins up a worker for each core on the system, so each worker creates their own `App` instance
  - needs to be cloneable to have one copy of `App` for every worker
  - solution -> wrap `PgConnection` in `web::Data`, a `actix-web` extractor; instead of a raw copy of the connection, it will get a pointer to a connection; the pointer is cloneable! this is called _Atomic Reference Counter pointer_, or an `Arc`
  - `Arc<T>` is always cloneable
  - we then use `move` to extract connection from the context
- back in `subscribe`, we add `_connection`:
  ```rust
  pub async fn subscribe(
      _form: web::Form<FormData>,
      _connection: web::Data<PgConnection>,
  ) -> HttpResponse {
      HttpResponse::Ok().finish()
  }
  ```
  - how does it know where `_connection` is coming from?
  - `actix-web` uses a type-map; it checks which item in the application state has type `PgConnection`; in this case only one!
    - TODO: what if there are multiple items with same type?
  - this mechanism is similar to dependency injection in other languages
  - raw strings in Rust :
  ```rust
  r#"
    This is a raw string
  "#
  ```
  - `sqlx`'s `execute` needs an argument that implements the `Executor` trait. `&PgConnection` does not!
    - `&mut PgConnection` does!
    - `sqlx` uses an asynchronous interface, but only allows a single concurrent query using the same database connection!
    - to enforce this, it needs a mutable reference, immutable reference doesn't work for this
    - because mutable reference ~~ unique reference; enforced by compiler
    - but using `&mut PgConnection` would limit us to one concurrent connection - one slow query would slow down all queries
    - solution -> use a connection pool instead! `&Pool`
    - the reference itself is still unique, but `sqlx` will receive a new connection from the pool when it's free, and pool contains multiple connections
- refactoring of main, subscribe and tests to use connection pool instead
  - in tests, we define new struct `TestApp` for all required info for tests to "Arrange" - in this case, we added connection pool and address fields here for now
  - now running tests multiple times fails, because we are not cleaning up test data correctly yet -> test isolation!

#### test isolation

- two high-level approaches to ensure test isolation for db:
  - wrap the test in a transaction; roll it back at the end
  - spin up a brand-new db at the start of each test

### Chapter 4 - Telemetry

#### Known unknowns vs Unknown unknowns

- known unknowns -> we could tackle most of these; egs:
  - what happens when malicious input is sent with the `POST` to `/subscriptions`?
  - what happens if we lose the db connection? does `sqlx::PgPool` automatically recover?
- unknown unknowns -> this is where telemetry is crucial
  - experience can sometimes turn unknown unknowns into known unknowns (like DB pool eg above)
  - some external state that might trigger unknown unknowns:
    - system is pushed outside of its usual operating conditions (eg huge traffic spike)
    - multiple components fail at the same time
    - a change is introduced that moves the system equilibrium (eg. retry policy is changed)
    - no changes have been made in a long while; no deploys in a long while (eg. memory leaks become apparent)
  - _they are often difficult to reproduce outside of the live environment!_
  - so then, how can we prepare for an unknown unknown to occur? -> observability

#### Observability

- preparing logs, visibility into the system etc. _before_ an incident happens
- we need to predict what information we need during an incident
- quote from Honeycomb website [Link](https://www.honeycomb.io/what-is-observability)
  > Observability is about being able to ask arbitrary questions about your environment without — and this is the key part — having to know ahead of time what you wanted to ask.
- s/arbitrary/sufficiently detailed

In the end, we need:

1. to instrument our app to collect high-quality telemetry data
1. tools and systems to efficiently slice, dice and manipulate the data to find answers to our questions

#### Logging

- logs are the most common type of telemetry data
- rust offers the `log` crate ootb
  - macros: `trace`, `debug`, `info`, `warn` and `error` - built-in log levels
- common pattern: use function returning `Result<String, String>` or some other `<Ok, Err>` return type, then trace on success and error on error
- `actix-web` comes with `Logger` middleware
- when we now run `curl http://127.0.0.1:8000/health_check -v`... nothing gets logged?
  - -> `log` provides `set_logger` method to choose actual logger that determines logging behavior [Link](https://docs.rs/log/latest/log/fn.set_logger.html)
  - here we use `env_logger` [Link](https://docs.rs/env_logger/latest/env_logger/), a logger mostly used to print to terminal; log-level can be controlled with `RUST_LOG` env var
- useful rule of thumb: _closely monitor interactions with other systems over the network_
- to log errors, we are using ` {:?}`, the `std::fmt::Debug` format, to capture the query error.

  ```rust
  log::error!("Failed to execute query: {:?}", e);
  ```

- adding a request id to all logs so we can correlate them

#### Structured Logging - Tracing

- `tracing` crate [Link](https://docs.rs/tracing/latest/tracing/)
- we create traces that come with a `Span` (start and end time) and do logging automatically, using instrumentation!
- when instrumenting the database calls like so:

  ```rust
  let query_span = tracing::info_span!("Saving new subscriber details in the database");
  // ...
  match sqlx::query!(/* */)
    .execute(pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
  ```

  and run `RUST_LOG=trace cargo run`, we see logs like this:

  ```sh
  [.. INFO  zero2prod] Saving new subscriber details in the database
  [.. TRACE zero2prod] -> Saving new subscriber details in the database
  [.. TRACE zero2prod] <- Saving new subscriber details in the database
  [.. TRACE zero2prod] -> Saving new subscriber details in the database
  [.. TRACE zero2prod] <- Saving new subscriber details in the database
  [.. TRACE zero2prod] -> Saving new subscriber details in the database
  [.. TRACE zero2prod] <- Saving new subscriber details in the database
  # ...
  [.. TRACE zero2prod] -- Saving new subscriber details in the database
  ```

  this is because, once again, `async` in Rust is pull-based! what we see here is the runtime polling the `executor` until it is done.

- our original issue of having request ids go along logs is not solved yet - because we're still using `env_logger`!
- the `Subscriber` trait in `tracing` is equivalent to `Log` trait in `log` - [Link](https://docs.rs/tracing/latest/tracing/trait.Subscriber.html)
- find subscribers in `tracing-subscriber`; it also introduces two more key traits:
  - `Layer`: we can build a processing pipeline for spans data using layers
  - `Registry`: goes together with the layering approach. it is responsible for storing span metadata, recording relationships between spans, and tracking active and closed spans.
- in the end, we use three layers:
  - `tracing_subscriber::filter::EnvFilter` to discard lower log levels
  - `tracing_bunyan_formatter::JsonStorageLayer` to process spans data and associated metadata in easy-to-consume JSON (port from node's `bunyan`)
  - `tracing_bunyan_formatter::BunyanFormatterLayer` builds on `JsonStorageLayer` and outputs logs in bunyan-compatible format
- with all of this, we lost the logs from library code; while traces automatically emit logs (we were able to output traces with `env_logger`), the opposite isn't true; we need an additional logger that explicitely turns logs into traces; `tracing-log` does exactly that
- `cargo-udeps` to remove unused dependencies

#### Logs for integration tests

- `cargo test` swallows log by default, show all logs with `cargo test --no-capture`
- traces are not swallowed by default -> add a sink to `get_subscriber`
- higher-ranked trait bound (HRTB):

```rust
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
// this is a higher-ranked trait bound (HRTB)
// It basically means that Sink implements the `MakeWriter` trait for all choices of the lifetime parameter `'a`
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
  // [...]
}
```

- important concepts:

  - ownership [Link](https://doc.rust-lang.org/nomicon/ownership.html)
  - higher-ranked trait bounds [Link](https://doc.rust-lang.org/nomicon/hrtb.html)
  - lifetimes [Link](https://doc.rust-lang.org/nomicon/lifetimes.html)

#### Cleaning up instrumentation code

- instead of interlacing code in `subscribe` (for example) with tracing instrumentation, we'd rather _wrap_ the function in a span -> common pattern
- using `tracing::instrument` procedural macro
- _pit of success_ - the right thing to do is the easiest thing to do
- refactored `subscriptions.rs` to have two functions:
  - `subscribe`: take input from web/form context and prepare arguments; delegate to `insert_subscriber` for actual db logic
- all fields by default are passed to the `tracing::instrument` macro - high risk to leak private/sensitive data like passwords!
  - -> using `secrecy` crate to exclude fields
