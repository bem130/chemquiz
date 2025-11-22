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
                iupac_name: "methanol".to_string(),
                common_name: Some("methyl alcohol".to_string()),
                local_name: Some("メタノール".to_string()),
                skeletal_formula: "CH3OH".to_string(),
                molecular_formula: "CH4O".to_string(),
            },
            vec![
                "Organic".to_string(),
                "Alcohols".to_string(),
                "Primary alcohols".to_string(),
            ],
        ),
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
                "Primary alcohols".to_string(),
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
                "Secondary alcohols".to_string(),
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
                iupac_name: "propanoic acid".to_string(),
                common_name: Some("propionic acid".to_string()),
                local_name: Some("プロピオン酸".to_string()),
                skeletal_formula: "CH3-CH2-COOH".to_string(),
                molecular_formula: "C3H6O2".to_string(),
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
                iupac_name: "methylbenzene".to_string(),
                common_name: Some("toluene".to_string()),
                local_name: Some("トルエン".to_string()),
                skeletal_formula: "C6H5-CH3".to_string(),
                molecular_formula: "C7H8".to_string(),
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
                iupac_name: "but-2-yne".to_string(),
                common_name: Some("dimethylacetylene".to_string()),
                local_name: Some("2-ブチン".to_string()),
                skeletal_formula: "CH3-C≡C-CH3".to_string(),
                molecular_formula: "C4H6".to_string(),
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
        (
            Compound {
                iupac_name: "hexane".to_string(),
                common_name: None,
                local_name: Some("ヘキサン".to_string()),
                skeletal_formula: "CH3-(CH2)4-CH3".to_string(),
                molecular_formula: "C6H14".to_string(),
            },
            vec!["Organic".to_string(), "Alkanes".to_string()],
        ),
        (
            Compound {
                iupac_name: "propane-1,2,3-triol".to_string(),
                common_name: Some("glycerol".to_string()),
                local_name: Some("グリセリン".to_string()),
                skeletal_formula: "HO-CH2-CH(OH)-CH2-OH".to_string(),
                molecular_formula: "C3H8O3".to_string(),
            },
            vec![
                "Organic".to_string(),
                "Alcohols".to_string(),
                "Polyols".to_string(),
            ],
        ),
        (
            Compound {
                iupac_name: "sodium chloride".to_string(),
                common_name: Some("table salt".to_string()),
                local_name: Some("塩化ナトリウム".to_string()),
                skeletal_formula: "NaCl".to_string(),
                molecular_formula: "NaCl".to_string(),
            },
            vec!["Inorganic".to_string(), "Salts".to_string(), "Halides".to_string()],
        ),
        (
            Compound {
                iupac_name: "calcium carbonate".to_string(),
                common_name: Some("calcite".to_string()),
                local_name: Some("炭酸カルシウム".to_string()),
                skeletal_formula: "CaCO3".to_string(),
                molecular_formula: "CaCO3".to_string(),
            },
            vec!["Inorganic".to_string(), "Salts".to_string(), "Carbonates".to_string()],
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
            "Primary alcohols".to_string(),
        ]));
        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Alcohols".to_string(),
            "Secondary alcohols".to_string(),
        ]));
        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Alcohols".to_string(),
            "Polyols".to_string(),
        ]));
        assert!(paths.contains(&vec!["Organic".to_string(), "Carboxylic acids".to_string()]));
    }

    #[test]
    fn catalog_filters_by_prefix() {
        let catalog = demo_catalog();
        let alcohols = catalog
            .compounds_for(&vec!["Organic".to_string(), "Alcohols".to_string()])
            .expect("alcohol category exists");

        assert_eq!(alcohols.len(), 4);
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
        assert!(
            alcohols
                .iter()
                .any(|compound| compound.iupac_name == "propane-1,2,3-triol")
        );
    }
}
