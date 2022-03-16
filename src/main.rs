#![allow(unused)]
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
mod tcp_listener;
use tcp_listener::TcpServer;
use tokio::{net::UdpSocket, sync::mpsc};
use webrtc_ice::{
    agent::{agent_config::AgentConfig, Agent},
    candidate::Candidate,
    network_type::NetworkType,
    state::ConnectionState,
    udp_mux::{UDPMuxDefault, UDPMuxParams},
    udp_network::UDPNetwork,
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let is_controlling = false;
    // let use_mux = false;

    // let remote_port = 4000;

    // let ice_agent = create_ice_agent(is_controlling).await.unwrap();
    // ice_agent.on_candidate(handle_candidate()).await;
    // let (ice_done_tx, mut ice_done_rx) = mpsc::channel::<()>(1);

    // ice_agent
    //     .on_connection_state_change(Box::new(move |c: ConnectionState| {
    //         println!("ICE Connection State has changed: {}", c);
    //         if c == ConnectionState::Failed {
    //             let _ = ice_done_tx.try_send(());
    //         }
    //         Box::pin(async move {})
    //     }))
    //     .await;
    // // Get the local auth details and send to remote peer
    // let (local_ufrag, local_pwd) = ice_agent.get_local_user_credentials().await;

    // println!("posting remoteAuth with {}:{}", local_ufrag, local_pwd);
    // ice_agent.gather_candidates().await?;
    // println!("Connecting...");
    // let (_cancel_tx, cancel_rx) = mpsc::channel(1);
    //     let (remote_ufrag, remote_pwd) = (local_ufrag,local_pwd);
    // let conn = ice_agent.dial(cancel_rx, remote_ufrag, remote_pwd).await?;
    // let weak_conn = Arc::downgrade(&conn);
    
    // let mut int = tokio::time::interval(Duration::from_secs(1));
    // let weak_agent = Arc::downgrade(&ice_agent);
    // loop {
    //     int.tick().await;
    //     // println!(
    //     //     "weak_agent: weak count = {}, strong count = {}",
    //     //     weak_agent.weak_count(),
    //     //     weak_agent.strong_count(),
    //     // );
    //     if weak_agent.strong_count() == 0 {
    //         break;
    //     }
    // }

    let server = TcpServer::new(9001, "127.0.0.1");
    let _ = server.connect().await;
    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;
    Ok(())
}

async fn create_ice_agent(is_controlling: bool) -> Result<Arc<Agent>, Error> {
    let local_port = if is_controlling { 4000 } else { 4001};

    let udp_network = {
        let udp_socket = UdpSocket::bind(("0.0.0.0", local_port)).await?;
        let udp_mux = UDPMuxDefault::new(UDPMuxParams::new(udp_socket));
        UDPNetwork::Muxed(udp_mux)
    };
    let agent = Agent::new(AgentConfig {
        network_types: vec![NetworkType::Udp4],
        udp_network,
        ..Default::default()
    })
    .await?;
    let ice_agent = Arc::new(agent);
    Ok(ice_agent)
}

fn handle_candidate<'a>() -> Box<
    dyn FnMut(Option<Arc<dyn Candidate + Send + Sync>>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
> {
    Box::new(move |c: Option<Arc<dyn Candidate + Send + Sync>>| {
        Box::pin(async move {
            if let Some(c) = c {
                println!("posting remoteCandidate with {}", c.marshal());
            }
        })
    })
}
