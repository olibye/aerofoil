# Proposal: Add GitHub Actions CI/CD

## Problem

The project currently lacks a continuous integration and continuous delivery (CI/CD) pipeline. This means that tests are not automatically run on new code, and there is no automated process for building and releasing the software. This slows down development and increases the risk of introducing regressions.

## Proposed Solution

Introduce GitHub Actions to create a CI/CD pipeline for the project. This pipeline will:

1.  **Run on every push and pull request:** Automatically run tests to ensure that new changes don't break existing functionality.
2.  **Test feature flags:** Run tests for both the `rusteron` and `aeron-rs` features to ensure that both transport layers are working correctly.
3.  **Build the project:** Ensure that the project builds correctly.
4.  **Lint the code:** Check for code style issues.
5.  **(Future) Create releases:** Automate the process of creating new releases.

## High-Level Plan

1.  Create a new GitHub Actions workflow file in `.github/workflows/`.
2.  Define a CI job that runs on push and pull requests to the `main` branch.
3.  The CI job will have steps to:
    a. Checkout the code.
    b. Install the Rust toolchain.
    c. Run `cargo test --all-features` to test with all features enabled.
    d. Run `cargo test --no-default-features --features rusteron` to test only with the `rusteron` feature.
    e. Run `cargo test --no-default-features --features aeron-rs` to test only with the `aeron-rs` feature.
    f. Run `cargo clippy` to lint the code.
    g. Run `cargo build` with the same feature combinations.
