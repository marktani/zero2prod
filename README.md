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

- choose web framework -> `actix-web`

  - a big reason is that it runs on `tokio`
  - what is tokio? -> an asynchronous Rust runtime
  - useful links:
    - website [Link](https://actix.rs/)
    - docs [Link](https://docs.rs/actix-web/4.0.1/actix_web/index.html)
    - examples [Link](https://github.com/actix/examples)

- base health check `/health_check`

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

- integration tests

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
