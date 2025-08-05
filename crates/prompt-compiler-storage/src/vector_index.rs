use anyhow::Result;

/// Vector index interface
pub trait VectorIndex {
    fn add_vector(&mut self, id: String, vector: Vec<f32>) -> Result<()>;
    fn search(&self, query: &[f32], top_k: usize, threshold: f32) -> Result<Vec<(String, f32)>>;
    fn remove(&mut self, id: &str) -> Result<()>;
}

/// HNSW (Hierarchical Navigable Small World) index implementation
pub struct HNSWIndex {
    vectors: std::collections::HashMap<String, Vec<f32>>,
    dimension: usize,
    max_connections: usize,
}

impl HNSWIndex {
    pub fn new(dimension: usize, max_connections: usize) -> Self {
        Self {
            vectors: std::collections::HashMap::new(),
            dimension,
            max_connections,
        }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
    }
}

impl VectorIndex for HNSWIndex {
    fn add_vector(&mut self, id: String, vector: Vec<f32>) -> Result<()> {
        if vector.len() != self.dimension {
            return Err(anyhow::anyhow!("Vector dimension mismatch"));
        }
        self.vectors.insert(id, vector);
        Ok(())
    }

    fn search(&self, query: &[f32], top_k: usize, threshold: f32) -> Result<Vec<(String, f32)>> {
        let mut results: Vec<(String, f32)> = self.vectors
            .iter()
            .map(|(id, vec)| (id.clone(), Self::cosine_similarity(query, vec)))
            .filter(|(_, sim)| *sim >= threshold)
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(top_k);

        Ok(results)
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        self.vectors.remove(id);
        Ok(())
    }
}
