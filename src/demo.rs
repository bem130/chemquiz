use crate::{Catalog, CatalogEntry, Compound};

#[cfg(test)]
use crate::{QuizMode, generate_quiz};
#[cfg(test)]
use rand::SeedableRng;

pub const DEMO_OPTION_COUNT: usize = 4;

fn build_compound(
    iupac_name: &str,
    common_name: Option<&str>,
    local_name: Option<&str>,
    skeletal_formula: &str,
    molecular_formula: &str,
    smiles: Option<&str>,
) -> Compound {
    Compound {
        iupac_name: iupac_name.to_string(),
        common_name: common_name.map(str::to_string),
        local_name: local_name.map(str::to_string),
        skeletal_formula: skeletal_formula.to_string(),
        molecular_formula: molecular_formula.to_string(),
        series_general_formula: None,
        functional_groups: Vec::new(),
        notes: None,
        smiles: smiles.map(str::to_string),
    }
}

fn demo_entries() -> Vec<(Compound, Vec<String>)> {
    vec![
        (
            build_compound(
                "methanol",
                Some("methyl alcohol"),
                Some("メタノール"),
                "CH3OH",
                "CH4O",
                Some("CO"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Alcohols_and_ethers".to_string(),
                "Primary_alcohols".to_string(),
            ],
        ),
        (
            build_compound(
                "ethanol",
                Some("ethyl alcohol"),
                Some("エタノール"),
                "CH3-CH2-OH",
                "C2H6O",
                Some("CCO"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Alcohols_and_ethers".to_string(),
                "Primary_alcohols".to_string(),
            ],
        ),
        (
            build_compound(
                "propan-2-ol",
                Some("isopropyl alcohol"),
                Some("イソプロパノール"),
                "(CH3)2CHOH",
                "C3H8O",
                Some("CC(O)C"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Alcohols_and_ethers".to_string(),
                "Secondary_alcohols".to_string(),
            ],
        ),
        (
            build_compound(
                "ethanoic acid",
                Some("acetic acid"),
                Some("酢酸"),
                "CH3COOH",
                "C2H4O2",
                Some("CC(=O)O"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Carboxylic_acids_and_esters".to_string(),
                "Carboxylic_acids".to_string(),
            ],
        ),
        (
            build_compound(
                "propanoic acid",
                Some("propionic acid"),
                Some("プロピオン酸"),
                "CH3-CH2-COOH",
                "C3H6O2",
                Some("CCC(=O)O"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Carboxylic_acids_and_esters".to_string(),
                "Carboxylic_acids".to_string(),
            ],
        ),
        (
            build_compound(
                "benzene",
                None,
                Some("ベンゼン"),
                "C6H6",
                "C6H6",
                Some("c1ccccc1"),
            ),
            vec![
                "Organic".to_string(),
                "Aromatic_compounds".to_string(),
                "Aromatic_hydrocarbons".to_string(),
            ],
        ),
        (
            build_compound(
                "methylbenzene",
                Some("toluene"),
                Some("トルエン"),
                "C6H5-CH3",
                "C7H8",
                Some("Cc1ccccc1"),
            ),
            vec![
                "Organic".to_string(),
                "Aromatic_compounds".to_string(),
                "Aromatic_hydrocarbons".to_string(),
            ],
        ),
        (
            build_compound(
                "ethyne",
                Some("acetylene"),
                Some("アセチレン"),
                "HC≡CH",
                "C2H2",
                Some("C#C"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Hydrocarbons".to_string(),
                "Alkynes".to_string(),
            ],
        ),
        (
            build_compound(
                "but-2-yne",
                Some("dimethylacetylene"),
                Some("2-ブチン"),
                "CH3-C≡C-CH3",
                "C4H6",
                Some("CC#CC"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Hydrocarbons".to_string(),
                "Alkynes".to_string(),
            ],
        ),
        (
            build_compound(
                "2-methylpropane",
                Some("isobutane"),
                Some("イソブタン"),
                "(CH3)2CH-CH3",
                "C4H10",
                Some("CC(C)C"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Hydrocarbons".to_string(),
                "Alkanes".to_string(),
            ],
        ),
        (
            build_compound(
                "hexane",
                None,
                Some("ヘキサン"),
                "CH3-(CH2)4-CH3",
                "C6H14",
                Some("CCCCCC"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Hydrocarbons".to_string(),
                "Alkanes".to_string(),
            ],
        ),
        (
            build_compound(
                "propane-1,2,3-triol",
                Some("glycerol"),
                Some("グリセリン"),
                "HO-CH2-CH(OH)-CH2-OH",
                "C3H8O3",
                Some("OCC(O)CO"),
            ),
            vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Alcohols_and_ethers".to_string(),
                "Polyols".to_string(),
            ],
        ),
        (
            build_compound(
                "sodium chloride",
                Some("table salt"),
                Some("塩化ナトリウム"),
                "NaCl",
                "NaCl",
                Some("Cl[Na]"),
            ),
            vec![
                "Inorganic".to_string(),
                "Salts".to_string(),
                "Halides".to_string(),
            ],
        ),
        (
            build_compound(
                "calcium carbonate",
                Some("calcite"),
                Some("炭酸カルシウム"),
                "CaCO3",
                "CaCO3",
                Some("[Ca+2].[O-]C(=O)[O-]"),
            ),
            vec![
                "Inorganic".to_string(),
                "Salts".to_string(),
                "Carbonates".to_string(),
            ],
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
            "Aliphatic_compounds".to_string(),
            "Alcohols_and_ethers".to_string(),
            "Primary_alcohols".to_string(),
        ]));
        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Aliphatic_compounds".to_string(),
            "Alcohols_and_ethers".to_string(),
            "Secondary_alcohols".to_string(),
        ]));
        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Aliphatic_compounds".to_string(),
            "Alcohols_and_ethers".to_string(),
            "Polyols".to_string(),
        ]));
        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Aliphatic_compounds".to_string(),
            "Carboxylic_acids_and_esters".to_string(),
            "Carboxylic_acids".to_string(),
        ]));
    }

    #[test]
    fn catalog_filters_by_prefix() {
        let catalog = demo_catalog();
        let alcohols = catalog
            .compounds_for(&vec![
                "Organic".to_string(),
                "Aliphatic_compounds".to_string(),
                "Alcohols_and_ethers".to_string(),
            ])
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
