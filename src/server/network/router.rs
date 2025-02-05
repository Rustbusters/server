use crate::RustBustersServer;
use log::info;
use std::collections::{HashMap, HashSet, VecDeque};
use wg_2024::network::NodeId;

impl RustBustersServer {
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
                    if !visited.contains(&neighbor) {
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
