"""Tests for declarative config validation logic."""

import json
import yaml
import pytest


SAMPLE_YAML_CONFIG = """
hostname: "kairosos-node1"
kernel:
  modules:
    - kvm-amd
    - nftables
    - wireguard
services:
  sshd:
    enabled: true
    port: 22
  docker:
    enabled: true
  ollama:
    enabled: true
    model: "hermes-3-llama-3.1:8b-q4_k_m"
users:
  - name: "kairos"
    groups: ["wheel", "docker"]
    shell: "/bin/bash"
networking:
  firewall:
    backend: "nftables"
    rules:
      - action: accept
        protocol: tcp
        port: 22
      - action: accept
        protocol: tcp
        port: 11434
updates:
  channel: "stable"
  auto_apply: false
"""


class TestDeclarativeConfig:
    def test_parse_yaml(self):
        config = yaml.safe_load(SAMPLE_YAML_CONFIG)
        assert config["hostname"] == "kairosos-node1"
        assert config["services"]["sshd"]["enabled"] is True
        assert len(config["users"]) == 1

    def test_config_validation_hostname(self):
        config = yaml.safe_load(SAMPLE_YAML_CONFIG)
        hostname = config.get("hostname", "")
        import re
        assert re.match(r'^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$', hostname), f"Invalid hostname: {hostname}"

    def test_config_validation_users(self):
        config = yaml.safe_load(SAMPLE_YAML_CONFIG)
        for user in config.get("users", []):
            assert "name" in user
            assert "groups" in user

    def test_config_validation_services(self):
        config = yaml.safe_load(SAMPLE_YAML_CONFIG)
        for svc_name, svc_config in config.get("services", {}).items():
            if "enabled" in svc_config:
                assert isinstance(svc_config["enabled"], bool)

    def test_config_serialization_roundtrip(self):
        config = yaml.safe_load(SAMPLE_YAML_CONFIG)
        serialized = yaml.dump(config, default_flow_style=False)
        reloaded = yaml.safe_load(serialized)
        assert reloaded == config
