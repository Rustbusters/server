mod router;

use crate::RustBustersServer;

impl RustBustersServer {
    pub fn discover_network(&self) {
        todo!()
    }

    //pub(crate) fn discover_network(&mut self) {
    //    // Generate a unique flood_id
    //    self.flood_id_counter += 1;
    //    let flood_id = self.flood_id_counter;
    //
    //    // Initialize the FloodRequest
    //    let flood_request = FloodRequest {
    //        flood_id,
    //        initiator_id: self.id,
    //        path_trace: vec![(self.id, self.server_type)],
    //    };
    //
    //    // Create the packet without routing header (it's ignored for FloodRequest)
    //    let packet = Packet {
    //        pack_type: PacketType::FloodRequest(flood_request),
    //        routing_header: SourceRoutingHeader {
    //            hop_index: 0,
    //            hops: vec![],
    //        },
    //        session_id: 0,
    //    };
    //
    //    for (&neighbor_id, neighbor_sender) in &self.packet_send {
    //        info!(
    //            "Node {}: Sending FloodRequest to {} with flood_id {}",
    //            self.id, neighbor_id, flood_id
    //        );
    //        let _ = neighbor_sender.send(packet.clone());
    //    }
    //}

}
