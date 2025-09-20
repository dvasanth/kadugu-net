use crate::config::Config;
use anyhow::Result;
use async_compat::Compat;
use futures::stream::StreamExt;
use libp2p::{
    dcutr, identify, identity::Keypair, multiaddr::Protocol, noise, relay, swarm::NetworkBehaviour,
    tcp, yamux, Multiaddr, StreamProtocol,
};
use libp2p_stream as stream;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

pub struct PortForwardingServer {
    server_addr: SocketAddr,
    accepted_peer_ids: Vec<String>,
    config: Config,
    key_pair: Keypair,
    stop_sender: broadcast::Sender<()>,
    join_handle: Option<thread::JoinHandle<()>>,
    runtime: Arc<Runtime>,
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    stream: stream::Behaviour,
    relay_client: relay::client::Behaviour,
    dcutr: dcutr::Behaviour,
}

impl Drop for PortForwardingServer {
    fn drop(&mut self) {
        tracing::warn!("PortForwardingServer is being dropped!");
    }
}

impl PortForwardingServer {
    pub fn new(server_addr: SocketAddr, accepted_peer_ids: Vec<String>, config: Config) -> Self {
        let key_pair = Keypair::generate_ed25519();
        let (stop_sender, _) = broadcast::channel(1);
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
        Self {
            server_addr,
            accepted_peer_ids,
            config,
            key_pair: key_pair.clone(),
            stop_sender,
            join_handle: None,
            runtime,
        }
    }

    /// Returns the peer ID as a string
    pub fn peer_id(&self) -> String {
        self.key_pair.public().to_peer_id().to_string()
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.stop_sender.send(())?;
        if let Some(handle) = self.join_handle.take() {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Failed to join server thread"))?;
        }
        tracing::info!("Server shutdown completed");
        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let server_addr = self.server_addr;
        let accepted_peer_ids = self.accepted_peer_ids.clone();
        let config = self.config.clone();
        let key_pair = self.key_pair.clone();
        let stop_sender_clone = self.stop_sender.clone();
        let stop_receiver = stop_sender_clone.subscribe();
        let runtime = self.runtime.clone();

        // Spawn the server on the provided runtime
        let join_handle = thread::spawn(move || {
            // Run the server in the new runtime
            runtime.block_on(async move {
                if let Err(e) = Self::start_internal(
                    server_addr,
                    accepted_peer_ids,
                    config,
                    key_pair,
                    stop_receiver,
                )
                .await
                {
                    tracing::error!("Server error: {}", e);
                }
            });
        });

        self.join_handle = Some(join_handle);
        Ok(())
    }

    async fn start_internal(
        server_addr: SocketAddr,
        accepted_peer_ids: Vec<String>,
        config: Config,
        key_pair: Keypair,
        stop_receiver: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key_pair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_dns()?
            .with_relay_client(noise::Config::new, yamux::Config::default)?
            .with_behaviour(|key_pair, relay_behaviour| Behaviour {
                stream: stream::Behaviour::new(),
                identify: identify::Behaviour::new(
                    identify::Config::new("/proxy/0.0.1".to_string(), key_pair.public())
                        .with_agent_version(config.proxy_agent.clone()),
                ),
                relay_client: relay_behaviour,
                dcutr: dcutr::Behaviour::new(key_pair.public().to_peer_id()),
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(10)))
            .build();

        let relay_address: Multiaddr = config.relay_address.parse().expect("Invalid relay address");

        swarm.listen_on("/ip4/0.0.0.0/udp/12007/quic-v1".parse()?)?;
        swarm.listen_on("/ip6/::/udp/12007/quic-v1".parse()?)?;
        swarm.dial(relay_address.clone())?;

        let stop_receiver = stop_receiver;
        const PROXY_PROTOCOL: &str = "/proxy";
        let incoming_streams = swarm
            .behaviour()
            .stream
            .new_control()
            .accept(StreamProtocol::new(PROXY_PROTOCOL))?;

        let accepted_peer_ids = accepted_peer_ids.clone();
        let server_addr = server_addr; // Clone the listen_addr

        // Create a new stop receiver for the incoming streams handler
        let streams_stop_receiver = stop_receiver.resubscribe();

        tokio::spawn(async move {
            PortForwardingServer::handle_incoming_streams(
                incoming_streams,
                accepted_peer_ids,
                server_addr,
                streams_stop_receiver,
            )
            .await;
        });

        let mut relay_reservation_complete = false;

        // Poll the swarm to make progress.
        let mut swarm_stop_receiver = stop_receiver.resubscribe();
        loop {
            let result: Result<_, anyhow::Error> = {
                tokio::select! {
                    _ = swarm_stop_receiver.recv() => {
                        tracing::info!("Received stop signal, shutting down server");
                        break Ok(());
                    }
                    event = swarm.next() => {
                        match event {
                            Some(libp2p::swarm::SwarmEvent::ExternalAddrExpired { .. }) => {
                                relay_reservation_complete = false;
                                Ok(())
                            }
                            Some(libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::RelayClient(
                                relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
                            ))) => {
                                tracing::info!("Reservation with relay {:?} completed ", relay_peer_id);
                                relay_reservation_complete = true;
                                Ok(())
                            }
                            Some(libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::Identify(
                                identify::Event::Received { .. },
                            ))) => {
                                if !relay_reservation_complete {
                                    swarm.listen_on(relay_address.clone().with(Protocol::P2pCircuit))?;
                                }
                                Ok(())
                            }
                            Some(event) => {
                                tracing::trace!(?event);
                                Ok(())
                            }
                            None => {
                                tracing::info!("Swarm stream ended");
                                Ok(())
                            }
                        }
                    }
                }
            };

            // Return early if there was an error
            // If there's an error (including our stop signal), return it
            if let Err(e) = result {
                // If it's our stop signal, return Ok(())
                if e.to_string() == "Stop signal received" {
                    return Ok(());
                }
                // Otherwise, return the actual error
                return Err(e);
            }
        }
    }

    async fn handle_incoming_streams(
        mut incoming_streams: stream::IncomingStreams,
        accepted_peer_ids: Vec<String>,
        server_addr: SocketAddr,
        mut stop_receiver: broadcast::Receiver<()>,
    ) {
        loop {
            let next_stream = incoming_streams.next();
            tokio::select! {
                _ = stop_receiver.recv() => {
                    tracing::info!("Received stop signal in handle_incoming_streams");
                    return;
                }
                stream = next_stream => {
                    let (peer, p2p_stream) = match stream {
                        Some(s) => s,
                        None => break, // No more incoming streams
                    };

                    let peer_id_str = peer.to_string();
                    let mut is_accepted = true;
                    for accepted_id in &accepted_peer_ids {
                        if accepted_id.contains(&peer_id_str) {
                            is_accepted = true;
                            break;
                        }
                        tracing::info!("Accepted peer ID: {} didn't match with peer ID: {}", accepted_id, peer_id_str);
                        is_accepted = false;
                    }

                    // If no accepted peer IDs are specified, accept all peers
                    if accepted_peer_ids.is_empty() {
                        is_accepted = true;
                    }

                    // Check if peer ID is in the allowed vector of strings
                    if !is_accepted {
                        tracing::warn!("Unauthorized peer: {}", peer_id_str);
                        continue;
                    }

                    let target_addr = server_addr;
                    tokio::spawn(async move {
                        let mut app_stream = match TcpStream::connect(target_addr).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                tracing::error!("Failed to connect to target: {}", e);
                                return;
                            }
                        };

                        let _ = app_stream.set_nodelay(true);
                        let mut p2p_tokio_stream = Compat::new(p2p_stream);

                        if let Err(e) = tokio::io::copy_bidirectional(&mut p2p_tokio_stream, &mut app_stream).await {
                            tracing::info!("Error copying data: {}", e);
                        }
                    });
                }
            }
        }
    }
}
