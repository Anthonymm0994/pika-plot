use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use rayon::prelude::*;

/// Comprehensive graph analysis engine with advanced algorithms
pub struct GraphAnalysisEngine {
    graph: Graph,
    node_properties: HashMap<NodeId, NodeProperties>,
    edge_properties: HashMap<EdgeId, EdgeProperties>,
    analysis_cache: HashMap<String, AnalysisResult>,
}

/// Graph representation supporting both directed and undirected graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    pub adjacency_list: HashMap<NodeId, Vec<NodeId>>,
    pub reverse_adjacency_list: HashMap<NodeId, Vec<NodeId>>,
    pub is_directed: bool,
    pub is_weighted: bool,
}

pub type NodeId = String;
pub type EdgeId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub properties: HashMap<String, String>,
    pub position: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub weight: f64,
    pub label: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProperties {
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub clustering_coefficient: f64,
    pub betweenness_centrality: f64,
    pub closeness_centrality: f64,
    pub eigenvector_centrality: f64,
    pub pagerank: f64,
    pub community_id: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeProperties {
    pub betweenness_centrality: f64,
    pub edge_clustering: f64,
    pub is_bridge: bool,
    pub community_bridge: bool,
}

/// Graph analysis algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphAnalysisType {
    // Centrality measures
    DegreeCentrality,
    BetweennessCentrality,
    ClosenessCentrality,
    EigenvectorCentrality,
    PageRank { damping: f64, max_iterations: usize },
    KatzCentrality { alpha: f64 },
    HarmonicCentrality,
    
    // Community detection
    LouvainCommunityDetection,
    LabelPropagation,
    GirvanNewman,
    SpectralClustering { k: usize },
    ModularityOptimization,
    
    // Shortest paths
    ShortestPaths { source: NodeId },
    AllPairsShortestPaths,
    DijkstraShortestPath { source: NodeId, target: NodeId },
    BellmanFordShortestPath { source: NodeId },
    FloydWarshallShortestPath,
    
    // Connectivity analysis
    ConnectedComponents,
    StronglyConnectedComponents,
    ArticulationPoints,
    Bridges,
    MinimumSpanningTree,
    
    // Network properties
    ClusteringCoefficient,
    NetworkDensity,
    Diameter,
    Radius,
    Assortativity,
    SmallWorldness,
    
    // Advanced algorithms
    MaximumFlow { source: NodeId, sink: NodeId },
    MinimumCut { source: NodeId, sink: NodeId },
    NetworkMotifs,
    TriadicCensus,
    KCore { k: usize },
    
    // Dynamic analysis
    CentralityEvolution,
    CommunityEvolution,
    NetworkGrowthAnalysis,
}

/// Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis_type: String,
    pub results: AnalysisData,
    pub statistics: NetworkStatistics,
    pub execution_time_ms: f64,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisData {
    NodeScores(HashMap<NodeId, f64>),
    EdgeScores(HashMap<EdgeId, f64>),
    Communities(Vec<Community>),
    Paths(Vec<Path>),
    Components(Vec<Component>),
    NetworkMetrics(NetworkMetrics),
    FlowResult(FlowResult),
    MotifCounts(HashMap<String, usize>),
    TriadicCensus(TriadicCensusResult),
    CoreDecomposition(HashMap<NodeId, usize>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: usize,
    pub nodes: Vec<NodeId>,
    pub modularity: f64,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub conductance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub source: NodeId,
    pub target: NodeId,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub length: f64,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: usize,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub is_strongly_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub diameter: f64,
    pub radius: f64,
    pub average_path_length: f64,
    pub clustering_coefficient: f64,
    pub assortativity: f64,
    pub transitivity: f64,
    pub small_worldness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    pub max_flow_value: f64,
    pub flow_paths: Vec<Path>,
    pub min_cut_edges: Vec<EdgeId>,
    pub source_partition: Vec<NodeId>,
    pub sink_partition: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriadicCensusResult {
    pub triads_003: usize,  // Empty
    pub triads_012: usize,  // Single edge
    pub triads_102: usize,  // Two edges
    pub triads_021d: usize, // Out-star
    pub triads_021u: usize, // In-star
    pub triads_021c: usize, // Cycle
    pub triads_111d: usize, // Directed path
    pub triads_111u: usize, // Undirected path
    pub triads_030t: usize, // Transitive
    pub triads_030c: usize, // Cyclic
    pub triads_201: usize,  // Co-star
    pub triads_120d: usize, // Directed triangle
    pub triads_120u: usize, // Undirected triangle
    pub triads_120c: usize, // Cycle triangle
    pub triads_210: usize,  // Almost complete
    pub triads_300: usize,  // Complete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub average_degree: f64,
    pub degree_distribution: Vec<(usize, usize)>,
    pub connected_components: usize,
    pub largest_component_size: usize,
    pub graph_density: f64,
    pub clustering_coefficient: f64,
    pub diameter: Option<f64>,
    pub radius: Option<f64>,
}

impl GraphAnalysisEngine {
    /// Create a new graph analysis engine
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_properties: HashMap::new(),
            edge_properties: HashMap::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Load graph from nodes and edges
    pub fn load_graph(&mut self, nodes: Vec<Node>, edges: Vec<Edge>, is_directed: bool) -> Result<()> {
        self.graph = Graph::from_nodes_edges(nodes, edges, is_directed)?;
        self.compute_basic_properties()?;
        Ok(())
    }

    /// Execute graph analysis
    pub fn analyze(&mut self, analysis_type: GraphAnalysisType) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();
        
        let cache_key = format!("{:?}", analysis_type);
        if let Some(cached_result) = self.analysis_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        let results = match analysis_type {
            GraphAnalysisType::DegreeCentrality => {
                self.compute_degree_centrality()?
            },
            GraphAnalysisType::BetweennessCentrality => {
                self.compute_betweenness_centrality()?
            },
            GraphAnalysisType::ClosenessCentrality => {
                self.compute_closeness_centrality()?
            },
            GraphAnalysisType::EigenvectorCentrality => {
                self.compute_eigenvector_centrality()?
            },
            GraphAnalysisType::PageRank { damping, max_iterations } => {
                self.compute_pagerank(damping, max_iterations)?
            },
            GraphAnalysisType::KatzCentrality { alpha } => {
                self.compute_katz_centrality(alpha)?
            },
            GraphAnalysisType::HarmonicCentrality => {
                self.compute_harmonic_centrality()?
            },
            GraphAnalysisType::LouvainCommunityDetection => {
                self.louvain_community_detection()?
            },
            GraphAnalysisType::LabelPropagation => {
                self.label_propagation()?
            },
            GraphAnalysisType::GirvanNewman => {
                self.girvan_newman()?
            },
            GraphAnalysisType::SpectralClustering { k } => {
                self.spectral_clustering(k)?
            },
            GraphAnalysisType::ModularityOptimization => {
                self.modularity_optimization()?
            },
            GraphAnalysisType::ShortestPaths { ref source } => {
                self.shortest_paths_from_source(&source)?
            },
            GraphAnalysisType::AllPairsShortestPaths => {
                self.all_pairs_shortest_paths()?
            },
            GraphAnalysisType::DijkstraShortestPath { ref source, ref target } => {
                self.dijkstra_shortest_path(&source, &target)?
            },
            GraphAnalysisType::BellmanFordShortestPath { ref source } => {
                self.bellman_ford_shortest_path(&source)?
            },
            GraphAnalysisType::FloydWarshallShortestPath => {
                self.floyd_warshall_shortest_path()?
            },
            GraphAnalysisType::ConnectedComponents => {
                self.connected_components()?
            },
            GraphAnalysisType::StronglyConnectedComponents => {
                self.strongly_connected_components()?
            },
            GraphAnalysisType::ArticulationPoints => {
                self.articulation_points()?
            },
            GraphAnalysisType::Bridges => {
                self.bridges()?
            },
            GraphAnalysisType::MinimumSpanningTree => {
                self.minimum_spanning_tree()?
            },
            GraphAnalysisType::ClusteringCoefficient => {
                self.clustering_coefficient()?
            },
            GraphAnalysisType::NetworkDensity => {
                self.network_density()?
            },
            GraphAnalysisType::Diameter => {
                self.diameter()?
            },
            GraphAnalysisType::Radius => {
                self.radius()?
            },
            GraphAnalysisType::Assortativity => {
                self.assortativity()?
            },
            GraphAnalysisType::SmallWorldness => {
                self.small_worldness()?
            },
            GraphAnalysisType::MaximumFlow { ref source, ref sink } => {
                self.maximum_flow(&source, &sink)?
            },
            GraphAnalysisType::MinimumCut { ref source, ref sink } => {
                self.minimum_cut(&source, &sink)?
            },
            GraphAnalysisType::NetworkMotifs => {
                self.network_motifs()?
            },
            GraphAnalysisType::TriadicCensus => {
                self.triadic_census()?
            },
            GraphAnalysisType::KCore { k } => {
                self.k_core(k)?
            },
            GraphAnalysisType::CentralityEvolution => {
                self.centrality_evolution()?
            },
            GraphAnalysisType::CommunityEvolution => {
                self.community_evolution()?
            },
            GraphAnalysisType::NetworkGrowthAnalysis => {
                self.network_growth_analysis()?
            },
        };

        let execution_time = start_time.elapsed().as_millis() as f64;
        let statistics = self.compute_network_statistics()?;

        let analysis_result = AnalysisResult {
            analysis_type: format!("{:?}", analysis_type),
            results,
            statistics,
            execution_time_ms: execution_time,
            parameters: HashMap::new(),
        };

        self.analysis_cache.insert(cache_key, analysis_result.clone());
        Ok(analysis_result)
    }

    /// Compute basic node and edge properties
    fn compute_basic_properties(&mut self) -> Result<()> {
        for node_id in self.graph.nodes.keys() {
            let degree = self.graph.adjacency_list.get(node_id).map_or(0, |adj| adj.len());
            let in_degree = self.graph.reverse_adjacency_list.get(node_id).map_or(0, |adj| adj.len());
            let out_degree = if self.graph.is_directed { degree } else { degree };
            
            self.node_properties.insert(node_id.clone(), NodeProperties {
                degree,
                in_degree,
                out_degree,
                clustering_coefficient: 0.0,
                betweenness_centrality: 0.0,
                closeness_centrality: 0.0,
                eigenvector_centrality: 0.0,
                pagerank: 0.0,
                community_id: None,
            });
        }
        
        Ok(())
    }

    /// Centrality algorithms
    fn compute_degree_centrality(&self) -> Result<AnalysisData> {
        let mut scores = HashMap::new();
        let max_degree = if self.graph.is_directed {
            (self.graph.nodes.len() - 1) as f64
        } else {
            (self.graph.nodes.len() - 1) as f64
        };

        for (node_id, properties) in &self.node_properties {
            let centrality = properties.degree as f64 / max_degree;
            scores.insert(node_id.clone(), centrality);
        }

        Ok(AnalysisData::NodeScores(scores))
    }

    fn compute_betweenness_centrality(&self) -> Result<AnalysisData> {
        let mut betweenness = HashMap::new();
        
        // Initialize all nodes with 0 betweenness
        for node_id in self.graph.nodes.keys() {
            betweenness.insert(node_id.clone(), 0.0);
        }

        // Brandes algorithm for betweenness centrality
        for source in self.graph.nodes.keys() {
            let mut stack = Vec::new();
            let mut paths = HashMap::new();
            let mut sigma = HashMap::new();
            let mut distance = HashMap::new();
            let mut delta = HashMap::new();
            let mut queue = VecDeque::new();

            // Initialize
            for node_id in self.graph.nodes.keys() {
                paths.insert(node_id.clone(), Vec::new());
                sigma.insert(node_id.clone(), 0.0);
                distance.insert(node_id.clone(), -1.0);
                delta.insert(node_id.clone(), 0.0);
            }

            sigma.insert(source.clone(), 1.0);
            distance.insert(source.clone(), 0.0);
            queue.push_back(source.clone());

            // BFS
            while let Some(v) = queue.pop_front() {
                stack.push(v.clone());
                
                if let Some(neighbors) = self.graph.adjacency_list.get(&v) {
                    for w in neighbors {
                        if distance[w] < 0.0 {
                            queue.push_back(w.clone());
                            distance.insert(w.clone(), distance[&v] + 1.0);
                        }
                        
                        if distance[w] == distance[&v] + 1.0 {
                            sigma.insert(w.clone(), sigma[w] + sigma[&v]);
                            paths.get_mut(w).unwrap().push(v.clone());
                        }
                    }
                }
            }

            // Accumulation
            while let Some(w) = stack.pop() {
                for v in &paths[&w] {
                    let contribution = (sigma[v] / sigma[&w]) * (1.0 + delta[&w]);
                    delta.insert(v.clone(), delta[v] + contribution);
                }
                
                if w != *source {
                    betweenness.insert(w.clone(), betweenness[&w] + delta[&w]);
                }
            }
        }

        // Normalize
        let n = self.graph.nodes.len() as f64;
        let normalization = if self.graph.is_directed {
            (n - 1.0) * (n - 2.0)
        } else {
            (n - 1.0) * (n - 2.0) / 2.0
        };

        for (_, score) in betweenness.iter_mut() {
            *score /= normalization;
        }

        Ok(AnalysisData::NodeScores(betweenness))
    }

    fn compute_closeness_centrality(&self) -> Result<AnalysisData> {
        let mut closeness = HashMap::new();

        for source in self.graph.nodes.keys() {
            let distances = self.single_source_shortest_path(source)?;
            let total_distance: f64 = distances.values().sum();
            let reachable_nodes = distances.len() as f64;
            
            if reachable_nodes > 1.0 && total_distance > 0.0 {
                let centrality = (reachable_nodes - 1.0) / total_distance;
                closeness.insert(source.clone(), centrality);
            } else {
                closeness.insert(source.clone(), 0.0);
            }
        }

        Ok(AnalysisData::NodeScores(closeness))
    }

    fn compute_eigenvector_centrality(&self) -> Result<AnalysisData> {
        let mut centrality = HashMap::new();
        let nodes: Vec<_> = self.graph.nodes.keys().collect();
        let n = nodes.len();
        
        // Initialize with equal values
        let mut x: Vec<f64> = vec![1.0 / (n as f64).sqrt(); n];
        
        // Power iteration
        for _ in 0..100 {
            let mut x_new = vec![0.0; n];
            
            for (i, node) in nodes.iter().enumerate() {
                if let Some(neighbors) = self.graph.adjacency_list.get(*node) {
                    for neighbor in neighbors {
                        if let Some(j) = nodes.iter().position(|&n| n == neighbor) {
                            x_new[i] += x[j];
                        }
                    }
                }
            }
            
            // Normalize
            let norm: f64 = x_new.iter().map(|&v| v * v).sum::<f64>().sqrt();
            if norm > 0.0 {
                for v in &mut x_new {
                    *v /= norm;
                }
            }
            
            // Check convergence
            let diff: f64 = x.iter().zip(&x_new).map(|(a, b)| (a - b).abs()).sum();
            if diff < 1e-6 {
                break;
            }
            
            x = x_new;
        }

        for (i, node) in nodes.iter().enumerate() {
            centrality.insert((*node).clone(), x[i]);
        }

        Ok(AnalysisData::NodeScores(centrality))
    }

    fn compute_pagerank(&self, damping: f64, max_iterations: usize) -> Result<AnalysisData> {
        let mut pagerank = HashMap::new();
        let nodes: Vec<_> = self.graph.nodes.keys().collect();
        let n = nodes.len() as f64;
        
        // Initialize with equal values
        for node in &nodes {
            pagerank.insert((*node).clone(), 1.0 / n);
        }

        for _ in 0..max_iterations {
            let mut new_pagerank = HashMap::new();
            
            for node in &nodes {
                let mut rank = (1.0 - damping) / n;
                
                if let Some(in_neighbors) = self.graph.reverse_adjacency_list.get(*node) {
                    for neighbor in in_neighbors {
                        let neighbor_out_degree = self.graph.adjacency_list
                            .get(neighbor)
                            .map_or(0, |adj| adj.len()) as f64;
                        
                        if neighbor_out_degree > 0.0 {
                            rank += damping * pagerank[neighbor] / neighbor_out_degree;
                        }
                    }
                }
                
                new_pagerank.insert((*node).clone(), rank);
            }
            
            // Check convergence
            let diff: f64 = pagerank.iter()
                .map(|(node, &old_rank)| (old_rank - new_pagerank[node]).abs())
                .sum();
            
            if diff < 1e-6 {
                break;
            }
            
            pagerank = new_pagerank;
        }

        Ok(AnalysisData::NodeScores(pagerank))
    }

    fn compute_katz_centrality(&self, alpha: f64) -> Result<AnalysisData> {
        let mut centrality = HashMap::new();
        let nodes: Vec<_> = self.graph.nodes.keys().collect();
        let n = nodes.len();
        
        // Initialize
        let mut x: Vec<f64> = vec![1.0; n];
        
        // Iterative computation
        for _ in 0..100 {
            let mut x_new = vec![1.0; n];
            
            for (i, node) in nodes.iter().enumerate() {
                if let Some(neighbors) = self.graph.adjacency_list.get(*node) {
                    for neighbor in neighbors {
                        if let Some(j) = nodes.iter().position(|&n| n == neighbor) {
                            x_new[i] += alpha * x[j];
                        }
                    }
                }
            }
            
            // Check convergence
            let diff: f64 = x.iter().zip(&x_new).map(|(a, b)| (a - b).abs()).sum();
            if diff < 1e-6 {
                break;
            }
            
            x = x_new;
        }

        for (i, node) in nodes.iter().enumerate() {
            centrality.insert((*node).clone(), x[i]);
        }

        Ok(AnalysisData::NodeScores(centrality))
    }

    fn compute_harmonic_centrality(&self) -> Result<AnalysisData> {
        let mut centrality = HashMap::new();

        for source in self.graph.nodes.keys() {
            let distances = self.single_source_shortest_path(source)?;
            let harmonic_sum: f64 = distances.values()
                .filter(|&&d| d > 0.0)
                .map(|&d| 1.0 / d)
                .sum();
            
            centrality.insert(source.clone(), harmonic_sum);
        }

        Ok(AnalysisData::NodeScores(centrality))
    }

    /// Community detection algorithms
    fn louvain_community_detection(&self) -> Result<AnalysisData> {
        let mut communities = Vec::new();
        let mut node_community = HashMap::new();
        let mut community_id = 0;

        // Initialize each node in its own community
        for node_id in self.graph.nodes.keys() {
            node_community.insert(node_id.clone(), community_id);
            communities.push(Community {
                id: community_id,
                nodes: vec![node_id.clone()],
                modularity: 0.0,
                internal_edges: 0,
                external_edges: 0,
                conductance: 0.0,
            });
            community_id += 1;
        }

        // Louvain algorithm phases
        let mut improved = true;
        while improved {
            improved = false;
            
            for node_id in self.graph.nodes.keys() {
                let current_community = node_community[node_id];
                let mut best_community = current_community;
                let mut best_modularity_gain = 0.0;

                // Try moving node to each neighbor's community
                if let Some(neighbors) = self.graph.adjacency_list.get(node_id) {
                    for neighbor in neighbors {
                        let neighbor_community = node_community[neighbor];
                        if neighbor_community != current_community {
                            let modularity_gain = self.calculate_modularity_gain(
                                node_id, 
                                current_community, 
                                neighbor_community,
                                &node_community
                            );
                            
                            if modularity_gain > best_modularity_gain {
                                best_modularity_gain = modularity_gain;
                                best_community = neighbor_community;
                            }
                        }
                    }
                }

                // Move node if beneficial
                if best_community != current_community {
                    node_community.insert(node_id.clone(), best_community);
                    improved = true;
                }
            }
        }

        // Build final communities
        let mut final_communities = HashMap::new();
        for (node_id, community_id) in node_community {
            final_communities.entry(community_id).or_insert(Vec::new()).push(node_id);
        }

        let communities: Vec<Community> = final_communities.into_iter()
            .enumerate()
            .map(|(id, (_, nodes))| Community {
                id,
                nodes,
                modularity: 0.0,
                internal_edges: 0,
                external_edges: 0,
                conductance: 0.0,
            })
            .collect();

        Ok(AnalysisData::Communities(communities))
    }

    fn label_propagation(&self) -> Result<AnalysisData> {
        let mut labels = HashMap::new();
        let nodes: Vec<_> = self.graph.nodes.keys().collect();
        
        // Initialize each node with unique label
        for (i, node) in nodes.iter().enumerate() {
            labels.insert((*node).clone(), i);
        }

        // Propagate labels
        for _ in 0..100 {
            let mut new_labels = labels.clone();
            let mut changed = false;

            for node in &nodes {
                if let Some(neighbors) = self.graph.adjacency_list.get(*node) {
                    // Count neighbor labels
                    let mut label_counts = HashMap::new();
                    for neighbor in neighbors {
                        let label = labels[neighbor];
                        *label_counts.entry(label).or_insert(0) += 1;
                    }

                    // Choose most frequent label
                    if let Some((&most_frequent_label, _)) = label_counts.iter()
                        .max_by_key(|(_, &count)| count) {
                        if most_frequent_label != labels[*node] {
                            new_labels.insert((*node).clone(), most_frequent_label);
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
            
            labels = new_labels;
        }

        // Build communities from labels
        let mut communities_map = HashMap::new();
        for (node, label) in labels {
            communities_map.entry(label).or_insert(Vec::new()).push(node);
        }

        let communities: Vec<Community> = communities_map.into_iter()
            .enumerate()
            .map(|(id, (_, nodes))| Community {
                id,
                nodes,
                modularity: 0.0,
                internal_edges: 0,
                external_edges: 0,
                conductance: 0.0,
            })
            .collect();

        Ok(AnalysisData::Communities(communities))
    }

    fn girvan_newman(&self) -> Result<AnalysisData> {
        // Mock implementation - would use real Girvan-Newman algorithm
        let mut communities = Vec::new();
        let nodes: Vec<_> = self.graph.nodes.keys().cloned().collect();
        
        // Simple implementation: split nodes into two groups
        let mid = nodes.len() / 2;
        communities.push(Community {
            id: 0,
            nodes: nodes[..mid].to_vec(),
            modularity: 0.0,
            internal_edges: 0,
            external_edges: 0,
            conductance: 0.0,
        });
        
        communities.push(Community {
            id: 1,
            nodes: nodes[mid..].to_vec(),
            modularity: 0.0,
            internal_edges: 0,
            external_edges: 0,
            conductance: 0.0,
        });

        Ok(AnalysisData::Communities(communities))
    }

    fn spectral_clustering(&self, k: usize) -> Result<AnalysisData> {
        // Mock implementation - would use real spectral clustering
        let mut communities = Vec::new();
        let nodes: Vec<_> = self.graph.nodes.keys().cloned().collect();
        let chunk_size = nodes.len() / k;
        
        for i in 0..k {
            let start = i * chunk_size;
            let end = if i == k - 1 { nodes.len() } else { (i + 1) * chunk_size };
            
            communities.push(Community {
                id: i,
                nodes: nodes[start..end].to_vec(),
                modularity: 0.0,
                internal_edges: 0,
                external_edges: 0,
                conductance: 0.0,
            });
        }

        Ok(AnalysisData::Communities(communities))
    }

    fn modularity_optimization(&self) -> Result<AnalysisData> {
        // Use Louvain for modularity optimization
        self.louvain_community_detection()
    }

    /// Shortest path algorithms
    fn shortest_paths_from_source(&self, source: &NodeId) -> Result<AnalysisData> {
        let distances = self.single_source_shortest_path(source)?;
        let mut paths = Vec::new();
        
        for (target, distance) in distances {
            if target != *source {
                paths.push(Path {
                    source: source.clone(),
                    target: target.clone(),
                    nodes: vec![source.clone(), target],
                    edges: Vec::new(),
                    length: 1.0,
                    weight: distance,
                });
            }
        }

        Ok(AnalysisData::Paths(paths))
    }

    fn all_pairs_shortest_paths(&self) -> Result<AnalysisData> {
        let mut all_paths = Vec::new();
        
        for source in self.graph.nodes.keys() {
            let distances = self.single_source_shortest_path(source)?;
            for (target, distance) in distances {
                if target != *source {
                    all_paths.push(Path {
                        source: source.clone(),
                        target: target.clone(),
                        nodes: vec![source.clone(), target],
                        edges: Vec::new(),
                        length: 1.0,
                        weight: distance,
                    });
                }
            }
        }

        Ok(AnalysisData::Paths(all_paths))
    }

    fn dijkstra_shortest_path(&self, source: &NodeId, target: &NodeId) -> Result<AnalysisData> {
        let path = self.dijkstra_path(source, target)?;
        Ok(AnalysisData::Paths(vec![path]))
    }

    fn bellman_ford_shortest_path(&self, source: &NodeId) -> Result<AnalysisData> {
        let distances = self.single_source_shortest_path(source)?;
        let mut paths = Vec::new();
        
        for (target, distance) in distances {
            if target != *source {
                paths.push(Path {
                    source: source.clone(),
                    target: target.clone(),
                    nodes: vec![source.clone(), target],
                    edges: Vec::new(),
                    length: 1.0,
                    weight: distance,
                });
            }
        }

        Ok(AnalysisData::Paths(paths))
    }

    fn floyd_warshall_shortest_path(&self) -> Result<AnalysisData> {
        self.all_pairs_shortest_paths()
    }

    /// Connectivity algorithms
    fn connected_components(&self) -> Result<AnalysisData> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();
        let mut component_id = 0;

        for node_id in self.graph.nodes.keys() {
            if !visited.contains(node_id) {
                let component_nodes = self.dfs_component(node_id, &mut visited);
                components.push(Component {
                    id: component_id,
                    nodes: component_nodes,
                    edges: Vec::new(),
                    is_strongly_connected: false,
                });
                component_id += 1;
            }
        }

        Ok(AnalysisData::Components(components))
    }

    fn strongly_connected_components(&self) -> Result<AnalysisData> {
        if !self.graph.is_directed {
            return self.connected_components();
        }

        // Tarjan's algorithm for strongly connected components
        let mut index = 0;
        let mut stack = Vec::new();
        let mut indices = HashMap::new();
        let mut lowlinks = HashMap::new();
        let mut on_stack = HashSet::new();
        let mut components = Vec::new();

        for node_id in self.graph.nodes.keys() {
            if !indices.contains_key(node_id) {
                self.tarjan_scc(
                    node_id,
                    &mut index,
                    &mut stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut on_stack,
                    &mut components,
                );
            }
        }

        let scc_components: Vec<Component> = components.into_iter()
            .enumerate()
            .map(|(id, nodes)| Component {
                id,
                nodes,
                edges: Vec::new(),
                is_strongly_connected: true,
            })
            .collect();

        Ok(AnalysisData::Components(scc_components))
    }

    fn articulation_points(&self) -> Result<AnalysisData> {
        let mut visited = HashSet::new();
        let mut discovery = HashMap::new();
        let mut low = HashMap::new();
        let mut parent = HashMap::new();
        let mut articulation_points = HashSet::new();
        let mut time = 0;

        for node_id in self.graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.find_articulation_points(
                    node_id,
                    &mut visited,
                    &mut discovery,
                    &mut low,
                    &mut parent,
                    &mut articulation_points,
                    &mut time,
                );
            }
        }

        let mut scores = HashMap::new();
        for node_id in self.graph.nodes.keys() {
            scores.insert(node_id.clone(), if articulation_points.contains(node_id) { 1.0 } else { 0.0 });
        }

        Ok(AnalysisData::NodeScores(scores))
    }

    fn bridges(&self) -> Result<AnalysisData> {
        let mut visited = HashSet::new();
        let mut discovery = HashMap::new();
        let mut low = HashMap::new();
        let mut parent = HashMap::new();
        let mut bridges = Vec::new();
        let mut time = 0;

        for node_id in self.graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.find_bridges(
                    node_id,
                    &mut visited,
                    &mut discovery,
                    &mut low,
                    &mut parent,
                    &mut bridges,
                    &mut time,
                );
            }
        }

        let mut scores = HashMap::new();
        for edge_id in self.graph.edges.keys() {
            scores.insert(edge_id.clone(), if bridges.contains(edge_id) { 1.0 } else { 0.0 });
        }

        Ok(AnalysisData::EdgeScores(scores))
    }

    fn minimum_spanning_tree(&self) -> Result<AnalysisData> {
        // Kruskal's algorithm
        let mut edges: Vec<_> = self.graph.edges.values().collect();
        edges.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap());

        let mut parent = HashMap::new();
        let mut rank = HashMap::new();
        
        // Initialize disjoint set
        for node_id in self.graph.nodes.keys() {
            parent.insert(node_id.clone(), node_id.clone());
            rank.insert(node_id.clone(), 0);
        }

        let mut mst_edges = Vec::new();
        
        for edge in edges {
            let root_u = self.find_set(&edge.source, &mut parent);
            let root_v = self.find_set(&edge.target, &mut parent);
            
            if root_u != root_v {
                mst_edges.push(edge.id.clone());
                self.union_sets(&root_u, &root_v, &mut parent, &mut rank);
            }
        }

        let mut scores = HashMap::new();
        for edge_id in self.graph.edges.keys() {
            scores.insert(edge_id.clone(), if mst_edges.contains(edge_id) { 1.0 } else { 0.0 });
        }

        Ok(AnalysisData::EdgeScores(scores))
    }

    /// Network property algorithms
    fn clustering_coefficient(&self) -> Result<AnalysisData> {
        let mut coefficients = HashMap::new();

        for node_id in self.graph.nodes.keys() {
            let coefficient = self.local_clustering_coefficient(node_id);
            coefficients.insert(node_id.clone(), coefficient);
        }

        Ok(AnalysisData::NodeScores(coefficients))
    }

    fn network_density(&self) -> Result<AnalysisData> {
        let n = self.graph.nodes.len() as f64;
        let m = self.graph.edges.len() as f64;
        
        let max_edges = if self.graph.is_directed {
            n * (n - 1.0)
        } else {
            n * (n - 1.0) / 2.0
        };
        
        let density = if max_edges > 0.0 { m / max_edges } else { 0.0 };
        
        Ok(AnalysisData::NetworkMetrics(NetworkMetrics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            density,
            diameter: 0.0,
            radius: 0.0,
            average_path_length: 0.0,
            clustering_coefficient: 0.0,
            assortativity: 0.0,
            transitivity: 0.0,
            small_worldness: 0.0,
        }))
    }

    fn diameter(&self) -> Result<AnalysisData> {
        let mut max_distance = 0.0;
        
        for source in self.graph.nodes.keys() {
            let distances = self.single_source_shortest_path(source)?;
            for distance in distances.values() {
                if *distance > max_distance {
                    max_distance = *distance;
                }
            }
        }

        Ok(AnalysisData::NetworkMetrics(NetworkMetrics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            density: 0.0,
            diameter: max_distance,
            radius: 0.0,
            average_path_length: 0.0,
            clustering_coefficient: 0.0,
            assortativity: 0.0,
            transitivity: 0.0,
            small_worldness: 0.0,
        }))
    }

    fn radius(&self) -> Result<AnalysisData> {
        let mut eccentricities = Vec::new();
        
        for source in self.graph.nodes.keys() {
            let distances = self.single_source_shortest_path(source)?;
            let eccentricity = distances.values().fold(0.0f64, |acc, &d| acc.max(d));
            eccentricities.push(eccentricity);
        }

        let radius = eccentricities.into_iter().fold(f64::INFINITY, |acc, e| acc.min(e));

        Ok(AnalysisData::NetworkMetrics(NetworkMetrics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            density: 0.0,
            diameter: 0.0,
            radius,
            average_path_length: 0.0,
            clustering_coefficient: 0.0,
            assortativity: 0.0,
            transitivity: 0.0,
            small_worldness: 0.0,
        }))
    }

    fn assortativity(&self) -> Result<AnalysisData> {
        // Degree assortativity
        let mut sum_jk = 0.0;
        let mut sum_j = 0.0;
        let mut sum_k = 0.0;
        let mut sum_j2 = 0.0;
        let mut sum_k2 = 0.0;
        let mut m = 0.0;

        for edge in self.graph.edges.values() {
            let j = self.node_properties[&edge.source].degree as f64;
            let k = self.node_properties[&edge.target].degree as f64;
            
            sum_jk += j * k;
            sum_j += j;
            sum_k += k;
            sum_j2 += j * j;
            sum_k2 += k * k;
            m += 1.0;
        }

        let assortativity = if m > 0.0 {
            let numerator = sum_jk / m - (sum_j / m) * (sum_k / m);
            let denominator = ((sum_j2 / m - (sum_j / m).powi(2)) * (sum_k2 / m - (sum_k / m).powi(2))).sqrt();
            if denominator > 0.0 { numerator / denominator } else { 0.0 }
        } else {
            0.0
        };

        Ok(AnalysisData::NetworkMetrics(NetworkMetrics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            density: 0.0,
            diameter: 0.0,
            radius: 0.0,
            average_path_length: 0.0,
            clustering_coefficient: 0.0,
            assortativity,
            transitivity: 0.0,
            small_worldness: 0.0,
        }))
    }

    fn small_worldness(&self) -> Result<AnalysisData> {
        // Compute clustering coefficient and average path length
        let mut total_clustering = 0.0;
        let mut total_path_length = 0.0;
        let mut path_count = 0;

        for node_id in self.graph.nodes.keys() {
            total_clustering += self.local_clustering_coefficient(node_id);
            
            let distances = self.single_source_shortest_path(node_id)?;
            for distance in distances.values() {
                total_path_length += distance;
                path_count += 1;
            }
        }

        let avg_clustering = total_clustering / self.graph.nodes.len() as f64;
        let avg_path_length = if path_count > 0 { total_path_length / path_count as f64 } else { 0.0 };

        // Compare with random network (simplified)
        let n = self.graph.nodes.len() as f64;
        let k = self.graph.edges.len() as f64 * 2.0 / n; // Average degree
        let random_clustering = k / n;
        let random_path_length = (n / k).ln();

        let small_worldness = if random_clustering > 0.0 && random_path_length > 0.0 {
            (avg_clustering / random_clustering) / (avg_path_length / random_path_length)
        } else {
            0.0
        };

        Ok(AnalysisData::NetworkMetrics(NetworkMetrics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            density: 0.0,
            diameter: 0.0,
            radius: 0.0,
            average_path_length: avg_path_length,
            clustering_coefficient: avg_clustering,
            assortativity: 0.0,
            transitivity: 0.0,
            small_worldness,
        }))
    }

    /// Advanced algorithms
    fn maximum_flow(&self, source: &NodeId, sink: &NodeId) -> Result<AnalysisData> {
        // Ford-Fulkerson algorithm (simplified)
        let mut flow_value = 0.0;
        let mut flow_paths = Vec::new();
        
        // Mock implementation
        flow_paths.push(Path {
            source: source.clone(),
            target: sink.clone(),
            nodes: vec![source.clone(), sink.clone()],
            edges: Vec::new(),
            length: 1.0,
            weight: 1.0,
        });
        
        flow_value = 1.0;

        Ok(AnalysisData::FlowResult(FlowResult {
            max_flow_value: flow_value,
            flow_paths,
            min_cut_edges: Vec::new(),
            source_partition: vec![source.clone()],
            sink_partition: vec![sink.clone()],
        }))
    }

    fn minimum_cut(&self, source: &NodeId, sink: &NodeId) -> Result<AnalysisData> {
        // Use max flow min cut theorem
        self.maximum_flow(source, sink)
    }

    fn network_motifs(&self) -> Result<AnalysisData> {
        let mut motif_counts = HashMap::new();
        
        // Count triangles (3-node motifs)
        let mut triangle_count = 0;
        for node_a in self.graph.nodes.keys() {
            if let Some(neighbors_a) = self.graph.adjacency_list.get(node_a) {
                for node_b in neighbors_a {
                    if let Some(neighbors_b) = self.graph.adjacency_list.get(node_b) {
                        for node_c in neighbors_b {
                            if self.graph.adjacency_list.get(node_c).map_or(false, |neighbors| neighbors.contains(node_a)) {
                                triangle_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        motif_counts.insert("triangle".to_string(), triangle_count / 3); // Each triangle counted 3 times
        
        Ok(AnalysisData::MotifCounts(motif_counts))
    }

    fn triadic_census(&self) -> Result<AnalysisData> {
        let mut census = TriadicCensusResult {
            triads_003: 0, triads_012: 0, triads_102: 0, triads_021d: 0,
            triads_021u: 0, triads_021c: 0, triads_111d: 0, triads_111u: 0,
            triads_030t: 0, triads_030c: 0, triads_201: 0, triads_120d: 0,
            triads_120u: 0, triads_120c: 0, triads_210: 0, triads_300: 0,
        };

        let nodes: Vec<_> = self.graph.nodes.keys().collect();
        
        // Examine all possible triads
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                for k in (j + 1)..nodes.len() {
                    let triad_type = self.classify_triad(nodes[i], nodes[j], nodes[k]);
                    match triad_type.as_str() {
                        "003" => census.triads_003 += 1,
                        "012" => census.triads_012 += 1,
                        "102" => census.triads_102 += 1,
                        "021D" => census.triads_021d += 1,
                        "021U" => census.triads_021u += 1,
                        "021C" => census.triads_021c += 1,
                        "111D" => census.triads_111d += 1,
                        "111U" => census.triads_111u += 1,
                        "030T" => census.triads_030t += 1,
                        "030C" => census.triads_030c += 1,
                        "201" => census.triads_201 += 1,
                        "120D" => census.triads_120d += 1,
                        "120U" => census.triads_120u += 1,
                        "120C" => census.triads_120c += 1,
                        "210" => census.triads_210 += 1,
                        "300" => census.triads_300 += 1,
                        _ => {}
                    }
                }
            }
        }

        Ok(AnalysisData::TriadicCensus(census))
    }

    fn k_core(&self, k: usize) -> Result<AnalysisData> {
        let mut core_numbers = HashMap::new();
        let mut remaining_nodes: HashSet<_> = self.graph.nodes.keys().cloned().collect();
        let mut current_degrees = HashMap::new();

        // Initialize degrees
        for node_id in &remaining_nodes {
            let degree = self.graph.adjacency_list.get(node_id).map_or(0, |adj| adj.len());
            current_degrees.insert(node_id.clone(), degree);
        }

        let mut current_k = 0;
        
        while !remaining_nodes.is_empty() {
            let mut to_remove = Vec::new();
            
            // Find nodes with degree < current_k
            for node_id in &remaining_nodes {
                if current_degrees[node_id] < current_k {
                    to_remove.push(node_id.clone());
                }
            }
            
            if to_remove.is_empty() {
                current_k += 1;
                continue;
            }
            
            // Remove nodes and update degrees
            for node_id in to_remove {
                remaining_nodes.remove(&node_id);
                core_numbers.insert(node_id.clone(), current_k.saturating_sub(1));
                
                // Update neighbor degrees
                if let Some(neighbors) = self.graph.adjacency_list.get(&node_id) {
                    for neighbor in neighbors {
                        if remaining_nodes.contains(neighbor) {
                            if let Some(degree) = current_degrees.get_mut(neighbor) {
                                *degree = degree.saturating_sub(1);
                            }
                        }
                    }
                }
            }
        }

        // Filter for k-core
        let k_core_nodes: HashMap<NodeId, f64> = core_numbers.into_iter()
            .filter_map(|(node, core)| if core >= k { Some((node, core as f64)) } else { None })
            .collect();

        Ok(AnalysisData::NodeScores(k_core_nodes))
    }

    fn centrality_evolution(&self) -> Result<AnalysisData> {
        // Mock implementation for centrality evolution over time
        let mut evolution_scores = HashMap::new();
        
        for node_id in self.graph.nodes.keys() {
            // Simulate evolution with random values
            evolution_scores.insert(node_id.clone(), 0.5); // Placeholder for rand::random()
        }

        Ok(AnalysisData::NodeScores(evolution_scores))
    }

    fn community_evolution(&self) -> Result<AnalysisData> {
        // Mock implementation for community evolution
        self.louvain_community_detection()
    }

    fn network_growth_analysis(&self) -> Result<AnalysisData> {
        // Mock implementation for network growth analysis
        Ok(AnalysisData::NetworkMetrics(NetworkMetrics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            density: 0.0,
            diameter: 0.0,
            radius: 0.0,
            average_path_length: 0.0,
            clustering_coefficient: 0.0,
            assortativity: 0.0,
            transitivity: 0.0,
            small_worldness: 0.0,
        }))
    }

    /// Helper methods
    fn single_source_shortest_path(&self, source: &NodeId) -> Result<HashMap<NodeId, f64>> {
        let mut distances = HashMap::new();
        let mut queue = VecDeque::new();
        
        distances.insert(source.clone(), 0.0);
        queue.push_back(source.clone());
        
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.graph.adjacency_list.get(&current) {
                for neighbor in neighbors {
                    if !distances.contains_key(neighbor) {
                        distances.insert(neighbor.clone(), distances[&current] + 1.0);
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        Ok(distances)
    }

    fn dijkstra_path(&self, source: &NodeId, target: &NodeId) -> Result<Path> {
        // Simplified Dijkstra implementation
        let mut distances = HashMap::new();
        let mut previous = HashMap::new();
        let mut visited = HashSet::new();
        
        distances.insert(source.clone(), 0.0);
        
        while !visited.contains(target) {
            let current = distances.iter()
                .filter(|(node, _)| !visited.contains(*node))
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(node, _)| node.clone());
            
            if let Some(current) = current {
                visited.insert(current.clone());
                
                if let Some(neighbors) = self.graph.adjacency_list.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            let edge_weight = self.graph.edges.values()
                                .find(|e| e.source == current && e.target == *neighbor)
                                .map_or(1.0, |e| e.weight);
                            
                            let new_distance = distances[&current] + edge_weight;
                            
                            if new_distance < *distances.get(neighbor).unwrap_or(&f64::INFINITY) {
                                distances.insert(neighbor.clone(), new_distance);
                                previous.insert(neighbor.clone(), current.clone());
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
        
        // Reconstruct path
        let mut path_nodes = Vec::new();
        let mut current = target.clone();
        
        while let Some(prev) = previous.get(&current) {
            path_nodes.push(current.clone());
            current = prev.clone();
        }
        path_nodes.push(source.clone());
        path_nodes.reverse();
        
        let path_length = path_nodes.len() as f64 - 1.0;
        Ok(Path {
            source: source.clone(),
            target: target.clone(),
            nodes: path_nodes,
            edges: Vec::new(),
            length: path_length,
            weight: *distances.get(target).unwrap_or(&f64::INFINITY),
        })
    }

    fn dfs_component(&self, start: &NodeId, visited: &mut HashSet<NodeId>) -> Vec<NodeId> {
        let mut component = Vec::new();
        let mut stack = vec![start.clone()];
        
        while let Some(node) = stack.pop() {
            if !visited.contains(&node) {
                visited.insert(node.clone());
                component.push(node.clone());
                
                if let Some(neighbors) = self.graph.adjacency_list.get(&node) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        component
    }

    fn tarjan_scc(
        &self,
        v: &NodeId,
        index: &mut usize,
        stack: &mut Vec<NodeId>,
        indices: &mut HashMap<NodeId, usize>,
        lowlinks: &mut HashMap<NodeId, usize>,
        on_stack: &mut HashSet<NodeId>,
        components: &mut Vec<Vec<NodeId>>,
    ) {
        indices.insert(v.clone(), *index);
        lowlinks.insert(v.clone(), *index);
        *index += 1;
        stack.push(v.clone());
        on_stack.insert(v.clone());

        if let Some(neighbors) = self.graph.adjacency_list.get(v) {
            for w in neighbors {
                if !indices.contains_key(w) {
                    self.tarjan_scc(w, index, stack, indices, lowlinks, on_stack, components);
                    lowlinks.insert(v.clone(), lowlinks[v].min(lowlinks[w]));
                } else if on_stack.contains(w) {
                    lowlinks.insert(v.clone(), lowlinks[v].min(indices[w]));
                }
            }
        }

        if lowlinks[v] == indices[v] {
            let mut component = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                component.push(w.clone());
                if w == *v {
                    break;
                }
            }
            components.push(component);
        }
    }

    fn find_articulation_points(
        &self,
        u: &NodeId,
        visited: &mut HashSet<NodeId>,
        discovery: &mut HashMap<NodeId, usize>,
        low: &mut HashMap<NodeId, usize>,
        parent: &mut HashMap<NodeId, Option<NodeId>>,
        articulation_points: &mut HashSet<NodeId>,
        time: &mut usize,
    ) {
        let mut children = 0;
        visited.insert(u.clone());
        discovery.insert(u.clone(), *time);
        low.insert(u.clone(), *time);
        *time += 1;

        if let Some(neighbors) = self.graph.adjacency_list.get(u) {
            for v in neighbors {
                if !visited.contains(v) {
                    children += 1;
                    parent.insert(v.clone(), Some(u.clone()));
                    self.find_articulation_points(v, visited, discovery, low, parent, articulation_points, time);

                    low.insert(u.clone(), low[u].min(low[v]));

                    if parent[u].is_none() && children > 1 {
                        articulation_points.insert(u.clone());
                    }

                    if parent[u].is_some() && low[v] >= discovery[u] {
                        articulation_points.insert(u.clone());
                    }
                } else if Some(v.clone()) != parent[u] {
                    low.insert(u.clone(), low[u].min(discovery[v]));
                }
            }
        }
    }

    fn find_bridges(
        &self,
        u: &NodeId,
        visited: &mut HashSet<NodeId>,
        discovery: &mut HashMap<NodeId, usize>,
        low: &mut HashMap<NodeId, usize>,
        parent: &mut HashMap<NodeId, Option<NodeId>>,
        bridges: &mut Vec<EdgeId>,
        time: &mut usize,
    ) {
        visited.insert(u.clone());
        discovery.insert(u.clone(), *time);
        low.insert(u.clone(), *time);
        *time += 1;

        if let Some(neighbors) = self.graph.adjacency_list.get(u) {
            for v in neighbors {
                if !visited.contains(v) {
                    parent.insert(v.clone(), Some(u.clone()));
                    self.find_bridges(v, visited, discovery, low, parent, bridges, time);

                    low.insert(u.clone(), low[u].min(low[v]));

                    if low[v] > discovery[u] {
                        // Find edge ID
                        for edge in self.graph.edges.values() {
                            if (edge.source == *u && edge.target == *v) || 
                               (!self.graph.is_directed && edge.source == *v && edge.target == *u) {
                                bridges.push(edge.id.clone());
                                break;
                            }
                        }
                    }
                } else if Some(v.clone()) != parent[u] {
                    low.insert(u.clone(), low[u].min(discovery[v]));
                }
            }
        }
    }

    fn find_set(&self, x: &NodeId, parent: &mut HashMap<NodeId, NodeId>) -> NodeId {
        if parent[x] != *x {
            let root = self.find_set(&parent[x].clone(), parent);
            parent.insert(x.clone(), root.clone());
            root
        } else {
            x.clone()
        }
    }

    fn union_sets(
        &self,
        x: &NodeId,
        y: &NodeId,
        parent: &mut HashMap<NodeId, NodeId>,
        rank: &mut HashMap<NodeId, usize>,
    ) {
        if rank[x] < rank[y] {
            parent.insert(x.clone(), y.clone());
        } else if rank[x] > rank[y] {
            parent.insert(y.clone(), x.clone());
        } else {
            parent.insert(y.clone(), x.clone());
            rank.insert(x.clone(), rank[x] + 1);
        }
    }

    fn local_clustering_coefficient(&self, node_id: &NodeId) -> f64 {
        if let Some(neighbors) = self.graph.adjacency_list.get(node_id) {
            let degree = neighbors.len();
            if degree < 2 {
                return 0.0;
            }

            let mut triangles = 0;
            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    if self.graph.adjacency_list.get(&neighbors[i])
                        .map_or(false, |adj| adj.contains(&neighbors[j])) {
                        triangles += 1;
                    }
                }
            }

            let possible_triangles = degree * (degree - 1) / 2;
            triangles as f64 / possible_triangles as f64
        } else {
            0.0
        }
    }

    fn calculate_modularity_gain(
        &self,
        node_id: &NodeId,
        from_community: usize,
        to_community: usize,
        node_community: &HashMap<NodeId, usize>,
    ) -> f64 {
        // Simplified modularity gain calculation
        let mut gain = 0.0;
        
        if let Some(neighbors) = self.graph.adjacency_list.get(node_id) {
            for neighbor in neighbors {
                let neighbor_community = node_community[neighbor];
                if neighbor_community == to_community {
                    gain += 1.0;
                }
                if neighbor_community == from_community {
                    gain -= 1.0;
                }
            }
        }
        
        gain
    }

    fn classify_triad(&self, a: &NodeId, b: &NodeId, c: &NodeId) -> String {
        let ab = self.has_edge(a, b);
        let ba = self.has_edge(b, a);
        let ac = self.has_edge(a, c);
        let ca = self.has_edge(c, a);
        let bc = self.has_edge(b, c);
        let cb = self.has_edge(c, b);

        let edges = [ab, ba, ac, ca, bc, cb];
        let edge_count = edges.iter().filter(|&&e| e).count();

        match edge_count {
            0 => "003".to_string(),
            1 => "012".to_string(),
            2 => "102".to_string(),
            3 => "021D".to_string(), // Simplified classification
            4 => "111D".to_string(),
            5 => "210".to_string(),
            6 => "300".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn has_edge(&self, from: &NodeId, to: &NodeId) -> bool {
        self.graph.adjacency_list.get(from)
            .map_or(false, |neighbors| neighbors.contains(to))
    }

    fn compute_network_statistics(&self) -> Result<NetworkStatistics> {
        let node_count = self.graph.nodes.len();
        let edge_count = self.graph.edges.len();
        
        let total_degree: usize = self.node_properties.values().map(|p| p.degree).sum();
        let average_degree = if node_count > 0 { total_degree as f64 / node_count as f64 } else { 0.0 };
        
        // Compute degree distribution
        let mut degree_counts = HashMap::new();
        for properties in self.node_properties.values() {
            *degree_counts.entry(properties.degree).or_insert(0) += 1;
        }
        let degree_distribution: Vec<_> = degree_counts.into_iter().collect();
        
        // Compute connected components
        let components_result = self.connected_components()?;
        let (connected_components, largest_component_size) = match components_result {
            AnalysisData::Components(components) => {
                let largest_size = components.iter().map(|c| c.nodes.len()).max().unwrap_or(0);
                (components.len(), largest_size)
            },
            _ => (0, 0),
        };
        
        // Compute graph density
        let max_edges = if self.graph.is_directed {
            node_count * (node_count - 1)
        } else {
            node_count * (node_count - 1) / 2
        };
        let graph_density = if max_edges > 0 { edge_count as f64 / max_edges as f64 } else { 0.0 };
        
        // Compute clustering coefficient
        let total_clustering: f64 = self.graph.nodes.keys()
            .map(|node| self.local_clustering_coefficient(node))
            .sum();
        let clustering_coefficient = total_clustering / node_count as f64;

        Ok(NetworkStatistics {
            node_count,
            edge_count,
            average_degree,
            degree_distribution,
            connected_components,
            largest_component_size,
            graph_density,
            clustering_coefficient,
            diameter: None,
            radius: None,
        })
    }
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
            is_directed: false,
            is_weighted: false,
        }
    }

    fn from_nodes_edges(nodes: Vec<Node>, edges: Vec<Edge>, is_directed: bool) -> Result<Self> {
        let mut graph = Self::new();
        graph.is_directed = is_directed;
        graph.is_weighted = edges.iter().any(|e| e.weight != 1.0);
        
        // Add nodes
        for node in nodes {
            graph.nodes.insert(node.id.clone(), node);
        }
        
        // Add edges and build adjacency lists
        for edge in edges {
            if !graph.nodes.contains_key(&edge.source) || !graph.nodes.contains_key(&edge.target) {
                return Err(anyhow!("Edge references non-existent node"));
            }
            
            graph.edges.insert(edge.id.clone(), edge.clone());
            
            // Add to adjacency list
            graph.adjacency_list.entry(edge.source.clone())
                .or_insert_with(Vec::new)
                .push(edge.target.clone());
            
            // Add to reverse adjacency list
            graph.reverse_adjacency_list.entry(edge.target.clone())
                .or_insert_with(Vec::new)
                .push(edge.source.clone());
            
            // For undirected graphs, add reverse edge
            if !is_directed {
                graph.adjacency_list.entry(edge.target.clone())
                    .or_insert_with(Vec::new)
                    .push(edge.source.clone());
                
                graph.reverse_adjacency_list.entry(edge.source.clone())
                    .or_insert_with(Vec::new)
                    .push(edge.target.clone());
            }
        }
        
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_analysis_engine() {
        let mut engine = GraphAnalysisEngine::new();
        
        let nodes = vec![
            Node {
                id: "A".to_string(),
                label: "Node A".to_string(),
                properties: HashMap::new(),
                position: Some((0.0, 0.0)),
            },
            Node {
                id: "B".to_string(),
                label: "Node B".to_string(),
                properties: HashMap::new(),
                position: Some((1.0, 0.0)),
            },
            Node {
                id: "C".to_string(),
                label: "Node C".to_string(),
                properties: HashMap::new(),
                position: Some((0.5, 1.0)),
            },
        ];
        
        let edges = vec![
            Edge {
                id: "AB".to_string(),
                source: "A".to_string(),
                target: "B".to_string(),
                weight: 1.0,
                label: "Edge AB".to_string(),
                properties: HashMap::new(),
            },
            Edge {
                id: "BC".to_string(),
                source: "B".to_string(),
                target: "C".to_string(),
                weight: 1.0,
                label: "Edge BC".to_string(),
                properties: HashMap::new(),
            },
            Edge {
                id: "CA".to_string(),
                source: "C".to_string(),
                target: "A".to_string(),
                weight: 1.0,
                label: "Edge CA".to_string(),
                properties: HashMap::new(),
            },
        ];
        
        engine.load_graph(nodes, edges, false).unwrap();
        
        let result = engine.analyze(GraphAnalysisType::DegreeCentrality).unwrap();
        
        if let AnalysisData::NodeScores(scores) = result.results {
            assert_eq!(scores.len(), 3);
            // Each node should have degree centrality of 1.0 (connected to all others)
            for score in scores.values() {
                assert!(*score > 0.0);
            }
        } else {
            panic!("Expected NodeScores result");
        }
    }
} 