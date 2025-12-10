# Tasks: Add GitHub Actions CI/CD

- [x] Create `.github/workflows/rust.yml`
- [x] Add a step for checking formatting with `cargo fmt --check`
- [x] Add a step for linting the code with `clippy --features embedded-driver`
- [x] Add a step for running tests with `--features embedded-driver`
- [x] Add a step for building with `--features embedded-driver`
- [x] Add a step for building benchmarks with `--features embedded-driver`
- [x] Add a step for building allocation benchmarks with `--features embedded-driver,dhat-heap`
- [x] Ensure the workflow is triggered on pushes and pull requests to `main`/`master`
