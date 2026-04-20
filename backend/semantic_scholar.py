"""
Semantic Scholar search helpers.

This module keeps title-search concerns out of `resolvers.py` and centralizes
API key usage, light retry behavior, and candidate scoring.
"""

from __future__ import annotations

import os
import time
from dataclasses import dataclass
from typing import Any

import requests

SEARCH_URL = "https://api.semanticscholar.org/graph/v1/paper/search"
API_KEY_ENV = "SEMANTIC_SCHOLAR_API_KEY"
DEFAULT_TIMEOUT = 10
RETRYABLE_STATUS_CODES = {429, 500, 502, 503, 504}


class SemanticScholarError(RuntimeError):
    """Base error for Semantic Scholar search failures."""


class SemanticScholarRateLimitError(SemanticScholarError):
    """Raised when Semantic Scholar throttles the request."""


@dataclass
class SemanticScholarCandidate:
    title: str
    year: str
    doi: str | None
    arxiv_id: str | None
    authors: list[str]
    score: float


def get_api_key() -> str | None:
    value = os.environ.get(API_KEY_ENV, "").strip()
    return value or None


def search_title_candidates(title: str, limit: int = 5) -> list[SemanticScholarCandidate]:
    payload = _search(title, limit=limit)
    items = payload.get("data") or []
    candidates = [_candidate_from_item(title, item) for item in items]
    return sorted(candidates, key=lambda candidate: candidate.score, reverse=True)


def _search(title: str, limit: int) -> dict[str, Any]:
    headers = {"User-Agent": "any2bibtex/0.0.3"}
    api_key = get_api_key()
    if api_key:
        headers["x-api-key"] = api_key

    params = {
        "query": title,
        "limit": limit,
        "fields": "externalIds,title,authors,year",
    }

    # Semantic Scholar explicitly recommends using an API key when available,
    # but anonymous traffic is still supported. Keep the retry loop narrow so
    # UI requests fail fast instead of hanging for a long time.
    backoffs = (0.0, 1.25, 2.5)
    last_error: Exception | None = None

    for attempt, backoff in enumerate(backoffs, start=1):
        if backoff:
            time.sleep(backoff)

        try:
            response = requests.get(
                SEARCH_URL,
                params=params,
                headers=headers,
                timeout=DEFAULT_TIMEOUT,
            )
        except requests.RequestException as exc:
            last_error = exc
            continue

        if response.status_code == 200:
            return response.json()

        if response.status_code == 429:
            last_error = SemanticScholarRateLimitError(
                "Semantic Scholar rate limit exceeded. Add SEMANTIC_SCHOLAR_API_KEY "
                "or retry in a moment."
            )
            continue

        if response.status_code in RETRYABLE_STATUS_CODES:
            last_error = SemanticScholarError(
                f"Semantic Scholar temporary error: HTTP {response.status_code}"
            )
            continue

        raise SemanticScholarError(
            f"Semantic Scholar search failed: HTTP {response.status_code}"
        )

    if isinstance(last_error, SemanticScholarError):
        raise last_error
    if last_error is not None:
        raise SemanticScholarError(
            f"Semantic Scholar request failed: {last_error}"
        ) from last_error
    raise SemanticScholarError("Semantic Scholar search failed for an unknown reason")


def _candidate_from_item(query: str, item: dict[str, Any]) -> SemanticScholarCandidate:
    external_ids = item.get("externalIds") or {}
    title = str(item.get("title") or "").strip()
    year = str(item.get("year") or "")
    authors = [
        str(author.get("name") or "").strip()
        for author in (item.get("authors") or [])
        if str(author.get("name") or "").strip()
    ]
    doi = _clean_identifier(external_ids.get("DOI"))
    arxiv_id = _clean_identifier(external_ids.get("ArXiv"))

    score = _score_candidate(query=query, title=title, doi=doi, authors=authors)
    return SemanticScholarCandidate(
        title=title,
        year=year,
        doi=doi,
        arxiv_id=arxiv_id,
        authors=authors,
        score=score,
    )


def _score_candidate(query: str, title: str, doi: str | None, authors: list[str]) -> float:
    query_norm = _normalize_text(query)
    title_norm = _normalize_text(title)

    score = 0.0
    if title_norm == query_norm:
        score += 100.0
    elif query_norm and title_norm:
        query_words = set(query_norm.split())
        title_words = set(title_norm.split())
        overlap = len(query_words & title_words)
        if query_words:
            score += 60.0 * (overlap / len(query_words))

    if doi:
        score += 25.0
    if authors:
        score += min(len(authors), 4)

    lowered = title_norm.lower()
    if "survey" in lowered or "review" in lowered:
        score -= 20.0

    return score


def _normalize_text(value: str) -> str:
    normalized = "".join(char.lower() if char.isalnum() else " " for char in value)
    return " ".join(normalized.split())


def _clean_identifier(value: Any) -> str | None:
    text = str(value or "").strip()
    return text or None
