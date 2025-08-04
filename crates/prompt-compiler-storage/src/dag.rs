//! DAG management - Versioned prompt evolution graph

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use crate::{Result, StorageError};
use prompt_compiler_crypto::Hash;

/// Versioned prompt DAG
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptDAG {
    pub nodes: HashMap<String, VersionedPrompt>,
    pub edges: HashMap<String, Vec<String>>, // hash -> children hashes
    pub roots: Vec<String>,                  // root node hashes
    pub branches: HashMap<String, String>,   // branch_name -> head_hash
}

/// Versioned prompt node
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionedPrompt {
    pub content_hash: Hash,
    pub parent_hashes: Vec<String>,          // Support multi-parent nodes
    pub children_hashes: Vec<String>,        // Child node references
    pub branch_name: Option<String>,         // Branch name
    pub tags: Vec<String>,                   // Version tags
    pub metadata: VersionMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionMetadata {
    pub created_by: String,
    pub commit_message: String,
    pub weight_update_summary: WeightUpdateSummary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeightUpdateSummary {
    pub total_updates: usize,
    pub avg_update_norm: f32,
    pub convergence_achieved: bool,
    pub effectiveness_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGStats {
    pub total_nodes: usize,
    pub total_branches: usize,
    pub total_roots: usize,
    pub avg_children_per_node: f32,
    pub convergence_rate: f32,
}

impl PromptDAG {
    /// Create new DAG
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            roots: Vec::new(),
            branches: HashMap::new(),
        }
    }

    /// Add new node
    pub fn add_node(
        &mut self,
        hash: String,
        content_hash: Hash,
        parent_hashes: Vec<String>,
        branch_name: Option<String>,
        commit_message: String,
    ) -> Result<()> {
        // Verify parent nodes exist
        for parent_hash in &parent_hashes {
            if !self.nodes.contains_key(parent_hash) {
                return Err(StorageError::NotFound(parent_hash.clone()));
            }
        }

        // Create weight update summary
        let weight_summary = WeightUpdateSummary {
            total_updates: 0, // Will be filled from actual data
            avg_update_norm: 0.0,
            convergence_achieved: false,
            effectiveness_score: 0.0,
        };

        let metadata = VersionMetadata {
            created_by: "user".to_string(), // Could be obtained from environment
            commit_message,
            weight_update_summary: weight_summary,
        };

        let versioned_prompt = VersionedPrompt {
            content_hash,
            parent_hashes: parent_hashes.clone(),
            children_hashes: Vec::new(),
            branch_name: branch_name.clone(),
            tags: Vec::new(),
            metadata,
        };

        // Add node
        self.nodes.insert(hash.clone(), versioned_prompt);

        // Update parent nodes' children lists
        for parent_hash in &parent_hashes {
            if let Some(parent) = self.nodes.get_mut(parent_hash) {
                parent.children_hashes.push(hash.clone());
            }
        }

        // Update edges
        self.edges.insert(hash.clone(), Vec::new());
        for parent_hash in &parent_hashes {
            if let Some(children) = self.edges.get_mut(parent_hash) {
                children.push(hash.clone());
            }
        }

        // If no parents, it's a root node
        if parent_hashes.is_empty() {
            self.roots.push(hash.clone());
        }

        // Update branch
        if let Some(branch) = branch_name {
            self.branches.insert(branch, hash);
        }

        Ok(())
    }

    /// Find path from A to B
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.edges.get(&current) {
                for child in children {
                    if !visited.contains(child) {
                        visited.insert(child.clone());
                        parent_map.insert(child.clone(), current.clone());
                        queue.push_back(child.clone());

                        if child == to {
                            // Reconstruct path
                            let mut path = Vec::new();
                            let mut current_node = to.to_string();
                            path.push(current_node.clone());

                            while let Some(parent) = parent_map.get(&current_node) {
                                path.push(parent.clone());
                                current_node = parent.clone();
                            }

                            path.reverse();
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    /// Create new branch
    pub fn create_branch(&mut self, from_hash: &str, branch_name: &str) -> Result<()> {
        if !self.nodes.contains_key(from_hash) {
            return Err(StorageError::NotFound(from_hash.to_string()));
        }

        if self.branches.contains_key(branch_name) {
            return Err(StorageError::InvalidData(
                format!("Branch {} already exists", branch_name)
            ));
        }

        self.branches.insert(branch_name.to_string(), from_hash.to_string());
        Ok(())
    }

    /// Merge branches
    pub fn merge_branches(
        &mut self,
        source_branch: &str,
        target_branch: &str,
        merge_hash: String,
        content_hash: Hash,
        commit_message: String,
    ) -> Result<()> {
        let source_hash = self.branches.get(source_branch)
            .ok_or_else(|| StorageError::NotFound(
                format!("Source branch {} does not exist", source_branch)
            ))?
            .clone();

        let target_hash = self.branches.get(target_branch)
            .ok_or_else(|| StorageError::NotFound(
                format!("Target branch {} does not exist", target_branch)
            ))?
            .clone();

        // Create merge node
        self.add_node(
            merge_hash.clone(),
            content_hash,
            vec![source_hash, target_hash],
            Some(target_branch.to_string()),
            commit_message,
        )?;

        // Update target branch pointer
        self.branches.insert(target_branch.to_string(), merge_hash);

        Ok(())
    }

    /// Get all commits in a branch
    pub fn get_branch_commits(&self, branch_name: &str) -> Result<Vec<String>> {
        let head_hash = self.branches.get(branch_name)
            .ok_or_else(|| StorageError::NotFound(
                format!("Branch {} does not exist", branch_name)
            ))?;

        let mut commits = Vec::new();
        let mut current = head_hash.clone();

        loop {
            commits.push(current.clone());
            
            if let Some(node) = self.nodes.get(&current) {
                if node.parent_hashes.is_empty() {
                    break;
                }
                // Choose first parent (linear history)
                current = node.parent_hashes[0].clone();
            } else {
                break;
            }
        }

        commits.reverse();
        Ok(commits)
    }

    /// Add tag
    pub fn add_tag(&mut self, hash: &str, tag: &str) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(hash) {
            if !node.tags.contains(&tag.to_string()) {
                node.tags.push(tag.to_string());
            }
            Ok(())
        } else {
            Err(StorageError::NotFound(hash.to_string()))
        }
    }

    /// Find nodes by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<String> {
        self.nodes.iter()
            .filter(|(_, node)| node.tags.contains(&tag.to_string()))
            .map(|(hash, _)| hash.clone())
            .collect()
    }

    /// Get DAG statistics
    pub fn get_stats(&self) -> DAGStats {
        let total_nodes = self.nodes.len();
        let total_branches = self.branches.len();
        let total_roots = self.roots.len();
        
        let avg_children = if total_nodes > 0 {
            self.edges.values().map(|children| children.len()).sum::<usize>() as f32 / total_nodes as f32
        } else {
            0.0
        };

        let convergence_rate = if total_nodes > 0 {
            let converged_count = self.nodes.values()
                .filter(|node| node.metadata.weight_update_summary.convergence_achieved)
                .count();
            converged_count as f32 / total_nodes as f32
        } else {
            0.0
        };

        DAGStats {
            total_nodes,
            total_branches,
            total_roots,
            avg_children_per_node: avg_children,
            convergence_rate,
        }
    }
}

impl Default for PromptDAG {
    fn default() -> Self {
        Self::new()
    }
}
