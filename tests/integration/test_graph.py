"""Integration tests for KairosOS Knowledge Graph service."""

import os
import tempfile
import pytest

# Adjust path to find kairos_pkg
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../src/services/kairos-pkg/src'))

from kairos_pkg.graph import KnowledgeGraph
from kairos_pkg.extraction import extract_entities


@pytest.fixture
def kg():
    """Create a temporary knowledge graph for testing."""
    with tempfile.NamedTemporaryFile(suffix='.db', delete=False) as f:
        db_path = f.name
    kg = KnowledgeGraph(db_path)
    yield kg
    kg.close()
    os.unlink(db_path)


class TestKnowledgeGraph:
    def test_init_creates_tables(self, kg):
        tables = kg.list_tables()
        assert 'entities' in tables
        assert 'relationships' in tables
        assert 'documents' in tables

    def test_add_and_get_entity(self, kg):
        eid = kg.add_entity("test-entity", "url", {"href": "https://example.com"})
        assert eid is not None
        entity = kg.get_entity(eid)
        assert entity['name'] == "test-entity"
        assert entity['type'] == "url"

    def test_add_relationship(self, kg):
        e1 = kg.add_entity("entity-a", "concept", {})
        e2 = kg.add_entity("entity-b", "concept", {})
        rid = kg.add_relationship(e1, e2, "related_to")
        assert rid is not None

    def test_search_fts(self, kg):
        kg.add_entity("python programming language", "concept", {"description": "A high-level programming language"})
        kg.add_entity("rust systems programming", "concept", {"description": "A systems programming language"})
        results = kg.search("python", top_k=5)
        assert len(results) >= 1

    def test_store_document(self, kg):
        did = kg.store_document("Test document content with important information", tags=["test"])
        assert did is not None


class TestExtraction:
    def test_extract_urls(self):
        text = "Visit https://example.com and http://test.org for info"
        entities = extract_entities(text)
        urls = [e for e in entities if e['type'] == 'url']
        assert len(urls) == 2

    def test_extract_file_paths(self):
        text = "Check /etc/kairos/configuration.nix and /var/log/syslog"
        entities = extract_entities(text)
        paths = [e for e in entities if e['type'] == 'file_path']
        assert len(paths) >= 2

    def test_extract_packages(self):
        text = "Install nginx, postgresql-16, and python3-pip"
        entities = extract_entities(text)
        pkgs = [e for e in entities if e['type'] == 'package']
        assert len(pkgs) >= 3

    def test_extract_emails(self):
        text = "Contact admin@kairosos.org or support@example.com"
        entities = extract_entities(text)
        emails = [e for e in entities if e['type'] == 'email']
        assert len(emails) == 2

    def test_extract_ips(self):
        text = "Connect to 192.168.1.1 or 10.0.0.1:8080"
        entities = extract_entities(text)
        ips = [e for e in entities if e['type'] == 'ip_address']
        assert len(ips) >= 2
