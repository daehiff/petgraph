use std::collections::HashMap;

use std::hash::Hash;

use crate::algo::{BoundedMeasure, NegativeCycle};
use crate::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, NodeCompactIndexable,
};

#[allow(clippy::type_complexity, clippy::needless_range_loop)]
/// \[Generic\] [Floyd–Warshall algorithm](https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm) is an algorithm for all pairs shortest path problem
///
/// Compute distance of shortest paths in a weighted graph with positive or negative edge weights (but with no negative cycles)
///
/// # Arguments
/// * `graph`: graph with no negative cycle
/// * `edge_cost`: closure that returns cost of a particular edge
///
/// # Returns
/// * `Ok`: (if graph contains no negative cycle) a hashmap containing all pairs shortest paths
/// * `Err`: if graph contains negative cycle.
///
/// # Examples
/// ```rust
/// use petgraph::{prelude::*, Graph, Directed};
/// use petgraph::algo::floyd_warshall;
/// use std::collections::HashMap;
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[
///    (a, b),
///    (a, c),
///    (a, d),
///    (b, c),
///    (b, d),
///    (c, d)
/// ]);
///
/// let weight_map: HashMap<(NodeIndex, NodeIndex), i32> = [
///    ((a, a), 0), ((a, b), 1), ((a, c), 4), ((a, d), 10),
///    ((b, b), 0), ((b, c), 2), ((b, d), 2),
///    ((c, c), 0), ((c, d), 2)
/// ].iter().cloned().collect();
/// //     ----- b --------
/// //    |      ^         | 2
/// //    |    1 |    4    v
/// //  2 |      a ------> c
/// //    |   10 |         | 2
/// //    |      v         v
/// //     --->  d <-------
///
/// let inf = std::i32::MAX;
/// let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
///    ((a, a), 0), ((a, b), 1), ((a, c), 3), ((a, d), 3),
///    ((b, a), inf), ((b, b), 0), ((b, c), 2), ((b, d), 2),
///    ((c, a), inf), ((c, b), inf), ((c, c), 0), ((c, d), 2),
///    ((d, a), inf), ((d, b), inf), ((d, c), inf), ((d, d), 0),
/// ].iter().cloned().collect();
///
///
/// let res = floyd_warshall(&graph, |edge| {
///     if let Some(weight) = weight_map.get(&(edge.source(), edge.target())) {
///         *weight
///     } else {
///         inf
///     }
/// }).unwrap();
///
/// let nodes = [a, b, c, d];
/// for node1 in &nodes {
///     for node2 in &nodes {
///         assert_eq!(res.get(&(*node1, *node2)).unwrap(), expected_res.get(&(*node1, *node2)).unwrap());
///     }
/// }
/// ```
pub fn floyd_warshall<G, F, K>(
    graph: G,
    mut edge_cost: F,
) -> Result<HashMap<(G::NodeId, G::NodeId), K>, NegativeCycle>
where
    G: NodeCompactIndexable + IntoEdgeReferences + IntoNodeIdentifiers + GraphProp,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: BoundedMeasure + Copy,
{
    let num_of_nodes = graph.node_count();

    // |V|x|V| matrix
    let mut dist = vec![vec![K::max(); num_of_nodes]; num_of_nodes];

    // init distances of paths with no intermediate nodes
    for edge in graph.edge_references() {
        dist[graph.to_index(edge.source())][graph.to_index(edge.target())] = edge_cost(edge);
        if !graph.is_directed() {
            dist[graph.to_index(edge.target())][graph.to_index(edge.source())] = edge_cost(edge);
        }
    }

    // distance of each node to itself is 0(default value)
    for node in graph.node_identifiers() {
        dist[graph.to_index(node)][graph.to_index(node)] = K::default();
    }

    for k in 0..num_of_nodes {
        for i in 0..num_of_nodes {
            for j in 0..num_of_nodes {
                let (result, overflow) = dist[i][k].overflowing_add(dist[k][j]);
                if !overflow && dist[i][j] > result {
                    dist[i][j] = result;
                }
            }
        }
    }

    // value less than 0(default value) indicates a negative cycle
    for i in 0..num_of_nodes {
        if dist[i][i] < K::default() {
            return Err(NegativeCycle(()));
        }
    }

    let mut distance_map: HashMap<(G::NodeId, G::NodeId), K> =
        HashMap::with_capacity(num_of_nodes * num_of_nodes);

    for i in 0..num_of_nodes {
        for j in 0..num_of_nodes {
            distance_map.insert((graph.from_index(i), graph.from_index(j)), dist[i][j]);
        }
    }

    Ok(distance_map)
}

fn path_from_shortest_path_tree<G>(
    graph: G,
    shortest_path_tree: &[Vec<Option<usize>>],
    edge: (G::NodeId, G::NodeId),
) -> Vec<(G::NodeId, G::NodeId)>
where
    G: NodeCompactIndexable + IntoEdgeReferences + IntoNodeIdentifiers + GraphProp,
    G::NodeId: Eq + Hash,
{
    let (source, target) = edge;
    let u = graph.to_index(source);
    let mut v = graph.to_index(target);
    let mut v_id = target;

    if shortest_path_tree[u][v].is_none() {
        return Vec::new();
    }
    let mut path = Vec::new();
    while u != v {
        if let Some(new_v) = shortest_path_tree[u][v] {
            path.push((graph.from_index(new_v), v_id));
            v = new_v;
            v_id = graph.from_index(new_v);
        }
    }

    path.reverse();

    path
}

#[allow(clippy::type_complexity, clippy::needless_range_loop)]
/// \[Generic\] [Floyd–Warshall algorithm](https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm) is an algorithm for all pairs shortest path problem
///
/// Compute all pairs shortest paths in a weighted graph with positive or negative edge weights (but with no negative cycles).
/// Returns HashMap of shortest path lengths. Additionally, returns HashMap of intermediate nodes along shortest path for indicated edges.
///
/// # Arguments
/// * `graph`: graph with no negative cycle
/// * `edge_cost`: closure that returns cost of a particular edge
///
/// # Returns
/// * `Ok`: (if graph contains no negative cycle) a hashmap containing all pairs shortest path distances and a hashmap for all pairs shortest paths
/// * `Err`: if graph contains negative cycle.
///
/// # Examples
/// ```rust
/// use petgraph::{prelude::*, Graph, Directed};
/// use petgraph::algo::floyd_warshall_path;
/// use std::collections::HashMap;
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[
///    (a, b),
///    (a, c),
///    (a, d),
///    (b, c),
///    (b, d),
///    (c, d)
/// ]);
///
/// let weight_map: HashMap<(NodeIndex, NodeIndex), i32> = [
///    ((a, a), 0), ((a, b), 1), ((a, c), 4), ((a, d), 10),
///    ((b, b), 0), ((b, c), 2), ((b, d), 2),
///    ((c, c), 0), ((c, d), 2)
/// ].iter().cloned().collect();
/// //     ----- b --------
/// //    |      ^         | 2
/// //    |    1 |    4    v
/// //  2 |      a ------> c
/// //    |   10 |         | 2
/// //    |      v         v
/// //     --->  d <-------
///
/// let inf = std::i32::MAX;
/// let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
///    ((a, a), 0), ((a, b), 1), ((a, c), 3), ((a, d), 3),
///    ((b, a), inf), ((b, b), 0), ((b, c), 2), ((b, d), 2),
///    ((c, a), inf), ((c, b), inf), ((c, c), 0), ((c, d), 2),
///    ((d, a), inf), ((d, b), inf), ((d, c), inf), ((d, d), 0),
/// ].iter().cloned().collect();
///
///
/// let (res, paths) = floyd_warshall_path(&graph, Some([(a,c)].iter().cloned().collect()), |edge| {
///     if let Some(weight) = weight_map.get(&(edge.source(), edge.target())) {
///         *weight
///     } else {
///         inf
///     }
/// }).unwrap();
///
/// assert_eq!(paths.get(&(a, c)), Some(vec![(a, b), (b, c)].as_ref()));
///
/// let nodes = [a, b, c, d];
/// for node1 in &nodes {
///     for node2 in &nodes {
///         assert_eq!(res.get(&(*node1, *node2)).unwrap(), expected_res.get(&(*node1, *node2)).unwrap());
///     }
/// }
///
/// ```
pub fn floyd_warshall_path<G, F, K>(
    graph: G,
    required_paths: Option<Vec<(G::NodeId, G::NodeId)>>,
    mut edge_cost: F,
) -> Result<
    (
        HashMap<(G::NodeId, G::NodeId), K>,
        HashMap<(G::NodeId, G::NodeId), Vec<(G::NodeId, G::NodeId)>>,
    ),
    NegativeCycle,
>
where
    G: NodeCompactIndexable + IntoEdgeReferences + IntoNodeIdentifiers + GraphProp,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: BoundedMeasure + Copy,
{
    let num_of_nodes = graph.node_count();

    // |V|x|V| matrix
    let mut dist = vec![vec![K::max(); num_of_nodes]; num_of_nodes];
    // `prev[source][target]` holds the penultimate vertex on path from `source` to `target`, except `prev[source][source]`, which always stores `source`.
    let mut prev: Vec<Vec<Option<usize>>> = vec![vec![None; num_of_nodes]; num_of_nodes];

    // init distances of paths with no intermediate nodes
    for edge in graph.edge_references() {
        dist[graph.to_index(edge.source())][graph.to_index(edge.target())] = edge_cost(edge);
        prev[graph.to_index(edge.source())][graph.to_index(edge.target())] =
            Some(graph.to_index(edge.source()));
        if !graph.is_directed() {
            dist[graph.to_index(edge.target())][graph.to_index(edge.source())] = edge_cost(edge);
            prev[graph.to_index(edge.target())][graph.to_index(edge.source())] =
                Some(graph.to_index(edge.target()));
        }
    }

    // distance of each node to itself is 0(default value)
    for node in graph.node_identifiers() {
        dist[graph.to_index(node)][graph.to_index(node)] = K::default();
        prev[graph.to_index(node)][graph.to_index(node)] = Some(graph.to_index(node));
    }

    for k in 0..num_of_nodes {
        for i in 0..num_of_nodes {
            for j in 0..num_of_nodes {
                let (result, overflow) = dist[i][k].overflowing_add(dist[k][j]);
                if !overflow && dist[i][j] > result {
                    dist[i][j] = result;
                    prev[i][j] = prev[k][j];
                }
            }
        }
    }
    let mut distance_map = HashMap::with_capacity(num_of_nodes * num_of_nodes);

    for i in 0..num_of_nodes {
        for j in 0..num_of_nodes {
            distance_map.insert((graph.from_index(i), graph.from_index(j)), dist[i][j]);
        }
    }

    let mut path_map = HashMap::new();
    if let Some(edges) = required_paths {
        for edge in edges {
            path_map.insert(edge, path_from_shortest_path_tree(graph, &prev, edge));
        }
    }

    Ok((distance_map, path_map))
}
