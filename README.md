# ts-pinger

`ts-pinger` is a small little daemon that has one simple job: to constantly ping Tailscale clients, keeping the Wireguard connections used by Tailscale alive.

### Why?

My home is behind CGNAT. While Tailscale does an incredible job of managing to traverse CGNAT (at least when the other side has a public IP), it sometimes takes a few seconds to initialize a tunnel, during which packets are dropped or held up.

By constantly pinging Tailscale clients, the Wireguard connections are held open and traffic can immediately traverse them when needed.

### Installation
