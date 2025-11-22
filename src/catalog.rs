use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::compound::Compound;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, thiserror::Error)]
pub enum CatalogLoadError {
    #[error("category path must contain at least one segment")]
    EmptyCategoryPath,
    #[error("failed to read dataset file at {path}: {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse dataset file at {path}: {source}")]
    ParseError {
        path: String,
        source: serde_json::Error,
    },
}

impl PartialEq for CatalogLoadError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EmptyCategoryPath, Self::EmptyCategoryPath) => true,
            (Self::ReadError { path: left, .. }, Self::ReadError { path: right, .. }) => {
                left == right
            }
            (Self::ParseError { path: left, .. }, Self::ParseError { path: right, .. }) => {
                left == right
            }
            _ => false,
        }
    }
}

impl Eq for CatalogLoadError {}

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

    pub fn from_directory(root: impl AsRef<Path>) -> Result<Self, CatalogLoadError> {
        let mut entries = Vec::new();
        collect_entries(root.as_ref(), Vec::new(), &mut entries)?;

        Ok(Self { entries })
    }

    pub fn all_compounds(&self) -> Vec<Compound> {
        self.entries
            .iter()
            .map(|entry| entry.compound.clone())
            .collect()
    }

    pub fn available_paths(&self) -> BTreeSet<Vec<String>> {
        let mut paths = BTreeSet::new();

        for entry in &self.entries {
            for depth in 1..=entry.categories.len() {
                paths.insert(entry.categories[..depth].to_vec());
            }
        }

        paths
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct CompoundList {
    compounds: Vec<Compound>,
}

fn collect_entries(
    root: &Path,
    categories: Vec<String>,
    entries: &mut Vec<CatalogEntry>,
) -> Result<(), CatalogLoadError> {
    let walker = fs::read_dir(root).map_err(|source| CatalogLoadError::ReadError {
        path: root.display().to_string(),
        source,
    })?;

    for entry in walker {
        let entry = entry.map_err(|source| CatalogLoadError::ReadError {
            path: root.display().to_string(),
            source,
        })?;

        let path = entry.path();

        if path.is_dir() {
            let mut next_categories = categories.clone();
            if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
                next_categories.push(name.to_string());
            }
            collect_entries(&path, next_categories, entries)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            append_from_file(&path, &categories, entries)?;
        }
    }

    Ok(())
}

fn append_from_file(
    path: &Path,
    categories: &[String],
    entries: &mut Vec<CatalogEntry>,
) -> Result<(), CatalogLoadError> {
    if categories.is_empty() {
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|name| name == "index.json")
            .unwrap_or(false)
        {
            return Ok(());
        }

        return Err(CatalogLoadError::EmptyCategoryPath);
    }

    let data = fs::read(path).map_err(|source| CatalogLoadError::ReadError {
        path: path.display().to_string(),
        source,
    })?;

    let parsed: CompoundList =
        serde_json::from_slice(&data).map_err(|source| CatalogLoadError::ParseError {
            path: path.display().to_string(),
            source,
        })?;

    for compound in parsed.compounds {
        entries.push(CatalogEntry {
            compound,
            categories: categories.to_vec(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn sample_catalog() -> Catalog {
        Catalog::new(vec![
            CatalogEntry {
                compound: Compound {
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
                    series_general_formula: None,
                    functional_groups: Vec::new(),
                    notes: None,
                    smiles: Some("Cl[Na]".to_string()),
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

    #[test]
    fn loads_entries_from_directory_tree() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let organic_alcohols = root.path().join("Organic").join("Alcohols");
        fs::create_dir_all(&organic_alcohols).expect("directory tree should be created");

        let inorganic_salts = root.path().join("Inorganic").join("Salts");
        fs::create_dir_all(&inorganic_salts).expect("directory tree should be created");

        write_compound_list(
            organic_alcohols.join("compounds.json"),
            vec![Compound {
                iupac_name: "ethanol".to_string(),
                common_name: Some("ethyl alcohol".to_string()),
                local_name: Some("エタノール".to_string()),
                skeletal_formula: "CH3-CH2-OH".to_string(),
                molecular_formula: "C2H6O".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("CCO".to_string()),
            }],
        );

        write_compound_list(
            inorganic_salts.join("compounds.json"),
            vec![Compound {
                iupac_name: "sodium chloride".to_string(),
                common_name: Some("table salt".to_string()),
                local_name: Some("塩化ナトリウム".to_string()),
                skeletal_formula: "NaCl".to_string(),
                molecular_formula: "NaCl".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("Cl[Na]".to_string()),
            }],
        );

        let catalog = Catalog::from_directory(root.path()).expect("catalog should load");
        let paths = catalog.available_paths();

        assert!(paths.contains(&vec!["Organic".to_string(), "Alcohols".to_string()]));
        assert!(paths.contains(&vec!["Inorganic".to_string(), "Salts".to_string()]));
        assert_eq!(catalog.all_compounds().len(), 2);
    }

    #[test]
    fn errors_when_category_path_missing() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = root.path().join("compounds.json");
        write_compound_list(
            &path,
            vec![Compound {
                iupac_name: "methane".to_string(),
                common_name: Some("marsh gas".to_string()),
                local_name: Some("メタン".to_string()),
                skeletal_formula: "CH4".to_string(),
                molecular_formula: "CH4".to_string(),
                series_general_formula: None,
                functional_groups: Vec::new(),
                notes: None,
                smiles: Some("C".to_string()),
            }],
        );

        let error =
            Catalog::from_directory(root.path()).expect_err("missing category should error");
        assert_eq!(error, CatalogLoadError::EmptyCategoryPath);
    }

    fn write_compound_list(path: impl Into<PathBuf>, compounds: Vec<Compound>) {
        let payload = CompoundList { compounds };
        let serialized =
            serde_json::to_string(&payload).expect("compound list should serialize to JSON");
        fs::write(path.into(), serialized).expect("compound list should be written");
    }
}
