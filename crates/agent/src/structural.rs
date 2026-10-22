use std::collections::HashMap;

use bnpm_manifest::schema::AiBlock;

use crate::ToolInterfaceError;
use crate::tool_interface::{ToolDefinition, ToolFingerprint, ToolInterfaceFile};

/// A fingerprint of a package's API shape for structural search.
#[derive(Debug, Clone)]
pub struct ApiShapeFingerprint {
    pub package_name: String,
    pub version: String,
    pub intent: Option<String>,
    pub capabilities_declared: Vec<String>,
    pub side_effects: Option<String>,
    pub tool_fingerprint: Option<ToolFingerprint>,
    pub export_names_hash: String,
}

impl ApiShapeFingerprint {
    pub fn from_parts(
        package_name: String,
        version: String,
        ai: Option<&AiBlock>,
        tools: Option<&[ToolDefinition]>,
        exports: &[String],
    ) -> Self {
        let tool_fingerprint = tools.map(ToolFingerprint::from_interface);

        let mut export_names = exports.to_vec();
        export_names.sort();
        let export_names_hash = export_names.join(",");

        Self {
            package_name,
            version,
            intent: ai.and_then(|a| a.intent.clone()),
            capabilities_declared: ai
                .map(|a| a.capabilities_declared.clone().unwrap_or_default())
                .unwrap_or_default(),
            side_effects: ai
                .and_then(|a| a.side_effects.map(|s| format!("{:?}", s)).map(|s| s.to_lowercase())),
            tool_fingerprint,
            export_names_hash,
        }
    }
}

/// Structural indexer: indexes API shape fingerprints for structural search.
#[derive(Debug, Default)]
pub struct StructuralIndexer {
    fingerprints: HashMap<String, ApiShapeFingerprint>,
}

impl StructuralIndexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index_package(
        &mut self,
        package_name: String,
        version: String,
        ai: Option<&AiBlock>,
        tools_content: Option<&str>,
        exports: &[String],
    ) -> Result<(), ToolInterfaceError> {
        let tools = if let Some(content) = tools_content {
            let file = ToolInterfaceFile::from_json_str(content)?;
            Some(file.tools)
        } else {
            None
        };

        let fingerprint = ApiShapeFingerprint::from_parts(
            package_name.clone(),
            version,
            ai,
            tools.as_deref(),
            exports,
        );

        self.fingerprints.insert(package_name, fingerprint);
        Ok(())
    }

    pub fn index_package_from_tools(
        &mut self,
        package_name: String,
        version: String,
        ai: Option<&AiBlock>,
        tools: Option<Vec<ToolDefinition>>,
        exports: &[String],
    ) {
        let fingerprint = ApiShapeFingerprint::from_parts(
            package_name.clone(),
            version,
            ai,
            tools.as_deref(),
            exports,
        );

        self.fingerprints.insert(package_name, fingerprint);
    }

    pub fn search_by_api_shape(&self, shape_query: &ApiShapeQuery) -> Vec<String> {
        let mut results: Vec<(String, f64)> = Vec::new();

        for (name, fp) in &self.fingerprints {
            let score = shape_query.score(fp);
            if score > 0.0 {
                results.push((name.clone(), score));
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(name, _)| name).collect()
    }

    pub fn get_fingerprint(&self, package_name: &str) -> Option<&ApiShapeFingerprint> {
        self.fingerprints.get(package_name)
    }

    pub fn all_packages(&self) -> impl Iterator<Item = &String> {
        self.fingerprints.keys()
    }

    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }
}

/// A query for structural API shape search.
#[derive(Debug, Clone)]
pub struct ApiShapeQuery {
    pub intent: Option<String>,
    pub tool_name: Option<String>,
    pub has_tools: bool,
    pub export_name: Option<String>,
    pub side_effects: Option<String>,
    pub capability: Option<String>,
}

impl ApiShapeQuery {
    pub fn new() -> Self {
        Self {
            intent: None,
            tool_name: None,
            has_tools: false,
            export_name: None,
            side_effects: None,
            capability: None,
        }
    }

    pub fn with_intent(mut self, intent: String) -> Self {
        self.intent = Some(intent);
        self
    }

    pub fn with_tool_name(mut self, tool_name: String) -> Self {
        self.tool_name = Some(tool_name);
        self
    }

    pub fn with_has_tools(mut self) -> Self {
        self.has_tools = true;
        self
    }

    pub fn with_export_name(mut self, export_name: String) -> Self {
        self.export_name = Some(export_name);
        self
    }

    pub fn with_side_effects(mut self, side_effects: String) -> Self {
        self.side_effects = Some(side_effects);
        self
    }

    pub fn with_capability(mut self, capability: String) -> Self {
        self.capability = Some(capability);
        self
    }

    /// Score a fingerprint against this query (0.0 = no match, higher = better match).
    pub fn score(&self, fp: &ApiShapeFingerprint) -> f64 {
        let mut score = 0.0;
        let mut criteria = 0;

        if let Some(ref intent) = self.intent {
            criteria += 1;
            if fp.intent.as_deref().is_some_and(|i| i == intent) {
                score += 1.0;
            } else if fp.intent.as_deref().is_some_and(|i| i.contains(intent)) {
                score += 0.5;
            }
        }

        if self.has_tools {
            criteria += 1;
            if fp.tool_fingerprint.is_some() {
                score += 1.0;
            }
        }

        if let Some(ref tool_name) = self.tool_name {
            criteria += 1;
            if let Some(ref tf) = fp.tool_fingerprint {
                if tf.tool_names_hash.contains(tool_name) {
                    score += 1.0;
                }
            }
        }

        if let Some(ref export_name) = self.export_name {
            criteria += 1;
            if fp.export_names_hash.contains(export_name) {
                score += 1.0;
            }
        }

        if let Some(ref side_effects) = self.side_effects {
            criteria += 1;
            if fp.side_effects.as_deref() == Some(side_effects) {
                score += 1.0;
            }
        }

        if let Some(ref capability) = self.capability {
            criteria += 1;
            if fp.capabilities_declared.iter().any(|c| c == capability) {
                score += 1.0;
            }
        }

        if criteria == 0 || score == 0.0 {
            return 0.0;
        }

        score / criteria as f64
    }
}

impl Default for ApiShapeQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bnpm_manifest::schema::{AiBlock, SideEffect, StabilityLevel};

    fn make_ai_block() -> AiBlock {
        AiBlock {
            summary: Some("HTTP client".into()),
            intent: Some("networking/http-client".into()),
            capabilities_declared: Some(vec!["http.request".into(), "http.retry".into()]),
            stability: Some(StabilityLevel::Stable),
            side_effects: Some(SideEffect::None),
            docs: Some("./docs/agent.md".into()),
            tool_interfaces: Some("./ai/tools.json".into()),
            examples: Some("./ai/examples/".into()),
            supersedes: None,
            deprecated_in_favor_of: None,
            tags: None,
        }
    }

    fn make_tools() -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "request".into(),
            description: "Make an HTTP request".into(),
            input: crate::ToolSchema {
                type_: "object".into(),
                properties: Some(
                    vec![
                        (
                            "url".into(),
                            crate::ToolProperty {
                                type_: "string".into(),
                                description: Some("The URL".into()),
                                format: Some("uri".into()),
                                enum_: None,
                                items: None,
                            },
                        ),
                        (
                            "method".into(),
                            crate::ToolProperty {
                                type_: "string".into(),
                                description: Some("HTTP method".into()),
                                format: None,
                                enum_: Some(vec!["GET".into(), "POST".into()]),
                                items: None,
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                required: Some(vec!["url".into()]),
                description: None,
            },
            output: Some(crate::ToolSchema {
                type_: "object".into(),
                properties: None,
                required: None,
                description: None,
            }),
            side_effects: Some(vec!["network".into()]),
        }]
    }

    #[test]
    fn test_structural_indexer_index_package() {
        let mut indexer = StructuralIndexer::new();
        let ai = make_ai_block();
        let tool_file = crate::ToolInterfaceFile { tools: make_tools() };
        let tools_json = tool_file.to_json_string().unwrap();

        indexer
            .index_package(
                "@acme/http-client".into(),
                "2.3.0".into(),
                Some(&ai),
                Some(&tools_json),
                &["request".into(), "Client".into()],
            )
            .unwrap();

        assert_eq!(indexer.len(), 1);
    }

    #[test]
    fn test_search_by_api_shape_matches_intent() {
        let mut indexer = StructuralIndexer::new();
        let ai = make_ai_block();

        indexer.index_package_from_tools(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            Some(make_tools()),
            &["request".into()],
        );

        let query = ApiShapeQuery::new().with_intent("networking/http-client".into());
        let results = indexer.search_by_api_shape(&query);
        assert!(!results.is_empty());
        assert_eq!(results[0], "@acme/http-client");
    }

    #[test]
    fn test_search_by_api_shape_no_match() {
        let mut indexer = StructuralIndexer::new();
        let ai = make_ai_block();

        indexer.index_package_from_tools(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            Some(make_tools()),
            &["request".into()],
        );

        let query = ApiShapeQuery::new().with_intent("database/sql".into());
        let results = indexer.search_by_api_shape(&query);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_tool_name() {
        let mut indexer = StructuralIndexer::new();
        let ai = make_ai_block();

        indexer.index_package_from_tools(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            Some(make_tools()),
            &["request".into()],
        );

        let query = ApiShapeQuery::new().with_tool_name("request".into());
        let results = indexer.search_by_api_shape(&query);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_by_side_effects() {
        let mut indexer = StructuralIndexer::new();
        let ai = make_ai_block();

        indexer.index_package_from_tools(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            Some(make_tools()),
            &["request".into()],
        );

        let query = ApiShapeQuery::new().with_side_effects("none".into());
        let results = indexer.search_by_api_shape(&query);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_has_tools() {
        let mut indexer = StructuralIndexer::new();
        let ai = make_ai_block();

        indexer.index_package_from_tools(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            Some(make_tools()),
            &["request".into()],
        );

        // Package with no tools
        indexer.index_package_from_tools(
            "@acme/no-tools".into(),
            "1.0.0".into(),
            Some(&AiBlock {
                summary: Some("No tools".into()),
                intent: Some("utils/string".into()),
                capabilities_declared: None,
                stability: None,
                side_effects: None,
                docs: None,
                tool_interfaces: None,
                examples: None,
                supersedes: None,
                deprecated_in_favor_of: None,
                tags: None,
            }),
            None,
            &["format".into()],
        );

        let query = ApiShapeQuery::new().with_has_tools();
        let results = indexer.search_by_api_shape(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "@acme/http-client");
    }

    #[test]
    fn test_tool_fingerprint() {
        let tools = make_tools();
        let fp = ToolFingerprint::from_interface(&tools);
        assert_eq!(fp.tool_count, 1);
        assert!(fp.tool_names_hash.contains("request"));
        assert!(fp.param_signatures[0].contains("url:string"));
        assert!(fp.param_signatures[0].contains("method:string"));
    }

    #[test]
    fn test_tool_interface_file_roundtrip() {
        let tools = make_tools();
        let file = crate::ToolInterfaceFile { tools };
        let json = file.to_json_string().unwrap();
        let parsed = crate::ToolInterfaceFile::from_json_str(&json).unwrap();
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].name, "request");
    }
}
