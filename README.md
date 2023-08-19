# zero2prod

Going through the book From Zero to Production with Rust and documenting my learnings.

## Notes

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
