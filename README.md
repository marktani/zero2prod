# zero2prod

Going through the book From Zero to Production with Rust and documenting my learnings.

## Notes

- chapter 1
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
    - `cargo test`: run tests
    - `cargo clippy`: linter
    - `cargo fmt`: format
    - `cargo audit`: security audits for dependencies
  - CI setup
    - audit -> format -> lints -> compile -> tests
- chapter 2
  - user stories: as an X I want to Y so I can Z
- chapter 3
  - choose web framework -> `actix-web`
    - a big reason is that it runs on `tokio`
    - what is tokio? -> an asynchronous Rust runtime
    - useful links:
      - website [Link](https://actix.rs/)
      - docs [Link](https://docs.rs/actix-web/4.0.1/actix_web/index.html)
      - examples [Link](https://github.com/actix/examples)
  - base health check `/health_check`
