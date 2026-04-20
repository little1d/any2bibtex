"""
Semantic Scholar search helpers.

This module keeps title-search concerns out of `resolvers.py` and centralizes
API key usage, light retry behavior, and candidate scoring.
"""

from __future__ import annotations

import os
import time
from dataclasses import dataclass
from difflib import SequenceMatcher
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


class SemanticScholarLowConfidenceError(SemanticScholarError):
    """Raised when title search does not produce a trustworthy match."""


@dataclass
class SemanticScholarCandidate:
    title: str
    year: str
    doi: str | None
    arxiv_id: str | None
    authors: list[str]
    title_similarity: float
    score: float


def get_api_key() -> str | None:
    value = os.environ.get(API_KEY_ENV, "").strip()
    return value or None


def search_title_candidates(title: str, limit: int = 5) -> list[SemanticScholarCandidate]:
    payload = _search(title, limit=limit)
    items = payload.get("data") or []
    candidates = [_candidate_from_item(title, item) for item in items]
    return sorted(candidates, key=lambda candidate: candidate.score, reverse=True)


def choose_best_title_candidate(title: str, limit: int = 5) -> SemanticScholarCandidate:
    candidates = search_title_candidates(title, limit=limit)
    if not candidates:
        raise SemanticScholarLowConfidenceError("Semantic Scholar returned no title matches.")

    best = candidates[0]
    if best.title_similarity < 0.72:
        raise SemanticScholarLowConfidenceError(
            "Semantic Scholar did not return a confident title match."
        )

    if best.title_similarity < 0.9 and not (best.doi or best.arxiv_id):
        raise SemanticScholarLowConfidenceError(
            "Semantic Scholar only returned low-confidence matches without DOI/arXiv metadata."
        )

    return best


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

    title_similarity = _title_similarity(query, title)
    score = _score_candidate(
        query=query,
        title=title,
        doi=doi,
        arxiv_id=arxiv_id,
        authors=authors,
        title_similarity=title_similarity,
    )
    return SemanticScholarCandidate(
        title=title,
        year=year,
        doi=doi,
        arxiv_id=arxiv_id,
        authors=authors,
        title_similarity=title_similarity,
        score=score,
    )


def _score_candidate(
    query: str,
    title: str,
    doi: str | None,
    arxiv_id: str | None,
    authors: list[str],
    title_similarity: float,
) -> float:
    query_norm = _normalize_text(query)
    title_norm = _normalize_text(title)

    score = title_similarity * 100.0
    if title_norm == query_norm:
        score += 35.0
    elif query_norm and title_norm:
        query_words = set(query_norm.split())
        title_words = set(title_norm.split())
        overlap = len(query_words & title_words)
        if query_words:
            score += 20.0 * (overlap / len(query_words))

    if doi:
        score += 25.0
    if arxiv_id:
        score += 18.0
    if authors:
        score += min(len(authors), 4)

    lowered = title_norm.lower()
    if "survey" in lowered or "review" in lowered:
        score -= 20.0

    return score


def _normalize_text(value: str) -> str:
    normalized = "".join(char.lower() if char.isalnum() else " " for char in value)
    return " ".join(normalized.split())


def _title_similarity(query: str, title: str) -> float:
    return SequenceMatcher(None, _normalize_text(query), _normalize_text(title)).ratio()


def _clean_identifier(value: Any) -> str | None:
    text = str(value or "").strip()
    return text or None
