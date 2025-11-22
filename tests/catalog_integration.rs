use chemquiz::{CatalogError, QuizMode, demo_catalog, generate_quiz};
use rand::SeedableRng;

fn organic_alcohols_path() -> Vec<String> {
    vec!["Organic".to_string(), "Alcohols".to_string()]
}

#[test]
fn can_generate_quiz_from_category_subset() {
    let catalog = demo_catalog();
    let compounds = catalog
        .compounds_for(&organic_alcohols_path())
        .expect("category should exist");
    let mut rng = rand::rngs::StdRng::seed_from_u64(77);

    let quiz = generate_quiz(&mut rng, &compounds, QuizMode::StructureToName, 2)
        .expect("category should have enough variety");

    assert_eq!(quiz.options.len(), 2);
}

#[test]
fn category_errors_surface() {
    let catalog = demo_catalog();
    let error = catalog
        .compounds_for(&vec!["Organic".to_string(), "Nonexistent".to_string()])
        .expect_err("missing subcategory should return error");

    assert_eq!(
        error,
        CatalogError::CategoryNotFound {
            path: "Organic / Nonexistent".to_string(),
        }
    );
}
