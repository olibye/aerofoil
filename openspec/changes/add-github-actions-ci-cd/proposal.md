# Proposal: Add GitHub Actions CI/CD

## Why

The project currently lacks a continuous integration and continuous delivery (CI/CD) pipeline. This means that tests are not automatically run on new code, and there is no automated process for building and releasing the software. This slows down development and increases the risk of introducing regressions.

## What Changes

Introduce GitHub Actions to create a CI/CD pipeline for the project. This pipeline will:

1. **Run on every push and pull request:** Automatically run tests to ensure new changes don't break existing functionality.
2. **Build the project:** Ensure the project builds correctly with the `embedded-driver` feature.
3. **Lint the code:** Check for code style issues with clippy.
4. **Check formatting:** Ensure code follows rustfmt standards.
5. **Build benchmarks:** Verify benchmarks compile, including the allocation tracking benchmark with `dhat-heap` feature.

## Impact

- Affected code: `.github/workflows/rust.yml` (new file)
- Tests and builds will run automatically on CI
- PRs will be blocked if tests, linting, or formatting checks fail
- Benchmarks verified to compile but not run (too slow for CI)
