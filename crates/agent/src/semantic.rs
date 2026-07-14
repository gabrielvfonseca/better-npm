use std::collections::HashMap;

/// A trait for embedding engines that convert text to vector embeddings.
pub trait EmbeddingEngine: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>>;
    fn dimensions(&self) -> usize;
}

/// A stub embedding engine that returns deterministic fake embeddings.
///
/// This is useful for MVP/development — it generates a simple hash-based
/// vector so that identical inputs produce identical embeddings.
#[derive(Debug, Clone)]
pub struct StubEmbeddingEngine {
    dimensions: usize,
}

impl StubEmbeddingEngine {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl Default for StubEmbeddingEngine {
    fn default() -> Self {
        Self::new(128)
    }
}

impl EmbeddingEngine for StubEmbeddingEngine {
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; self.dimensions];
        let bytes = text.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            let idx = i % self.dimensions;
            vec[idx] += byte as f32 / 255.0;
        }
        // Normalize
        let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut vec {
                *v /= magnitude;
            }
        }
        vec
    }

    fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// A trait for semantic search indexers.
pub trait SemanticIndexer: Send + Sync {
    fn index_package(&mut self, package_id: String, text: &str);
    fn search_by_semantic(&self, query: &str, k: usize) -> Vec<(String, f32)>;
    fn remove_package(&mut self, package_id: &str);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A basic semantic search index using the stub embedding engine.
pub struct BasicSemanticIndex {
    engine: Box<dyn EmbeddingEngine>,
    embeddings: HashMap<String, Vec<f32>>,
    texts: HashMap<String, String>,
}

impl BasicSemanticIndex {
    pub fn new(engine: Box<dyn EmbeddingEngine>) -> Self {
        Self { engine, embeddings: HashMap::new(), texts: HashMap::new() }
    }
}

impl SemanticIndexer for BasicSemanticIndex {
    fn index_package(&mut self, package_id: String, text: &str) {
        let embedding = self.engine.embed(text);
        self.embeddings.insert(package_id.clone(), embedding);
        self.texts.insert(package_id, text.to_string());
    }

    fn search_by_semantic(&self, query: &str, k: usize) -> Vec<(String, f32)> {
        let query_embedding = self.engine.embed(query);
        let mut scores: Vec<(String, f32)> = self
            .embeddings
            .iter()
            .map(|(id, emb)| {
                let similarity = cosine_similarity(&query_embedding, emb);
                (id.clone(), similarity)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }

    fn remove_package(&mut self, package_id: &str) {
        self.embeddings.remove(package_id);
        self.texts.remove(package_id);
    }

    fn len(&self) -> usize {
        self.embeddings.len()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index() -> BasicSemanticIndex {
        BasicSemanticIndex::new(Box::new(StubEmbeddingEngine::default()))
    }

    #[test]
    fn test_stub_embedding_deterministic() {
        let engine = StubEmbeddingEngine::default();
        let a = engine.embed("hello world");
        let b = engine.embed("hello world");
        assert_eq!(a, b);
        assert_eq!(a.len(), 128);
    }

    #[test]
    fn test_stub_embedding_different_inputs() {
        let engine = StubEmbeddingEngine::default();
        let a = engine.embed("hello world");
        let b = engine.embed("goodbye world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_semantic_index_basic() {
        let mut index = make_index();
        index.index_package("pkg-a".into(), "HTTP client with retries");
        index.index_package("pkg-b".into(), "SQL database driver");

        let results = index.search_by_semantic("http client", 5);
        assert!(!results.is_empty());
        // Both should appear in results (stub is deterministic, both get some score)
        assert!(results.iter().any(|(id, _)| id == "pkg-a"));
        assert!(results.iter().any(|(id, _)| id == "pkg-b"));
    }

    #[test]
    fn test_semantic_index_empty() {
        let mut index = make_index();
        assert!(index.is_empty());
        index.index_package("pkg-a".into(), "test");
        assert!(!index.is_empty());
    }

    #[test]
    fn test_semantic_index_search_limit() {
        let mut index = make_index();
        index.index_package("pkg-a".into(), "HTTP client with retries");
        index.index_package("pkg-b".into(), "SQL database driver");
        index.index_package("pkg-c".into(), "Another HTTP library");

        let results = index.search_by_semantic("http", 2);
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_semantic_index_remove() {
        let mut index = make_index();
        index.index_package("pkg-a".into(), "HTTP client");
        assert_eq!(index.len(), 1);
        index.remove_package("pkg-a");
        assert!(index.is_empty());
    }

    #[test]
    fn test_embedding_dimensions() {
        let engine = StubEmbeddingEngine::new(64);
        let vec = engine.embed("test");
        assert_eq!(vec.len(), 64);
        assert_eq!(engine.dimensions(), 64);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let vec = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&vec, &vec);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }
}
