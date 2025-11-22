use std::collections::BTreeSet;

use crate::compound::Compound;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub compound: Compound,
    pub categories: Vec<String>,
}

impl CatalogEntry {
    pub fn category_path(&self) -> String {
        self.categories.join(" / ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    entries: Vec<CatalogEntry>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CatalogError {
    #[error("category path must contain at least one segment")]
    EmptyPath,
    #[error("no compounds found for category path: {path}")]
    CategoryNotFound { path: String },
}

impl Catalog {
    pub fn new(entries: Vec<CatalogEntry>) -> Self {
        Self { entries }
    }

    pub fn all_compounds(&self) -> Vec<Compound> {
        self.entries
            .iter()
            .map(|entry| entry.compound.clone())
            .collect()
    }

    pub fn available_paths(&self) -> BTreeSet<Vec<String>> {
        self.entries
            .iter()
            .map(|entry| entry.categories.clone())
            .collect()
    }

    pub fn compounds_for(&self, path: &[String]) -> Result<Vec<Compound>, CatalogError> {
        if path.is_empty() {
            return Err(CatalogError::EmptyPath);
        }

        let matches: Vec<Compound> = self
            .entries
            .iter()
            .filter(|entry| entry.categories.starts_with(path))
            .map(|entry| entry.compound.clone())
            .collect();

        if matches.is_empty() {
            return Err(CatalogError::CategoryNotFound {
                path: path.join(" / "),
            });
        }

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalog() -> Catalog {
        Catalog::new(vec![
            CatalogEntry {
                compound: Compound {
                    iupac_name: "ethanol".to_string(),
                    common_name: Some("ethyl alcohol".to_string()),
                    local_name: Some("エタノール".to_string()),
                    skeletal_formula: "CH3-CH2-OH".to_string(),
                    molecular_formula: "C2H6O".to_string(),
                },
                categories: vec![
                    "Organic".to_string(),
                    "Alcohols".to_string(),
                    "Primary Alcohols".to_string(),
                ],
            },
            CatalogEntry {
                compound: Compound {
                    iupac_name: "sodium chloride".to_string(),
                    common_name: Some("table salt".to_string()),
                    local_name: Some("塩化ナトリウム".to_string()),
                    skeletal_formula: "NaCl".to_string(),
                    molecular_formula: "NaCl".to_string(),
                },
                categories: vec!["Inorganic".to_string(), "Salts".to_string()],
            },
        ])
    }

    #[test]
    fn lists_available_paths() {
        let catalog = sample_catalog();
        let paths = catalog.available_paths();

        assert!(paths.contains(&vec![
            "Organic".to_string(),
            "Alcohols".to_string(),
            "Primary Alcohols".to_string(),
        ]));
        assert!(paths.contains(&vec!["Inorganic".to_string(), "Salts".to_string()]));
    }

    #[test]
    fn returns_compounds_for_category_prefix() {
        let catalog = sample_catalog();
        let compounds = catalog
            .compounds_for(&vec!["Organic".to_string(), "Alcohols".to_string()])
            .expect("category exists");

        assert_eq!(compounds.len(), 1);
        assert_eq!(compounds[0].iupac_name, "ethanol");
    }

    #[test]
    fn errors_on_unknown_path() {
        let catalog = sample_catalog();
        let error = catalog
            .compounds_for(&vec!["Nonexistent".to_string()])
            .expect_err("missing category should error");

        assert_eq!(
            error,
            CatalogError::CategoryNotFound {
                path: "Nonexistent".to_string()
            }
        );
    }

    #[test]
    fn errors_on_empty_path() {
        let catalog = sample_catalog();
        let error = catalog
            .compounds_for(&Vec::new())
            .expect_err("empty selection should error");

        assert_eq!(error, CatalogError::EmptyPath);
    }
}
