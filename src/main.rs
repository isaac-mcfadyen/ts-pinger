use std::{
	process::{Command, Stdio},
	time::{Duration, Instant},
};

use eyre::OptionExt;
use serde_json::Value;
use tracing::Level;

fn get_peers() -> eyre::Result<Vec<String>> {
	let res = Command::new("tailscale")
		.arg("status")
		.arg("--json")
		.arg("--self=false")
		.output()?;
	let parsed: Value = serde_json::from_slice(&res.stdout)?;
	let peer = parsed["Peer"]
		.as_object()
		.ok_or_eyre("Missing Peer entry from tailscale status!")?;
	let ips = peer
		.values()
		.map(|v| v["DNSName"].as_str().map(|v| v.to_string()))
		.collect::<Option<_>>()
		.ok_or_eyre("Missing DNSName from entries in tailscale status!")?;
	Ok(ips)
}
fn ping_peer(addr: &str) -> eyre::Result<()> {
	Command::new("ping")
		.args(["-c", "1", "-i", "0.5"])
		.arg(addr)
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	Ok(())
}
fn should_ping_peer(peer: &str, filter: &[&str]) -> bool {
	for p in filter {
		if peer.starts_with(p) {
			return true;
		}
	}
	false
}

fn main() -> eyre::Result<()> {
	tracing_subscriber::fmt().with_max_level(Level::INFO).init();

	let ping_time = std::env::var("PING_TIME_MS")
		.unwrap_or("4000".to_string())
		.parse::<u64>()?;
	let peer_filter = std::env::var("PEER_FILTER").unwrap_or("".to_string());
	let peer_filter = peer_filter.split(",").collect::<Vec<_>>();

	tracing::info!("Pinger starting.");
	tracing::info!("Ping time set to {}ms.", ping_time);
	if peer_filter.is_empty() {
		tracing::info!("No peer filter set, pinging all peers.");
	} else {
		tracing::info!("Using peer filter: {:?}", peer_filter);
	}

	let mut last_ping: Instant;
	loop {
		last_ping = Instant::now();

		// Ping all peers.
		let peers = get_peers()?;
		tracing::info!("Ping starting:");
		for peer in peers {
			if !should_ping_peer(&peer, &peer_filter) {
				continue;
			}
			tracing::info!("  Pinging {}", peer);
			ping_peer(&peer)?;
		}

		let since_last = last_ping.elapsed().as_millis() as u64;
		if since_last < ping_time {
			let to_wait = ping_time - since_last;
			tracing::info!("Waiting for {}ms before next ping.", to_wait);
			std::thread::sleep(Duration::from_millis(to_wait));
		}
	}
}
