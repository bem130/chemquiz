# chemquiz

A Rust crate for building a chemistry quiz application. The current focus is core quiz
generation logic that can be embedded into a future Leptos frontend.

## Modules
- `compound`: Compound data model with formatted display helpers for names and structures (including optional SMILES strings for rendering) plus optional series formulas, functional group metadata, and notes.
- `catalog`: Hierarchical categorization for compounds with filtering utilities for menu-based selection.
- `quiz`: Quiz mode definitions, quiz item structure, and randomized quiz generation.
- `demo`: Ready-to-use dataset and catalog for UI previews and integration checks.

## Usage
```rust
use chemquiz::{CatalogError, QuizMode, demo_catalog, generate_quiz};
use rand::SeedableRng;

let catalog = demo_catalog();
let alcohols = catalog
    .compounds_for(&vec![
        "Organic".to_string(),
        "Aliphatic_compounds".to_string(),
        "Alcohols_and_ethers".to_string(),
    ])
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

## JSON catalog

Compound lists live under `catalog/` as JSON files organized by folder hierarchy. The
`catalog/index.json` manifest exposes the available paths for the WASM frontend, which fetches and
deserializes the selected file at runtime using `serde`.

Each compound entry can optionally include:

- `series_general_formula`: a generalized formula describing the family to which the compound belongs.
- `functional_groups`: an array of `{ name_en, name_ja, pattern }` objects describing functional groups.
- `notes`: free-form descriptive text about properties or handling.
- `smiles`: a SMILES string for structure rendering when available.

## Frontend preview (WASM)

A Leptos client app is available for GitHub Pages. Build it locally with Trunk:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --public-url /chemquiz/
```

The app mounts to the page body and mirrors the `quizprototype.html` layout: a top settings panel
shows quiz mode, a skeletal/full structure toggle, score, and progress. Users can browse a catalog
tree to load JSON datasets, then start a session that cycles through Menu → Game → Result scenes.
Each quiz view matches the prototype with dedicated prompt and option cards, hint toggles, and a
feedback row. Structural prompts respect the view toggle (RDKit MinimalLib + Kekule.js), while
molecular formulas render through KaTeX/mhchem when present and fall back to text when SMILES data
is unavailable. The `--public-url` flag keeps asset paths compatible with GitHub Pages.

## Deploy to GitHub Pages

A workflow at `.github/workflows/gh-pages.yml` builds the WASM bundle with Trunk and publishes the
`dist/` directory to GitHub Pages. Trigger it via pushes to `main` or by running the workflow
manually.
