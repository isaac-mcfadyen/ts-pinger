use std::{
	process::{Command, Stdio},
	time::{Duration, Instant},
};

use clap::{ArgAction, Parser};
use eyre::{OptionExt, bail};
use serde::Deserialize;
use serde_json::Value;
use tracing::Level;

#[derive(Parser)]
struct Args {
	/// How often to ping peers in milliseconds.
	#[clap(long, short, default_value = "5000")]
	interval: u64,

	/// Whether to exclude mobile devices (iOS, Android). Disabling this may increase battery usage!
	#[clap(
		long,
		default_missing_value("true"),
        default_value("true"),
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set,
	)]
	exclude_mobile: bool,

	/// A comma-delimited list of devices to include. Mutually exclusive with the exclude argument.
	#[clap(long, group = "filter")]
	include: Option<String>,

	/// A comma-delimited list of devices to exclude. Mutually exclusive with the include argument.
	#[clap(long, group = "filter")]
	exclude: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Peer {
	#[serde(rename = "HostName")]
	hostname: String,
	#[serde(rename = "DNSName")]
	dns_name: String,
	#[serde(rename = "OS")]
	os: String,
}

fn get_peers() -> eyre::Result<Vec<Peer>> {
	let res = Command::new("tailscale")
		.arg("status")
		.arg("--json")
		.arg("--self=false")
		.output()?;
	let parsed: Value = serde_json::from_slice(&res.stdout)?;
	let peer = parsed["Peer"]
		.as_object()
		.ok_or_eyre("Missing Peer entry from tailscale status!")?
		.clone();
	let peers = peer
		.into_values()
		.map(serde_json::from_value)
		.collect::<Result<Vec<Peer>, _>>()?;
	Ok(peers)
}
fn ping_peer(peer: &Peer) -> eyre::Result<()> {
	Command::new("ping")
		.args(["-c", "1", "-i", "0.5"])
		.arg(&peer.dns_name)
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	Ok(())
}
fn filter_peers(mut peers: Vec<Peer>, args: &Args) -> eyre::Result<Vec<Peer>> {
	// First remove Android or IOS based on arg.
	if args.exclude_mobile {
		peers.retain(|v| v.os.to_lowercase() != "ios" && v.os.to_lowercase() != "android");
	}

	// Bail if both include and exclude are specified (shouldn't be possible b/c of clap group).
	if args.include.is_some() && args.exclude.is_some() {
		bail!("Include and exclude cannot be specified at the same time!")
	}

	if let Some(inc) = &args.include {
		let inc = inc.split(",").map(|v| v.to_string()).collect::<Vec<_>>();
		peers.retain(|v| inc.contains(&v.hostname));
	} else if let Some(exc) = &args.exclude {
		let exc = exc.split(",").map(|v| v.to_string()).collect::<Vec<_>>();
		peers.retain(|v| !exc.contains(&v.hostname));
	}

	Ok(peers)
}

fn main() -> eyre::Result<()> {
	tracing_subscriber::fmt().with_max_level(Level::INFO).init();

	let args = Args::parse();

	tracing::info!("ts-pinger starting.");
	tracing::info!("Ping interval set to {}ms.", args.interval);

	let mut last_ping: Instant;
	loop {
		last_ping = Instant::now();

		// Pull peers and filter. Must do this here because peers may get added/removed.
		let peers = get_peers()?;
		let peers = filter_peers(peers, &args)?;

		// Ping all peers.
		tracing::info!("Ping starting:");
		for peer in peers {
			tracing::info!("  Pinging {}", peer.hostname);
			ping_peer(&peer)?;
		}

		let since_last = last_ping.elapsed().as_millis() as u64;
		if since_last < args.interval {
			let to_wait = args.interval - since_last;
			tracing::info!("Waiting for {}ms before next ping.", to_wait);
			std::thread::sleep(Duration::from_millis(to_wait));
		}
	}
}
