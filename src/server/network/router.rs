use crate::RustBustersServer;
use log::info;
use std::collections::{HashMap, HashSet, VecDeque};
use wg_2024::{network::NodeId, packet::NodeType};

impl RustBustersServer {
    /// Finds the shortest path (number of edges) from the current server to the specified destination using Breadth-First Search (BFS).
    ///
    /// ### Parameters
    /// - `destination_id`: The ID of the node to which a route is being searched.
    ///
    /// ### Returns
    /// - `Some(Vec<NodeId>)`: A vector representing the shortest path from `self.id` to `destination_id`, if found.
    /// - `None`: If no valid route exists.
    ///
    /// ### Algorithm
    /// Uses BFS to explore the network graph and determine the shortest path.
    pub(crate) fn find_route(&self, destination_id: NodeId) -> Option<Vec<NodeId>> {
        // Simple BFS to find the shortest path
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors = HashMap::new();

        visited.insert(self.id);
        queue.push_back(self.id);

        while let Some(current) = queue.remove(0) {
            if current == destination_id {
                // Build the path from self.id to destination_id
                let mut path = vec![destination_id];
                let mut node = destination_id;
                while node != self.id {
                    if let Some(&pred) = predecessors.get(&node) {
                        path.push(pred);
                        node = pred;
                    } else {
                        break;
                    }
                }
                path.reverse();
                info!(
                    "Server {}: Found route to {}: {:?}",
                    self.id, destination_id, path
                );
                return Some(path);
            }

            if let Some(neighbors) = self.topology.get(&current) {
                for &neighbor in neighbors {
                    let neighbor_type = self
                        .known_node_types
                        .get(&neighbor)
                        .expect("No neighbor found")
                        .clone();
                    if !visited.contains(&neighbor)
                        && (neighbor_type == NodeType::Drone
                            || (neighbor_type == NodeType::Client && neighbor == destination_id))
                    // Check if path contains only drone nodes or client is the final destination
                    {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                        predecessors.insert(neighbor, current);
                    }
                }
            }
        }

        None // No route found
    }
}
