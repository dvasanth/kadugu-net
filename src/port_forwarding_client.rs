use libp2p::{
    Multiaddr,
    PeerId,
    identify,
    multiaddr::Protocol,
    relay,
    swarm::{
        NetworkBehaviour,
    },
    tcp,
    yamux,
    dcutr,
    noise,
};
use tokio::sync::broadcast;
use std::net::SocketAddr;
use std::time::Duration;

use async_compat::Compat;
use futures::stream::StreamExt;
use libp2p_stream as stream;
use tokio::net::{TcpListener};
use crate::config::Config;

pub struct PortForwardingClient {
    server_id: PeerId,
    local_forward_addr: SocketAddr,
    config: Config,
    stop_sender: broadcast::Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
    runtime: std::sync::Arc<tokio::runtime::Runtime>,
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    stream: stream::Behaviour,
    relay_client: relay::client::Behaviour,
    dcutr: dcutr::Behaviour,
}

impl PortForwardingClient {
    pub fn new(server_id: PeerId, local_forward_addr: SocketAddr, config: Config, runtime: std::sync::Arc<tokio::runtime::Runtime>) -> Self {
        let (stop_sender, _) = broadcast::channel(1);
        Self {
            server_id,
            local_forward_addr,
            config,
            stop_sender,
            join_handle: None,
            runtime,
        }
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        tracing::info!("Sending stop signal to client");
        let _ = self.stop_sender.send(());
        
        // Take the join handle to wait for the task to complete
        if let Some(handle) = self.join_handle.take() {
            handle.join().map_err(|e| anyhow::anyhow!("Failed to join client task: {:?}", e))?;
        }
        Ok(())
    }
    
    /// Start the client and store the join handle internally
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.join_handle.is_some() {
            return Err(anyhow::anyhow!("Client is already running"));
        }
        
        let server_id = self.server_id.clone();
        let local_forward_addr = self.local_forward_addr;
        let config = self.config.clone();
        let stop_receiver = self.stop_sender.subscribe();
        
        // Get a reference to the runtime
        let runtime = self.runtime.clone();
        
        // Create oneshot channel for dial complete notification
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        // Spawn the client on the provided runtime
        let join_handle = std::thread::spawn(move || {
            // Run the client in the new runtime
            runtime.block_on(async move {
                if let Err(e) = PortForwardingClient::start_internal(server_id, local_forward_addr, config, stop_receiver, tx).await {
                    tracing::error!("Client error: {}", e);
                }
            });
        });
        // ✅ Wait for signal with timeout (blocking)
        let start = std::time::Instant::now();
        let mut connected = false;
        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(_) = rx.try_recv() {
                tracing::info!("Client connected successfully");
                connected = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        if !connected {
            tracing::warn!("Timed out waiting for client to connect");
        }
        self.join_handle = Some(join_handle);
        Ok(())
    }

    async fn start_internal(
        server_id: PeerId,
        local_forward_addr: SocketAddr,
        config: Config,
        stop_receiver: broadcast::Receiver<()>,
        dial_complete_sender: tokio::sync::oneshot::Sender<()>,
    ) -> anyhow::Result<()> {
         let key_pair = libp2p::identity::Keypair::generate_ed25519();
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key_pair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                libp2p::noise::Config::new,
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
    
        let relay_address: Multiaddr = config.relay_address
            .parse()
            .expect("Invalid relay address");
    

        swarm.listen_on("/ip4/0.0.0.0/udp/12008/quic-v1".parse()?)?;
        swarm.listen_on("/ip6/::/udp/12008/quic-v1".parse()?)?;
        swarm.dial(relay_address.clone())?;
        // Poll the swarm to make progress.
        let mut sharer_dial_complete = false;
        let mut stop_receiver = stop_receiver;
        let mut connection_handle: Option<tokio::task::JoinHandle<()>> = None;
        let mut dial_complete_sender = Some(dial_complete_sender);        
        loop {
            tokio::select! {
                _ = stop_receiver.recv() => {
                    tracing::info!("Stopping port forwarding client");
                    // Wait for the connection handler to complete if it exists
                    if let Some(h) = connection_handle.take() {
                        if let Err(e) = h.await {
                            tracing::error!("Port forwarding connection handler failed: {}", e);
                        }
                    }
                    drop(swarm);                    
                    // Let the close operations complete
                    //tokio::time::sleep(Duration::from_millis(100)).await;
                    return Ok(());
                }
                event = swarm.next() => {
                    let event = event.expect("never terminates");
                    match event {
                        libp2p::swarm::SwarmEvent::ExternalAddrExpired { .. } => {
                            // Ignore external address expiration
                        }
                        libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::RelayClient(
                            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
                        )) => {
                            tracing::info!("Client Reservation with relay {:?} completed ", relay_peer_id);
                        }
                        libp2p::swarm::SwarmEvent::OutgoingConnectionError {
                            connection_id: _,
                            peer_id,
                            ..
                        } => {
                            if peer_id == Some(server_id) {
                                swarm.dial(relay_address.clone())?;
                            }
                        }
                        libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::Identify(
                            identify::Event::Received { .. },
                        )) => {
                            if sharer_dial_complete {
                                continue;
                            }
                            swarm
                                .dial(
                                    relay_address
                                        .clone()
                                        .with(Protocol::P2pCircuit)
                                        .with(Protocol::P2p(server_id)),
                                )?;
                            sharer_dial_complete = true;

                            // Notify once
                            if let Some(sender) = dial_complete_sender.take() {
                                let _ = sender.send(());
                            }
                            connection_handle = Some(tokio::spawn(Self::port_forward_connection_handler(
                                server_id,
                                swarm.behaviour().stream.new_control(),
                                local_forward_addr,
                                config.clone(),
                                stop_receiver.resubscribe(),
                            )));
                            
  
                        }
                        event => tracing::trace!(?event),
                    }
                }
        }
    }
        

    }
    /// A very simple, `async fn`-based connection handler for client side listening address.
    pub async fn port_forward_connection_handler(
        peer: PeerId,
        mut control: stream::Control,
        local_forward_addr: SocketAddr,
        config: Config,
        mut stop_receiver: broadcast::Receiver<()>
    ) {
        let listener = TcpListener::bind(local_forward_addr).await.unwrap();
        loop {
            tokio::select! {
                _ = stop_receiver.recv() => {
                    tracing::info!("Stopping port forwarding");
                    break;
                }
                result = listener.accept() => {
                    let (app_stream, _) = result.unwrap();
                    let _ = app_stream.set_nodelay(true);

                    let p2p_stream = match control.open_stream(peer, config.proxy_protocol.clone()).await {
                        Ok(stream) => stream,
                        Err(error @ stream::OpenStreamError::UnsupportedProtocol(_)) => {
                            tracing::info!(%peer, %error);
                            continue;
                        }
                        Err(error) => {
                            tracing::info!(%peer, %error);
                            continue;
                        }
                    };

                     tokio::spawn(async move {
                        tracing::info!("Accepted new connection from local");
            
                        let mut p2p_tokio_stream = Compat::new(p2p_stream);
                        let mut app_stream = app_stream;
            
                        let (from_p2p, from_app) =
                            match tokio::io::copy_bidirectional(&mut p2p_tokio_stream, &mut app_stream).await {
                                Ok((from_p2p, from_app)) => (from_p2p, from_app),
                                Err(error) => {
                                    // Handle the error
                                    // For now, let's just print it
                                    tracing::info!("Error copying data from app to p2p stream: {:?}", error);
                                    return;
                                }
                            };
                        tracing::info!(
                            "App wrote {} bytes and received {} bytes",
                            from_app,
                            from_p2p
                        );
                    });
                }
            }
        }
    }

}