# chemquiz

A Rust crate for building a chemistry quiz application. The current focus is core quiz
generation logic that can be embedded into a future Leptos frontend.

## Modules
- `compound`: Compound data model with formatted display helpers for names and structures.
- `catalog`: Hierarchical categorization for compounds with filtering utilities for menu-based selection.
- `quiz`: Quiz mode definitions, quiz item structure, and randomized quiz generation.
- `demo`: Ready-to-use dataset and catalog for UI previews and integration checks.

## Usage
```rust
use chemquiz::{CatalogError, QuizMode, demo_catalog, generate_quiz};
use rand::SeedableRng;

let catalog = demo_catalog();
let alcohols = catalog
    .compounds_for(&vec!["Organic".to_string(), "Alcohols".to_string()])
    .map_err(|error| match error {
        CatalogError::CategoryNotFound { path } => format!("missing path: {}", path),
        CatalogError::EmptyPath => "no category selected".to_string(),
    })?;

let mut rng = rand::rngs::StdRng::seed_from_u64(42);
let quiz = generate_quiz(&mut rng, &alcohols, QuizMode::NameToStructure, 2)?;
```

The generator returns a prompt and a set of unique options; seeding the RNG keeps results
reproducible for testing.

## Development
Install Rust (edition 2024) and run the test suite:

```bash
cargo test
```

Unit tests cover the compound formatting helpers, catalog filtering, and quiz edge cases.
Integration tests validate end-to-end quiz generation from catalog slices alongside typical and
edge configurations.

## Frontend preview (WASM)

A Leptos client app is available for GitHub Pages. Build it locally with Trunk:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --public-url /chemquiz/
```

The app mounts to the page body, toggles light/dark themes, and generates sample quizzes using the
`demo` module. The `--public-url` flag keeps asset paths compatible with GitHub Pages.

## Deploy to GitHub Pages

A workflow at `.github/workflows/gh-pages.yml` builds the WASM bundle with Trunk and publishes the
`dist/` directory to GitHub Pages. Trigger it via pushes to `main` or by running the workflow
manually.
