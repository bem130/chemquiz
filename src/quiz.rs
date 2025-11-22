use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashSet;

use crate::compound::Compound;

/// Quiz type describing the relationship between prompt and answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuizMode {
    /// Prompts with a compound name and expects the skeletal structure as the answer.
    NameToStructure,
    /// Prompts with a skeletal structure and expects the compound name as the answer.
    StructureToName,
}

/// A single generated quiz question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuizItem {
    pub mode: QuizMode,
    /// Text shown as the question prompt.
    pub prompt: String,
    /// List of answer options. Length is always `option_count` passed to the generator.
    pub options: Vec<String>,
    /// Index in `options` that contains the correct answer.
    pub correct_index: usize,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum QuizError {
    #[error("requires at least {required} compounds but only {available} provided")]
    NotEnoughCompounds { required: usize, available: usize },
    #[error("requires at least {required} unique options but only {unique} available")]
    InsufficientUniqueOptions { required: usize, unique: usize },
    #[error("option count must be at least 2")]
    OptionCountTooSmall,
}

/// Generates a quiz item for the given compounds and quiz mode.
///
/// The function picks unique compounds at random using the supplied random number generator,
/// making it deterministic for testing when used with a seeded RNG.
///
/// # Errors
/// * Returns [`QuizError::OptionCountTooSmall`] if `option_count` is less than 2.
/// * Returns [`QuizError::NotEnoughCompounds`] if fewer than `option_count` compounds are available.
/// * Returns [`QuizError::InsufficientUniqueOptions`] if the provided compounds do not contain
///   enough unique names or structures for the requested `option_count`.
pub fn generate_quiz<R: Rng + ?Sized>(
    rng: &mut R,
    compounds: &[Compound],
    mode: QuizMode,
    option_count: usize,
) -> Result<QuizItem, QuizError> {
    if option_count < 2 {
        return Err(QuizError::OptionCountTooSmall);
    }

    if compounds.len() < option_count {
        return Err(QuizError::NotEnoughCompounds {
            required: option_count,
            available: compounds.len(),
        });
    }

    let mut seen = HashSet::new();
    let mut unique_indices = Vec::new();

    for (idx, compound) in compounds.iter().enumerate() {
        let label = match mode {
            QuizMode::NameToStructure => compound.display_structure(),
            QuizMode::StructureToName => compound.display_name(),
        };

        if seen.insert(label) {
            unique_indices.push(idx);
        }
    }

    if unique_indices.len() < option_count {
        return Err(QuizError::InsufficientUniqueOptions {
            required: option_count,
            unique: unique_indices.len(),
        });
    }

    let mut selected = unique_indices;
    selected.shuffle(rng);
    selected.truncate(option_count);

    let correct_compound_index = selected[0];

    let mut options: Vec<(usize, String)> = selected
        .iter()
        .map(|idx| {
            let compound = &compounds[*idx];
            let option_text = match mode {
                QuizMode::NameToStructure => compound.display_structure(),
                QuizMode::StructureToName => compound.display_name(),
            };
            (*idx, option_text)
        })
        .collect();

    options.shuffle(rng);

    let correct_index = options
        .iter()
        .position(|(idx, _)| *idx == correct_compound_index)
        .expect("correct option must exist after shuffle");

    let prompt = match mode {
        QuizMode::NameToStructure => compounds[correct_compound_index].display_name(),
        QuizMode::StructureToName => compounds[correct_compound_index].display_structure(),
    };

    Ok(QuizItem {
        mode,
        prompt,
        options: options.into_iter().map(|(_, text)| text).collect(),
        correct_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compound::Compound;
    use rand::SeedableRng;

    fn sample_compounds() -> Vec<Compound> {
        vec![
            Compound {
                iupac_name: "ethanol".to_string(),
                common_name: Some("ethyl alcohol".to_string()),
                local_name: Some("エタノール".to_string()),
                skeletal_formula: "CH3-CH2-OH".to_string(),
                molecular_formula: "C2H6O".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CCO".to_string()),
            },
            Compound {
                iupac_name: "propan-2-ol".to_string(),
                common_name: Some("isopropyl alcohol".to_string()),
                local_name: Some("イソプロパノール".to_string()),
                skeletal_formula: "(CH3)2CHOH".to_string(),
                molecular_formula: "C3H8O".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CC(O)C".to_string()),
            },
            Compound {
                iupac_name: "ethanoic acid".to_string(),
                common_name: Some("acetic acid".to_string()),
                local_name: Some("酢酸".to_string()),
                skeletal_formula: "CH3COOH".to_string(),
                molecular_formula: "C2H4O2".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CC(=O)O".to_string()),
            },
            Compound {
                iupac_name: "benzene".to_string(),
                common_name: None,
                local_name: Some("ベンゼン".to_string()),
                skeletal_formula: "C6H6".to_string(),
                molecular_formula: "C6H6".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("c1ccccc1".to_string()),
            },
        ]
    }

    #[test]
    fn generate_name_to_structure_quiz() {
        let compounds = sample_compounds();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let quiz = generate_quiz(&mut rng, &compounds, QuizMode::NameToStructure, 4)
            .expect("quiz should generate");

        assert_eq!(quiz.mode, QuizMode::NameToStructure);
        assert_eq!(quiz.options.len(), 4);
        assert!(quiz.correct_index < quiz.options.len());

        let correct_compound = compounds
            .iter()
            .find(|compound| compound.display_name() == quiz.prompt)
            .expect("prompt should match a provided compound");

        assert_eq!(
            quiz.options[quiz.correct_index],
            correct_compound.display_structure()
        );
    }

    #[test]
    fn generate_structure_to_name_quiz() {
        let compounds = sample_compounds();
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);

        let quiz = generate_quiz(&mut rng, &compounds, QuizMode::StructureToName, 3)
            .expect("quiz should generate");

        assert_eq!(quiz.mode, QuizMode::StructureToName);
        assert_eq!(quiz.options.len(), 3);
        assert!(quiz.correct_index < quiz.options.len());

        let correct_compound = compounds
            .iter()
            .find(|compound| compound.display_structure() == quiz.prompt)
            .expect("prompt should correspond to a compound");

        assert_eq!(
            quiz.options[quiz.correct_index],
            correct_compound.display_name()
        );
    }

    #[test]
    fn error_when_too_few_compounds() {
        let compounds = sample_compounds();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);

        let error = generate_quiz(&mut rng, &compounds[..2], QuizMode::NameToStructure, 3)
            .expect_err("not enough compounds");

        assert!(matches!(
            error,
            QuizError::NotEnoughCompounds {
                required: 3,
                available: 2
            }
        ));
    }

    #[test]
    fn error_when_not_enough_unique_options() {
        let duplicated = vec![
            Compound {
                iupac_name: "ethanol".to_string(),
                common_name: None,
                local_name: None,
                skeletal_formula: "CH3-CH2-OH".to_string(),
                molecular_formula: "C2H6O".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CCO".to_string()),
            },
            Compound {
                iupac_name: "ethanol".to_string(),
                common_name: None,
                local_name: None,
                skeletal_formula: "CH3-CH2-OH".to_string(),
                molecular_formula: "C2H6O".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CCO".to_string()),
            },
            Compound {
                iupac_name: "propan-1-ol".to_string(),
                common_name: None,
                local_name: None,
                skeletal_formula: "CH3-CH2-CH2-OH".to_string(),
                molecular_formula: "C3H8O".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CCCO".to_string()),
            },
        ];

        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let error = generate_quiz(&mut rng, &duplicated, QuizMode::StructureToName, 3)
            .expect_err("insufficient unique options");

        assert!(matches!(
            error,
            QuizError::InsufficientUniqueOptions {
                required: 3,
                unique: 2
            }
        ));
    }

    #[test]
    fn error_when_option_count_too_small() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(3);
        let compounds = sample_compounds();

        let error = generate_quiz(&mut rng, &compounds, QuizMode::NameToStructure, 1)
            .expect_err("option count too small");

        assert_eq!(error, QuizError::OptionCountTooSmall);
    }
}
