use crate::{Catalog, CatalogEntry, Compound};

#[cfg(test)]
use crate::{generate_quiz, QuizMode};
#[cfg(test)]
use rand::SeedableRng;

pub const DEMO_OPTION_COUNT: usize = 4;

fn demo_entries() -> Vec<(Compound, Vec<String>)> {
    vec![
        (
            Compound {
                iupac_name: "ethanol".to_string(),
                common_name: Some("ethyl alcohol".to_string()),
                local_name: Some("エタノール".to_string()),
                skeletal_formula: "CH3-CH2-OH".to_string(),
                molecular_formula: "C2H6O".to_string(),
            },
            vec![
                "Organic".to_string(),
                "Alcohols".to_string(),
                "Primary Alcohols".to_string(),
            ],
        ),
        (
            Compound {
                iupac_name: "propan-2-ol".to_string(),
                common_name: Some("isopropyl alcohol".to_string()),
                local_name: Some("イソプロパノール".to_string()),
                skeletal_formula: "(CH3)2CHOH".to_string(),
                molecular_formula: "C3H8O".to_string(),
            },
            vec![
                "Organic".to_string(),
                "Alcohols".to_string(),
                "Secondary Alcohols".to_string(),
            ],
        ),
        (
            Compound {
                iupac_name: "ethanoic acid".to_string(),
                common_name: Some("acetic acid".to_string()),
                local_name: Some("酢酸".to_string()),
                skeletal_formula: "CH3COOH".to_string(),
                molecular_formula: "C2H4O2".to_string(),
            },
            vec!["Organic".to_string(), "Carboxylic acids".to_string()],
        ),
        (
            Compound {
                iupac_name: "benzene".to_string(),
                common_name: None,
                local_name: Some("ベンゼン".to_string()),
                skeletal_formula: "C6H6".to_string(),
                molecular_formula: "C6H6".to_string(),
            },
            vec!["Organic".to_string(), "Arenes".to_string()],
        ),
        (
            Compound {
                iupac_name: "ethyne".to_string(),
                common_name: Some("acetylene".to_string()),
                local_name: Some("アセチレン".to_string()),
                skeletal_formula: "HC≡CH".to_string(),
                molecular_formula: "C2H2".to_string(),
            },
            vec!["Organic".to_string(), "Alkynes".to_string()],
        ),
        (
            Compound {
                iupac_name: "2-methylpropane".to_string(),
                common_name: Some("isobutane".to_string()),
                local_name: Some("イソブタン".to_string()),
                skeletal_formula: "(CH3)2CH-CH3".to_string(),
                molecular_formula: "C4H10".to_string(),
            },
            vec!["Organic".to_string(), "Alkanes".to_string()],
        ),
    ]
}

pub fn demo_compounds() -> Vec<Compound> {
    demo_entries()
        .into_iter()
        .map(|(compound, _)| compound)
        .collect()
}

pub fn demo_catalog() -> Catalog {
    Catalog::new(
        demo_entries()
            .into_iter()
            .map(|(compound, categories)| CatalogEntry {
                compound,
                categories,
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;

    #[test]
    fn dataset_supports_name_to_structure() {
        let mut rng = StdRng::seed_from_u64(10);
        let quiz = generate_quiz(
            &mut rng,
            &demo_compounds(),
            QuizMode::NameToStructure,
            DEMO_OPTION_COUNT,
        )
        .expect("dataset should generate name to structure quiz");

        assert_eq!(quiz.options.len(), DEMO_OPTION_COUNT);
    }

    #[test]
    fn dataset_supports_structure_to_name() {
        let mut rng = StdRng::seed_from_u64(15);
        let quiz = generate_quiz(
            &mut rng,
            &demo_compounds(),
            QuizMode::StructureToName,
            DEMO_OPTION_COUNT,
        )
        .expect("dataset should generate structure to name quiz");

        assert_eq!(quiz.options.len(), DEMO_OPTION_COUNT);
    }

    #[test]
    fn catalog_exposes_expected_paths() {
        let catalog = demo_catalog();
        let paths = catalog.available_paths();

        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Alcohols".to_string(),
            "Primary Alcohols".to_string(),
        ]));
        assert!(paths.contains(&vec!["Organic".to_string(), "Carboxylic acids".to_string()]));
    }

    #[test]
    fn catalog_filters_by_prefix() {
        let catalog = demo_catalog();
        let alcohols = catalog
            .compounds_for(&vec!["Organic".to_string(), "Alcohols".to_string()])
            .expect("alcohol category exists");

        assert_eq!(alcohols.len(), 2);
        assert!(
            alcohols
                .iter()
                .any(|compound| compound.iupac_name == "ethanol")
        );
        assert!(
            alcohols
                .iter()
                .any(|compound| compound.iupac_name == "propan-2-ol")
        );
    }
}
