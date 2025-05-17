# ts-pinger

`ts-pinger` is a small little daemon that has one simple job: to constantly ping Tailscale clients, keeping the Wireguard connections used by Tailscale alive.

### Why?

My home is behind CGNAT. While Tailscale does an incredible job of managing to traverse CGNAT (at least when the other side has a public IP), it sometimes takes a few seconds to initialize a tunnel, during which packets are dropped or held up.

By constantly pinging Tailscale clients, the Wireguard connections are held open and traffic can immediately traverse them when needed.

### Installation

This program is written in Rust. Binaries are provided on the Releases page, or you can compile from source using:

```bash
cargo install --locked --git https://github.com/isaac-mcfadyen/ts-pinger
```

### Usage

Just run the binary. You probably want to install as a `systemd` service or similar so that it runs in the background.

The ping interval can be changed using the `--interval <time in ms>` flag. A filter for clients can be given using either the `--include client1,client2` or `--exclude client1,client2` flags.

For battery life reasons, mobile devices are automatically excluded from pings unless the `--exclude-mobile=false` flag is provided.
