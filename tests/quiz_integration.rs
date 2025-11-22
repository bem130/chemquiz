use chemquiz::{Compound, QuizMode, generate_quiz};
use rand::SeedableRng;

fn sample_compounds() -> Vec<Compound> {
    vec![
        Compound {
            iupac_name: "ethanol".to_string(),
            common_name: Some("ethyl alcohol".to_string()),
            local_name: Some("エタノール".to_string()),
            skeletal_formula: "CH3-CH2-OH".to_string(),
            molecular_formula: "C2H6O".to_string(),
        },
        Compound {
            iupac_name: "methanol".to_string(),
            common_name: Some("methyl alcohol".to_string()),
            local_name: Some("メタノール".to_string()),
            skeletal_formula: "CH3OH".to_string(),
            molecular_formula: "CH4O".to_string(),
        },
        Compound {
            iupac_name: "propanone".to_string(),
            common_name: Some("acetone".to_string()),
            local_name: Some("アセトン".to_string()),
            skeletal_formula: "(CH3)2CO".to_string(),
            molecular_formula: "C3H6O".to_string(),
        },
        Compound {
            iupac_name: "benzene".to_string(),
            common_name: None,
            local_name: Some("ベンゼン".to_string()),
            skeletal_formula: "C6H6".to_string(),
            molecular_formula: "C6H6".to_string(),
        },
    ]
}

#[test]
fn deterministic_generation_from_seed() {
    let compounds = sample_compounds();
    let mut rng = rand::rngs::StdRng::seed_from_u64(99);

    let quiz = generate_quiz(&mut rng, &compounds, QuizMode::StructureToName, 4)
        .expect("quiz should be generated with provided data");

    assert_eq!(quiz.prompt, "C6H6 (C6H6)");
    assert_eq!(quiz.options.len(), 4);
    assert_eq!(quiz.options[quiz.correct_index], "benzene / ベンゼン");
}

#[test]
fn options_are_unique() {
    let compounds = sample_compounds();
    let mut rng = rand::rngs::StdRng::seed_from_u64(120);

    let quiz = generate_quiz(&mut rng, &compounds, QuizMode::NameToStructure, 3)
        .expect("quiz should be generated with unique options");

    let mut seen = std::collections::HashSet::new();
    for option in &quiz.options {
        assert!(seen.insert(option));
    }
}

#[test]
fn prompt_matches_selected_option() {
    let compounds = sample_compounds();
    let mut rng = rand::rngs::StdRng::seed_from_u64(5);

    let quiz = generate_quiz(&mut rng, &compounds, QuizMode::StructureToName, 4)
        .expect("quiz should generate");

    let matched = compounds
        .iter()
        .find(|compound| compound.display_structure() == quiz.prompt)
        .expect("prompt should match provided compounds");

    assert_eq!(quiz.options[quiz.correct_index], matched.display_name());
}
