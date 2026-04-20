"""
any2bibtex - Resolvers Module
核心解析逻辑：通过不同来源获取论文元数据并转换为 BibTeX
"""

import re
import requests
from typing import Optional, Tuple

from semantic_scholar import (
    SemanticScholarError,
    SemanticScholarLowConfidenceError,
    SemanticScholarRateLimitError,
    choose_best_title_candidate,
)

# === 输入类型识别 ===

DOI_PATTERN = re.compile(r'^10\.\d{4,}/[^\s]+$')
ARXIV_PATTERN = re.compile(r'^(\d{4}\.\d{4,5})(v\d+)?$|^[a-z-]+/\d{7}$', re.IGNORECASE)


def identify_input(query: str) -> Tuple[str, str]:
    """
    识别输入类型
    返回: (type, normalized_query)
    type: 'doi' | 'arxiv' | 'title'
    """
    query = query.strip()
    
    # 尝试识别 DOI
    if DOI_PATTERN.match(query):
        return ('doi', query)
    
    # 处理常见的 DOI URL 格式
    if 'doi.org/' in query:
        doi = query.split('doi.org/')[-1]
        return ('doi', doi)
    
    # 尝试识别 arXiv ID
    if ARXIV_PATTERN.match(query):
        return ('arxiv', query)
    
    # 处理 arXiv URL
    if 'arxiv.org' in query:
        # 从 URL 中提取 ID
        match = re.search(r'(\d{4}\.\d{4,5})(v\d+)?', query)
        if match:
            return ('arxiv', match.group(0))
    
    # 默认按标题处理
    return ('title', query)


# === DOI 解析 (内容协商) ===

def resolve_doi(doi: str) -> Optional[str]:
    """
    通过 DOI 获取 BibTeX
    利用 doi.org 的内容协商机制直接获取 BibTeX 格式
    """
    url = f"https://doi.org/{doi}"
    headers = {
        'Accept': 'application/x-bibtex; charset=utf-8'
    }
    
    try:
        response = requests.get(url, headers=headers, timeout=10, allow_redirects=True)
        if response.status_code == 200:
            return response.text
        return None
    except requests.RequestException:
        return None


# === arXiv 解析 ===

def resolve_arxiv(arxiv_id: str) -> Optional[str]:
    """
    通过 arXiv ID 获取元数据并生成 BibTeX
    arXiv 不直接提供 BibTeX，需要手动构造
    """
    url = f"http://export.arxiv.org/api/query?id_list={arxiv_id}"
    
    try:
        response = requests.get(url, timeout=10)
        if response.status_code != 200:
            return None
        
        content = response.text
        
        # 查找 entry 部分（论文内容在 <entry> 中）
        entry_match = re.search(r'<entry>(.*?)</entry>', content, re.DOTALL)
        if not entry_match:
            return None
        
        entry = entry_match.group(1)
        
        # 从 entry 中提取标题
        title_match = re.search(r'<title>([^<]+)</title>', entry)
        title = title_match.group(1).strip() if title_match else "Unknown"
        # 移除换行和多余空格
        title = ' '.join(title.split())
        
        # 提取作者
        authors = re.findall(r'<name>([^<]+)</name>', entry)
        author_str = ' and '.join(authors) if authors else "Unknown"
        
        # 提取年份
        published_match = re.search(r'<published>(\d{4})', entry)
        year = published_match.group(1) if published_match else "Unknown"
        
        # 提取 primary category
        category_match = re.search(r'<arxiv:primary_category[^>]*term="([^"]+)"', entry)
        primary_class = category_match.group(1) if category_match else "cs.AI"
        
        # 尝试提取 DOI
        doi_match = re.search(r'<arxiv:doi[^>]*>([^<]+)</arxiv:doi>', entry)
        if doi_match:
            # 如果有 DOI，直接用内容协商获取更准确的 BibTeX
            doi_bibtex = resolve_doi(doi_match.group(1))
            if doi_bibtex:
                return doi_bibtex
        
        # 生成 BibTeX (没有 DOI 的情况)
        # 生成 citation key
        first_author = authors[0].split()[-1] if authors else "unknown"
        cite_key = f"{first_author.lower()}{year}arxiv"
        
        bibtex = f"""@article{{{cite_key},
  title = {{{title}}},
  author = {{{author_str}}},
  year = {{{year}}},
  eprint = {{{arxiv_id}}},
  archivePrefix = {{arXiv}},
  primaryClass = {{{primary_class}}},
  url = {{https://arxiv.org/abs/{arxiv_id}}}
}}"""
        return bibtex
        
    except requests.RequestException:
        return None


# === 标题搜索 (Semantic Scholar) ===

def resolve_title(title: str) -> Optional[str]:
    """
    通过标题在 Semantic Scholar 搜索候选论文，优先使用 DOI。
    """
    try:
        paper = choose_best_title_candidate(title, limit=5)

        if paper.doi:
            return resolve_doi(paper.doi)

        if paper.arxiv_id:
            return resolve_arxiv(paper.arxiv_id)

        author_names = paper.authors
        author_str = ' and '.join(author_names) if author_names else "Unknown"
        year = paper.year or 'Unknown'
        paper_title = paper.title or title

        first_author = author_names[0].split()[-1] if author_names else "unknown"
        cite_key = f"{first_author.lower()}{year}"

        bibtex = f"""@article{{{cite_key},
  title = {{{paper_title}}},
  author = {{{author_str}}},
  year = {{{year}}}
}}"""
        return bibtex

    except SemanticScholarRateLimitError:
        raise
    except SemanticScholarLowConfidenceError:
        raise
    except SemanticScholarError:
        return None


# === 主解析函数 ===

def resolve(query: str) -> dict:
    """
    主入口：自动识别输入类型并解析
    返回: {"success": bool, "type": str, "bibtex": str | None, "error": str | None}
    """
    input_type, normalized = identify_input(query)
    
    resolver_map = {
        'doi': resolve_doi,
        'arxiv': resolve_arxiv,
        'title': resolve_title
    }
    
    resolver = resolver_map.get(input_type)
    if not resolver:
        return {"success": False, "type": input_type, "bibtex": None, "error": "Unknown input type"}
    
    try:
        bibtex = resolver(normalized)
    except SemanticScholarLowConfidenceError as exc:
        return {
            "success": False,
            "type": input_type,
            "bibtex": None,
            "error": str(exc),
        }
    except SemanticScholarRateLimitError as exc:
        return {
            "success": False,
            "type": input_type,
            "bibtex": None,
            "error": str(exc),
        }

    if bibtex:
        return {"success": True, "type": input_type, "bibtex": bibtex, "error": None}
    else:
        return {"success": False, "type": input_type, "bibtex": None, "error": f"Failed to resolve {input_type}: {normalized}"}
