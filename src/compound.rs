use std::fmt;

/// Functional group metadata that appears within compound definitions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FunctionalGroup {
    /// English label for the functional group.
    pub name_en: String,
    /// Japanese label for the functional group.
    pub name_ja: String,
    /// Pattern or formula describing the group.
    pub pattern: String,
}

/// Represents a chemical compound used for quiz questions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Compound {
    /// IUPAC name of the compound.
    pub iupac_name: String,
    /// Common or trivial name if one exists.
    pub common_name: Option<String>,
    /// Name commonly used in Japan.
    pub local_name: Option<String>,
    /// Skeletal structure formula shown as the primary structural representation.
    pub skeletal_formula: String,
    /// Molecular formula shown as a compact representation.
    pub molecular_formula: String,
    /// Generalized formula shared across a series, when applicable.
    #[serde(default)]
    pub series_general_formula: Option<String>,
    /// Functional groups that appear in the compound.
    #[serde(default)]
    pub functional_groups: Vec<FunctionalGroup>,
    /// Free-form notes about properties or preparation.
    #[serde(default)]
    pub notes: Option<String>,
    /// SMILES string used for structure rendering when available.
    #[serde(default)]
    pub smiles: Option<String>,
}

impl Compound {
    /// Returns an English display label that prefers the IUPAC name
    /// and appends the common name in parentheses when available and
    /// distinct.
    pub fn english_label(&self) -> String {
        match &self.common_name {
            Some(common) if common != &self.iupac_name => {
                format!("{} ({})", self.iupac_name, common)
            }
            _ => self.iupac_name.clone(),
        }
    }

    pub fn display_name(&self) -> String {
        let mut parts = vec![self.english_label()];

        if let Some(local) = &self.local_name {
            parts.push(format!("/ {}", local));
        }

        parts.join(" ")
    }

    pub fn display_structure(&self) -> String {
        format!("{} ({})", self.skeletal_formula, self.molecular_formula)
    }
}

impl fmt::Display for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.display_name(), self.display_structure())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ethanol() -> Compound {
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
        }
    }

    #[test]
    fn english_label_prefers_common_name() {
        let label = ethanol().english_label();
        assert_eq!(label, "ethanol (ethyl alcohol)");
    }

    #[test]
    fn english_label_avoids_duplicate_common_name() {
        let compound = Compound {
            iupac_name: "benzene".to_string(),
            common_name: Some("benzene".to_string()),
            local_name: None,
            skeletal_formula: "C6H6".to_string(),
            molecular_formula: "C6H6".to_string(),
            series_general_formula: None,
            functional_groups: Vec::new(),
            notes: None,
            smiles: Some("c1ccccc1".to_string()),
        };

        assert_eq!(compound.english_label(), "benzene");
    }

    #[test]
    fn display_name_prefers_all_available_labels() {
        let name = ethanol().display_name();
        assert_eq!(name, "ethanol (ethyl alcohol) / エタノール");
    }

    #[test]
    fn display_name_skips_missing_fields() {
        let compound = Compound {
            iupac_name: "benzene".to_string(),
            common_name: None,
            local_name: Some("ベンゼン".to_string()),
            skeletal_formula: "C6H6".to_string(),
            molecular_formula: "C6H6".to_string(),
            series_general_formula: None,
            functional_groups: Vec::new(),
            notes: None,
            smiles: Some("c1ccccc1".to_string()),
        };

        let name = compound.display_name();
        assert_eq!(name, "benzene / ベンゼン");
    }

    #[test]
    fn display_structure_combines_forms() {
        let ethanol = ethanol();
        let structure = ethanol.display_structure();
        assert_eq!(structure, "CH3-CH2-OH (C2H6O)");
    }

    #[test]
    fn display_trait_includes_name_and_structure() {
        let compound = ethanol();
        let formatted = format!("{}", compound);

        assert!(formatted.contains("ethanol"));
        assert!(formatted.contains("CH3-CH2-OH (C2H6O)"));
    }

    #[test]
    fn smiles_defaults_to_none_on_missing_field() {
        let json = r#"{
            "iupac_name": "acetone",
            "common_name": "propanone",
            "local_name": null,
            "skeletal_formula": "(CH3)2CO",
            "molecular_formula": "C3H6O"
        }"#;

        let parsed: Compound =
            serde_json::from_str(json).expect("compound should parse without smiles");

        assert!(parsed.smiles.is_none());
    }

    #[test]
    fn smiles_is_preserved_when_present() {
        let json = r#"{
            "iupac_name": "acetic acid",
            "common_name": null,
            "local_name": "酢酸",
            "skeletal_formula": "CH3COOH",
            "molecular_formula": "C2H4O2",
            "functional_groups": [],
            "smiles": "CC(=O)O"
        }"#;

        let parsed: Compound =
            serde_json::from_str(json).expect("compound should parse with smiles");

        assert_eq!(parsed.smiles.as_deref(), Some("CC(=O)O"));
    }

    #[test]
    fn functional_groups_are_parsed() {
        let json = r#"{
            "iupac_name": "ammonium hydrogencarbonate",
            "common_name": "ammonium bicarbonate",
            "local_name": "炭酸水素アンモニウム",
            "skeletal_formula": "(NH4)HCO3",
            "molecular_formula": "NH4HCO3",
            "functional_groups": [
                { "name_en": "Ammonium ion", "name_ja": "アンモニウムイオン", "pattern": "NH4^+" },
                { "name_en": "Hydrogen carbonate ion", "name_ja": "炭酸水素イオン", "pattern": "HCO3^-" }
            ],
            "notes": "Unstable solid used in leavening mixtures"
        }"#;

        let parsed: Compound =
            serde_json::from_str(json).expect("compound should parse with groups");

        assert_eq!(parsed.functional_groups.len(), 2);
        assert_eq!(parsed.functional_groups[0].name_en, "Ammonium ion");
        assert_eq!(parsed.functional_groups[1].pattern, "HCO3^-");
        assert_eq!(
            parsed.notes.as_deref(),
            Some("Unstable solid used in leavening mixtures")
        );
    }

    #[test]
    fn optional_fields_default_to_none() {
        let json = r#"{
            "iupac_name": "sodium carbonate",
            "common_name": "soda ash",
            "local_name": "炭酸ナトリウム",
            "skeletal_formula": "Na2CO3",
            "molecular_formula": "Na2CO3"
        }"#;

        let parsed: Compound = serde_json::from_str(json).expect("compound should fill defaults");

        assert!(parsed.functional_groups.is_empty());
        assert!(parsed.series_general_formula.is_none());
        assert!(parsed.notes.is_none());
        assert!(parsed.smiles.is_none());
    }
}
