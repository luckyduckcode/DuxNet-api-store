//! # libp2p P2P Node
//!
//! This module implements a robust, extensible libp2p node for the DuxNet platform.
//! It supports peer discovery, secure transport, multiplexing, and pubsub messaging.
//!
//! ## Features
//! - Peer discovery (mDNS)
//! - Secure transport (Noise)
//! - Multiplexing (Yamux)
//! - Pubsub messaging (Gossipsub)
//! - Extensible for custom protocols (task, reputation, etc.)
//!
//! ## Integration
//! - Route incoming messages to core modules (task engine, DHT, reputation, etc.)
//! - Use for all P2P communication in the platform

use libp2p::{
    identity, PeerId, Multiaddr, Swarm, swarm::SwarmEvent,
    noise, yamux, tcp, Transport, core::upgrade, mplex, gossipsub::{self, Gossipsub, GossipsubEvent, MessageAuthenticity, IdentTopic},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    futures::StreamExt,
};
use std::error::Error;
use tokio::sync::mpsc;
use crate::core::data_structures::NetworkMessage;

pub struct DuxP2PNode {
    pub peer_id: PeerId,
    pub swarm: Swarm<Gossipsub>,
    pub task_tx: Option<mpsc::UnboundedSender<crate::core::data_structures::Task>>,
    pub service_tx: Option<mpsc::UnboundedSender<crate::core::data_structures::ServiceMetadata>>,
    pub rep_tx: Option<mpsc::UnboundedSender<crate::core::data_structures::ReputationAttestation>>,
    pub msg_tx: Option<mpsc::UnboundedSender<crate::core::data_structures::Message>>,
}

impl DuxP2PNode {
    pub async fn new_with_channels(
        task_tx: mpsc::UnboundedSender<crate::core::data_structures::Task>,
        service_tx: mpsc::UnboundedSender<crate::core::data_structures::ServiceMetadata>,
        rep_tx: mpsc::UnboundedSender<crate::core::data_structures::ReputationAttestation>,
        msg_tx: mpsc::UnboundedSender<crate::core::data_structures::Message>,
    ) -> Result<Self, Box<dyn Error>> {
        // Generate a keypair for this node
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());

        // Set up transport (TCP + Noise + Yamux)
        let transport = tcp::TokioTcpTransport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&id_keys).unwrap())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        // Set up Gossipsub (pubsub)
        let gossipsub_config = gossipsub::GossipsubConfig::default();
        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        )?;
        // Subscribe to a default topic (can add more)
        let topic = IdentTopic::new("duxnet-main");
        gossipsub.subscribe(&topic)?;

        // Set up mDNS for peer discovery
        let mdns = Mdns::new(MdnsConfig::default()).await?;

        // Build the swarm
        let mut swarm = Swarm::with_tokio_executor(transport, gossipsub, peer_id.clone());
        Swarm::behaviour_mut(&mut swarm).add_explicit_peer(&peer_id); // Add self for demo
        swarm.behaviour_mut().subscribe(&topic)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self {
            peer_id,
            swarm,
            task_tx: Some(task_tx),
            service_tx: Some(service_tx),
            rep_tx: Some(rep_tx),
            msg_tx: Some(msg_tx),
        })
    }

    /// Start the P2P event loop
    pub async fn start(mut self, mut msg_rx: mpsc::UnboundedReceiver<Vec<u8>>) -> Result<(), Box<dyn Error>> {
        println!("libp2p node started with PeerId: {}", self.peer_id);
        loop {
            tokio::select! {
                Some(msg) = msg_rx.recv() => {
                    // Publish outgoing messages to the network
                    let topic = IdentTopic::new("duxnet-main");
                    self.swarm.behaviour_mut().publish(topic, msg)?;
                }
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(GossipsubEvent::Message { message, .. }) => {
                            // Handle incoming pubsub messages
                            println!("Received: {:?}", message);
                            // --- Message Routing ---
                            if let Ok(network_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                                match network_msg {
                                    NetworkMessage::TaskSubmission(task) => {
                                        if let Some(tx) = &self.task_tx { let _ = tx.send(task); }
                                    }
                                    NetworkMessage::ServiceAnnouncement(service) => {
                                        if let Some(tx) = &self.service_tx { let _ = tx.send(service); }
                                    }
                                    NetworkMessage::ReputationAttestation(attestation) => {
                                        if let Some(tx) = &self.rep_tx { let _ = tx.send(attestation); }
                                    }
                                    NetworkMessage::DirectMessage(msg) => {
                                        if let Some(tx) = &self.msg_tx { let _ = tx.send(msg); }
                                    }
                                    // ... handle other message types ...
                                    _ => {}
                                }
                            }
                        }
                        SwarmEvent::Behaviour(GossipsubEvent::Subscribed { peer_id, topic }) => {
                            println!("Peer {:?} subscribed to {:?}", peer_id, topic);
                        }
                        SwarmEvent::Behaviour(GossipsubEvent::Unsubscribed { peer_id, topic }) => {
                            println!("Peer {:?} unsubscribed from {:?}", peer_id, topic);
                        }
                        SwarmEvent::Behaviour(GossipsubEvent::GossipsubNotSupported { peer_id }) => {
                            println!("Peer {:?} does not support gossipsub", peer_id);
                        }
                        SwarmEvent::Behaviour(_) => {}
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            println!("Connected to peer: {:?}", peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            println!("Disconnected from peer: {:?}", peer_id);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
// --- To extend: Add new NetworkMessage variants and route them here ---
// - Add custom protocols for task submission, reputation, escrow, etc.
// - Integrate with core modules via channels or callbacks
// - Add more topics for different message types 