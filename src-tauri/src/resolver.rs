use regex::Regex;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::collections::HashSet;
use strsim::normalized_levenshtein;

const SEARCH_URL: &str = "https://api.semanticscholar.org/graph/v1/paper/search";
const DEFAULT_TIMEOUT_SECS: u64 = 10;

#[derive(Clone, Debug)]
enum InputKind {
    Doi,
    Arxiv,
    Title,
}

#[derive(Debug)]
enum ResolveError {
    RateLimit(String),
    LowConfidence(String),
    Generic(String),
}

#[derive(Clone, Debug)]
struct SemanticScholarCandidate {
    title: String,
    year: Option<String>,
    doi: Option<String>,
    arxiv_id: Option<String>,
    authors: Vec<String>,
    title_similarity: f64,
    score: f64,
}

#[derive(Deserialize)]
struct SemanticScholarSearchResponse {
    #[serde(default)]
    data: Vec<SemanticScholarPaper>,
}

#[derive(Deserialize)]
struct SemanticScholarPaper {
    #[serde(default, rename = "externalIds")]
    external_ids: ExternalIds,
    title: Option<String>,
    year: Option<i32>,
    #[serde(default)]
    authors: Vec<SemanticScholarAuthor>,
}

#[derive(Default, Deserialize)]
struct ExternalIds {
    #[serde(rename = "DOI")]
    doi: Option<String>,
    #[serde(rename = "ArXiv")]
    arxiv: Option<String>,
}

#[derive(Deserialize)]
struct SemanticScholarAuthor {
    name: Option<String>,
}

pub fn identify_input(query: &str) -> (String, String) {
    let trimmed = query.trim();
    let doi_pattern = Regex::new(r"^10\.\d{4,}/[^\s]+$").expect("valid DOI regex");
    let arxiv_pattern =
        Regex::new(r"(?i)^(\d{4}\.\d{4,5})(v\d+)?$|^[a-z-]+/\d{7}$").expect("valid arXiv regex");

    if doi_pattern.is_match(trimmed) {
        return ("doi".to_string(), trimmed.to_string());
    }

    if let Some((_, doi)) = trimmed.split_once("doi.org/") {
        return ("doi".to_string(), doi.to_string());
    }

    if arxiv_pattern.is_match(trimmed) {
        return ("arxiv".to_string(), trimmed.to_string());
    }

    if trimmed.contains("arxiv.org") {
        let id_pattern = Regex::new(r"(\d{4}\.\d{4,5})(v\d+)?").expect("valid arXiv URL regex");
        if let Some(match_) = id_pattern.find(trimmed) {
            return ("arxiv".to_string(), match_.as_str().to_string());
        }
    }

    ("title".to_string(), trimmed.to_string())
}

pub async fn resolve(
    query: String,
    semantic_scholar_api_key: Option<String>,
) -> crate::ResolveResponse {
    let (input_type, normalized) = identify_input(&query);
    let kind = match input_type.as_str() {
        "doi" => InputKind::Doi,
        "arxiv" => InputKind::Arxiv,
        _ => InputKind::Title,
    };

    let result = match kind {
        InputKind::Doi => resolve_doi(&normalized).await,
        InputKind::Arxiv => resolve_arxiv(&normalized).await,
        InputKind::Title => resolve_title(&normalized, semantic_scholar_api_key).await,
    };

    match result {
        Ok(Some(bibtex)) => crate::ResolveResponse {
            success: true,
            input_type,
            bibtex: Some(bibtex),
            error: None,
        },
        Ok(None) => crate::ResolveResponse {
            success: false,
            input_type: input_type.clone(),
            bibtex: None,
            error: Some(format!("Failed to resolve {}: {}", input_type, normalized)),
        },
        Err(ResolveError::RateLimit(message) | ResolveError::LowConfidence(message)) => {
            crate::ResolveResponse {
                success: false,
                input_type,
                bibtex: None,
                error: Some(message),
            }
        }
        Err(ResolveError::Generic(_message)) => crate::ResolveResponse {
            success: false,
            input_type: input_type.clone(),
            bibtex: None,
            error: Some(format!("Failed to resolve {}: {}", input_type, normalized)),
        },
    }
}

async fn resolve_doi(doi: &str) -> Result<Option<String>, ResolveError> {
    let client = http_client()?;
    let response = client
        .get(format!("https://doi.org/{doi}"))
        .header(ACCEPT, "application/x-bibtex; charset=utf-8")
        .send()
        .await
        .map_err(|error| ResolveError::Generic(error.to_string()))?;

    if response.status().is_success() {
        return response
            .text()
            .await
            .map(Some)
            .map_err(|error| ResolveError::Generic(error.to_string()));
    }
    Ok(None)
}

async fn resolve_arxiv(arxiv_id: &str) -> Result<Option<String>, ResolveError> {
    let client = http_client()?;
    let response = client
        .get(format!(
            "http://export.arxiv.org/api/query?id_list={}",
            urlencoding::encode(arxiv_id)
        ))
        .send()
        .await
        .map_err(|error| ResolveError::Generic(error.to_string()))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let content = response
        .text()
        .await
        .map_err(|error| ResolveError::Generic(error.to_string()))?;
    let Some(entry) = capture(&content, r"(?s)<entry>(.*?)</entry>") else {
        return Ok(None);
    };

    let title = capture(&entry, r"(?s)<title>([^<]+)</title>")
        .map(|value| collapse_whitespace(&value))
        .unwrap_or_else(|| "Unknown".to_string());
    let authors = captures(&entry, r"<name>([^<]+)</name>");
    let author_str = if authors.is_empty() {
        "Unknown".to_string()
    } else {
        authors.join(" and ")
    };
    let year = capture(&entry, r"<published>(\d{4})").unwrap_or_else(|| "Unknown".to_string());
    let primary_class = capture(&entry, r#"<arxiv:primary_category[^>]*term="([^"]+)""#)
        .unwrap_or_else(|| "cs.AI".to_string());

    if let Some(doi) = capture(&entry, r"(?s)<arxiv:doi[^>]*>([^<]+)</arxiv:doi>") {
        if let Some(bibtex) = resolve_doi(&doi).await? {
            return Ok(Some(bibtex));
        }
    }

    let first_author = authors
        .first()
        .and_then(|author| author.split_whitespace().last())
        .unwrap_or("unknown")
        .to_lowercase();
    let cite_key = format!("{first_author}{year}arxiv");

    Ok(Some(format!(
        "@article{{{cite_key},\n  title = {{{title}}},\n  author = {{{author_str}}},\n  year = {{{year}}},\n  eprint = {{{arxiv_id}}},\n  archivePrefix = {{arXiv}},\n  primaryClass = {{{primary_class}}},\n  url = {{https://arxiv.org/abs/{arxiv_id}}}\n}}"
    )))
}

async fn resolve_title(
    title: &str,
    semantic_scholar_api_key: Option<String>,
) -> Result<Option<String>, ResolveError> {
    let paper = choose_best_title_candidate(title, semantic_scholar_api_key).await?;

    if let Some(doi) = paper.doi {
        return resolve_doi(&doi).await;
    }
    if let Some(arxiv_id) = paper.arxiv_id {
        return resolve_arxiv(&arxiv_id).await;
    }

    let author_str = if paper.authors.is_empty() {
        "Unknown".to_string()
    } else {
        paper.authors.join(" and ")
    };
    let year = paper.year.unwrap_or_else(|| "Unknown".to_string());
    let first_author = paper
        .authors
        .first()
        .and_then(|author| author.split_whitespace().last())
        .unwrap_or("unknown")
        .to_lowercase();
    let cite_key = format!("{first_author}{year}");

    Ok(Some(format!(
        "@article{{{cite_key},\n  title = {{{}}},\n  author = {{{author_str}}},\n  year = {{{year}}}\n}}",
        paper.title
    )))
}

async fn choose_best_title_candidate(
    title: &str,
    semantic_scholar_api_key: Option<String>,
) -> Result<SemanticScholarCandidate, ResolveError> {
    let mut candidates = search_title_candidates(title, 5, semantic_scholar_api_key).await?;
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));

    let Some(best) = candidates.into_iter().next() else {
        return Err(ResolveError::LowConfidence(
            "Semantic Scholar returned no title matches.".to_string(),
        ));
    };

    if best.title_similarity < 0.72 {
        return Err(ResolveError::LowConfidence(
            "Semantic Scholar did not return a confident title match.".to_string(),
        ));
    }
    if best.title_similarity < 0.9 && best.doi.is_none() && best.arxiv_id.is_none() {
        return Err(ResolveError::LowConfidence(
            "Semantic Scholar only returned low-confidence matches without DOI/arXiv metadata."
                .to_string(),
        ));
    }

    Ok(best)
}

async fn search_title_candidates(
    title: &str,
    limit: u8,
    semantic_scholar_api_key: Option<String>,
) -> Result<Vec<SemanticScholarCandidate>, ResolveError> {
    let client = http_client()?;
    let mut last_error = None;

    for backoff_ms in [0, 1250, 2500] {
        if backoff_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
        }

        let mut request = client
            .get(SEARCH_URL)
            .header(USER_AGENT, "any2bibtex/0.0.4")
            .query(&[
                ("query", title),
                ("limit", &limit.to_string()),
                ("fields", "externalIds,title,authors,year"),
            ]);

        if let Some(api_key) = semantic_scholar_api_key.as_deref().filter(|value| !value.is_empty())
        {
            request = request.header("x-api-key", api_key);
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                let payload = response
                    .json::<SemanticScholarSearchResponse>()
                    .await
                    .map_err(|error| ResolveError::Generic(error.to_string()))?;
                return Ok(payload
                    .data
                    .into_iter()
                    .map(|item| candidate_from_item(title, item))
                    .collect());
            }
            Ok(response) if response.status().as_u16() == 429 => {
                last_error = Some(ResolveError::RateLimit(
                    "Semantic Scholar rate limit exceeded. Add SEMANTIC_SCHOLAR_API_KEY or retry in a moment."
                        .to_string(),
                ));
            }
            Ok(response) if [500, 502, 503, 504].contains(&response.status().as_u16()) => {
                last_error = Some(ResolveError::Generic(format!(
                    "Semantic Scholar temporary error: HTTP {}",
                    response.status()
                )));
            }
            Ok(response) => {
                return Err(ResolveError::Generic(format!(
                    "Semantic Scholar search failed: HTTP {}",
                    response.status()
                )));
            }
            Err(error) => {
                last_error = Some(ResolveError::Generic(error.to_string()));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ResolveError::Generic("Semantic Scholar search failed for an unknown reason".to_string())
    }))
}

fn candidate_from_item(query: &str, item: SemanticScholarPaper) -> SemanticScholarCandidate {
    let title = item.title.unwrap_or_default().trim().to_string();
    let year = item.year.map(|value| value.to_string());
    let authors = item
        .authors
        .into_iter()
        .filter_map(|author| author.name.map(|name| name.trim().to_string()))
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    let doi = clean_identifier(item.external_ids.doi);
    let arxiv_id = clean_identifier(item.external_ids.arxiv);
    let title_similarity = title_similarity(query, &title);
    let score = score_candidate(
        query,
        &title,
        doi.as_deref(),
        arxiv_id.as_deref(),
        &authors,
        title_similarity,
    );

    SemanticScholarCandidate {
        title,
        year,
        doi,
        arxiv_id,
        authors,
        title_similarity,
        score,
    }
}

fn score_candidate(
    query: &str,
    title: &str,
    doi: Option<&str>,
    arxiv_id: Option<&str>,
    authors: &[String],
    title_similarity: f64,
) -> f64 {
    let query_norm = normalize_text(query);
    let title_norm = normalize_text(title);

    let mut score = title_similarity * 100.0;
    if title_norm == query_norm {
        score += 35.0;
    } else if !query_norm.is_empty() && !title_norm.is_empty() {
        let query_words = query_norm.split_whitespace().collect::<HashSet<_>>();
        let title_words = title_norm.split_whitespace().collect::<HashSet<_>>();
        let overlap = query_words.intersection(&title_words).count();
        if !query_words.is_empty() {
            score += 20.0 * (overlap as f64 / query_words.len() as f64);
        }
    }

    if doi.is_some() {
        score += 25.0;
    }
    if arxiv_id.is_some() {
        score += 18.0;
    }
    score += authors.len().min(4) as f64;

    if title_norm.contains("survey") || title_norm.contains("review") {
        score -= 20.0;
    }

    score
}

fn http_client() -> Result<reqwest::Client, ResolveError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|error| ResolveError::Generic(error.to_string()))
}

fn capture(content: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .captures(content)?
        .get(1)
        .map(|value| value.as_str().trim().to_string())
}

fn captures(content: &str, pattern: &str) -> Vec<String> {
    let Ok(regex) = Regex::new(pattern) else {
        return Vec::new();
    };
    regex
        .captures_iter(content)
        .filter_map(|capture| capture.get(1))
        .map(|value| value.as_str().trim().to_string())
        .collect()
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .map(|char_| {
            if char_.is_alphanumeric() {
                char_.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_similarity(query: &str, title: &str) -> f64 {
    normalized_levenshtein(&normalize_text(query), &normalize_text(title))
}

fn clean_identifier(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
