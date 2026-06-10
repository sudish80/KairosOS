# Network Manager Skill

## Description
Configure and troubleshoot network interfaces, firewall rules, DNS, and connectivity.

## Triggers
- User reports network issues
- New network interface detected
- Firewall configuration requests
- DNS resolution problems

## Actions

### status()
Show network interface status, IP addresses, routes, DNS config

### scan_wifi()
Scan available WiFi networks (if wireless interface present)

### connect_wifi(ssid, password)
Connect to a WiFi network

### set_static_ip(interface, ip, netmask, gateway)
Configure a static IP address

### firewall_list()
List current firewall rules

### firewall_rule(action, port, protocol)
Add/remove a firewall rule:
- `action`: allow | deny
- `port`: port number or service name
- `protocol`: tcp | udp

### diagnose()
Run network diagnostics:
- Ping test to gateway
- DNS resolution test
- Bandwidth test
- Packet loss check
- Traceroute to common destinations

### set_dns(servers)
Configure DNS servers (systemd-resolved or /etc/resolv.conf)

## Example
User: "The internet is slow"
Agent: *runs diagnose, finds DNS issues, switches to Cloudflare DNS*

## Dependencies
- ip, ifconfig, iwctl
- iptables/nftables
- systemd-resolved
- ping, traceroute, dig
