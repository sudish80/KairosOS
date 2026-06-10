# Entity extraction from text — powered by LLM or deterministic patterns

import json
import re
from typing import Optional


class EntityExtractor:
    def __init__(self, agent_endpoint: Optional[str] = None):
        self.agent_endpoint = agent_endpoint

    def extract_entities(self, text: str) -> list:
        """Extract entities using deterministic patterns (fast, no LLM needed)"""
        entities = []

        # Extract URLs
        url_pattern = re.compile(r'https?://[^\s]+')
        for url in url_pattern.findall(text):
            entities.append({
                "id": f"url-{hash(url) % 10**8:08x}",
                "type": "url",
                "name": url,
                "description": f"Referenced URL: {url}",
                "metadata": {"source": "url_pattern"},
                "confidence": 0.8,
            })

        # Extract file paths
        path_pattern = re.compile(r'(?:/[^/\s]+)+')
        for path in path_pattern.findall(text):
            if len(path) > 3:
                entities.append({
                    "id": f"path-{hash(path) % 10**8:08x}",
                    "type": "filepath",
                    "name": path,
                    "description": f"File path reference: {path}",
                    "metadata": {"source": "path_pattern"},
                    "confidence": 0.7,
                })

        # Extract package names
        pkg_pattern = re.compile(r'(?:apt install|pip install|npm install|pacman -S)\s+([\w\-\.]+)')
        for match in pkg_pattern.finditer(text):
            pkg = match.group(1)
            entities.append({
                "id": f"pkg-{hash(pkg) % 10**8:08x}",
                "type": "package",
                "name": pkg,
                "description": f"Software package: {pkg}",
                "metadata": {"source": "install_command"},
                "confidence": 0.9,
            })

        # Extract IP addresses
        ip_pattern = re.compile(r'\b(?:\d{1,3}\.){3}\d{1,3}\b')
        for ip in ip_pattern.findall(text):
            entities.append({
                "id": f"ip-{hash(ip) % 10**8:08x}",
                "type": "ip_address",
                "name": ip,
                "description": f"IP address: {ip}",
                "metadata": {"source": "ip_pattern"},
                "confidence": 0.9,
            })

        # Extract email addresses
        email_pattern = re.compile(r'\b[\w\.-]+@[\w\.-]+\.\w+\b')
        for email in email_pattern.findall(text):
            entities.append({
                "id": f"email-{hash(email) % 10**8:08x}",
                "type": "email",
                "name": email,
                "description": f"Email address: {email}",
                "metadata": {"source": "email_pattern"},
                "confidence": 0.9,
            })

        return entities

    def extract_relationships(self, text: str) -> list:
        """Extract relationships between entities"""
        rels = []

        # "installed" relationships
        install_pattern = re.compile(r'installed\s+([\w\-\.]+)')
        for match in install_pattern.finditer(text):
            rels.append({
                "type": "installed",
                "target_name": match.group(1),
                "confidence": 0.8,
            })

        # "configured" relationships
        config_pattern = re.compile(r'configured\s+(\w+)')
        for match in config_pattern.finditer(text):
            rels.append({
                "type": "configured",
                "target_name": match.group(1),
                "confidence": 0.7,
            })

        return rels
