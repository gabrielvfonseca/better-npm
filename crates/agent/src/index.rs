use crate::semantic::{BasicSemanticIndex, EmbeddingEngine, SemanticIndexer, StubEmbeddingEngine};
use crate::structural::StructuralIndexer;
use crate::tool_interface::ToolInterfaceFile;
use dash_manifest::schema::AiBlock;

/// A combined index that provides structural and semantic search over packages.
pub struct AgentIndex {
    pub structural: StructuralIndexer,
    pub semantic: BasicSemanticIndex,
}

impl AgentIndex {
    pub fn new() -> Self {
        Self {
            structural: StructuralIndexer::new(),
            semantic: BasicSemanticIndex::new(Box::new(StubEmbeddingEngine::default())),
        }
    }

    pub fn with_embedding_engine(engine: Box<dyn EmbeddingEngine>) -> Self {
        Self { structural: StructuralIndexer::new(), semantic: BasicSemanticIndex::new(engine) }
    }

    pub fn index_package(
        &mut self,
        package_name: String,
        version: String,
        ai: Option<&AiBlock>,
        tools_json: Option<&str>,
        exports: &[String],
        summary_text: &str,
    ) {
        let tools = if let Some(json) = tools_json {
            ToolInterfaceFile::from_json_str(json).ok().map(|f| f.tools)
        } else {
            None
        };

        self.structural
            .index_package_from_tools(package_name.clone(), version, ai, tools, exports);

        self.semantic.index_package(package_name, summary_text);
    }

    pub fn search(&self, query: &str, k: usize) -> Vec<(String, f32)> {
        self.semantic.search_by_semantic(query, k)
    }
}

impl Default for AgentIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_manifest::schema::{SideEffect, StabilityLevel};

    fn make_ai() -> AiBlock {
        AiBlock {
            summary: Some("HTTP client with automatic retries".into()),
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

    #[test]
    fn test_agent_index_search() {
        let mut index = AgentIndex::new();
        let ai = make_ai();

        index.index_package(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            None,
            &["request".into()],
            "HTTP client with automatic retries, no native deps",
        );

        let results = index.search("http client", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_agent_index_structural_search() {
        let mut index = AgentIndex::new();
        let ai = make_ai();

        index.index_package(
            "@acme/http-client".into(),
            "2.3.0".into(),
            Some(&ai),
            None,
            &["request".into()],
            "HTTP client",
        );

        let query = crate::ApiShapeQuery::new().with_intent("networking/http-client".into());
        let results = index.structural.search_by_api_shape(&query);
        assert!(!results.is_empty());
    }
}
