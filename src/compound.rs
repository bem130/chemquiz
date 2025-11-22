use std::fmt;

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
    /// SMILES string used for structure rendering when available.
    #[serde(default)]
    pub smiles: Option<String>,
}

impl Compound {
    pub fn display_name(&self) -> String {
        let mut parts = vec![self.iupac_name.clone()];

        if let Some(common) = &self.common_name {
            parts.push(format!("({})", common));
        }

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
            smiles: Some("CCO".to_string()),
        }
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
            "smiles": "CC(=O)O"
        }"#;

        let parsed: Compound =
            serde_json::from_str(json).expect("compound should parse with smiles");

        assert_eq!(parsed.smiles.as_deref(), Some("CC(=O)O"));
    }
}
