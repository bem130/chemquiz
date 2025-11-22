#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CatalogManifest {
    pub roots: Vec<CatalogNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CatalogNode {
    pub label: String,
    pub slug: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub children: Vec<CatalogNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CatalogLeaf {
    pub path: Vec<String>,
    pub file: String,
}

impl CatalogManifest {
    pub fn leaves(&self) -> Vec<CatalogLeaf> {
        let mut leaves = Vec::new();

        for node in &self.roots {
            gather_leaves(node, Vec::new(), &mut leaves);
        }

        leaves
    }
}

fn gather_leaves(node: &CatalogNode, mut prefix: Vec<String>, leaves: &mut Vec<CatalogLeaf>) {
    prefix.push(node.label.clone());

    if let Some(file) = &node.file {
        leaves.push(CatalogLeaf {
            path: prefix.clone(),
            file: file.clone(),
        });
    }

    for child in &node.children {
        gather_leaves(child, prefix.clone(), leaves);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_manifest_into_leaves() {
        let manifest = CatalogManifest {
            roots: vec![CatalogNode {
                label: "Organic".to_string(),
                slug: "Organic".to_string(),
                file: Some("catalog/Organic/compounds.json".to_string()),
                children: vec![CatalogNode {
                    label: "Aliphatic compounds".to_string(),
                    slug: "Aliphatic_compounds".to_string(),
                    file: Some("catalog/Organic/Aliphatic_compounds/compounds.json".to_string()),
                    children: vec![CatalogNode {
                        label: "Alcohols and ethers".to_string(),
                        slug: "Alcohols_and_ethers".to_string(),
                        file: Some(
                            "catalog/Organic/Aliphatic_compounds/Alcohols_and_ethers/compounds.json"
                                .to_string(),
                        ),
                        children: vec![CatalogNode {
                            label: "Primary alcohols".to_string(),
                            slug: "Primary_alcohols".to_string(),
                            file: Some(
                                "catalog/Organic/Aliphatic_compounds/Alcohols_and_ethers/Primary_alcohols/compounds.json"
                                    .to_string(),
                            ),
                            children: vec![],
                        }],
                    }],
                }],
            }],
        };

        let leaves = manifest.leaves();
        assert_eq!(leaves.len(), 4);
        assert!(leaves.contains(&CatalogLeaf {
            path: vec!["Organic".to_string()],
            file: "catalog/Organic/compounds.json".to_string(),
        }));
        assert!(leaves.contains(&CatalogLeaf {
            path: vec!["Organic".to_string(), "Aliphatic compounds".to_string()],
            file: "catalog/Organic/Aliphatic_compounds/compounds.json".to_string(),
        }));
        assert!(leaves.contains(&CatalogLeaf {
            path: vec![
                "Organic".to_string(),
                "Aliphatic compounds".to_string(),
                "Alcohols and ethers".to_string(),
            ],
            file: "catalog/Organic/Aliphatic_compounds/Alcohols_and_ethers/compounds.json"
                .to_string(),
        }));
        assert!(leaves.contains(&CatalogLeaf {
            path: vec![
                "Organic".to_string(),
                "Aliphatic compounds".to_string(),
                "Alcohols and ethers".to_string(),
                "Primary alcohols".to_string(),
            ],
            file: "catalog/Organic/Aliphatic_compounds/Alcohols_and_ethers/Primary_alcohols/compounds.json"
                .to_string(),
        }));
    }

    #[test]
    fn serializes_and_deserializes_manifest() {
        let manifest = CatalogManifest {
            roots: vec![CatalogNode {
                label: "Inorganic".to_string(),
                slug: "inorganic".to_string(),
                file: Some("catalog/Inorganic/compounds.json".to_string()),
                children: vec![],
            }],
        };

        let json = serde_json::to_string(&manifest).expect("manifest should serialize");
        let decoded: CatalogManifest =
            serde_json::from_str(&json).expect("manifest should round-trip through JSON");

        assert_eq!(decoded, manifest);
    }
}
