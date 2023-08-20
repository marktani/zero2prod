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
