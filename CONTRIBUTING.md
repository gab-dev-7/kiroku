# Contributing to Kiroku

Thank you for your interest in contributing to **Kiroku**! We welcome contributions from everyone.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/your-username/kiroku.git
    cd kiroku
    ```
3.  **Create a branch** for your feature or bugfix:
    ```bash
    git checkout -b feature/amazing-feature
    ```

## Development Workflow

Kiroku is written in **Rust**. Ensure you have the latest stable toolchain installed.

### Running the App
To run the application locally:
```bash
cargo run
```

### Running Tests
We use standard `cargo test` for unit and integration tests. Please ensure all tests pass before submitting a PR.
```bash
cargo test
```

### Formatting & Linting
We enforce standard Rust formatting and linting rules.
```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## Submitting Changes

1.  **Commit your changes** with clear, descriptive messages.
    - Good: `feat: implement folder navigation`
    - Bad: `fix stuff`
2.  **Push to your fork**:
    ```bash
    git push origin feature/amazing-feature
    ```
3.  **Open a Pull Request** against the `main` branch of the upstream repository.
4.  Describe your changes in detail and link to any relevant issues.

## Code of Conduct

Please be respectful and considerate in all interactions. We want to build a welcoming community.
