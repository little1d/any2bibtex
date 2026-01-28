"""
any2bibtex - FastAPI Backend
提供 RESTful API 供 Electron 前端调用
"""

from fastapi import FastAPI, Query
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Optional

from resolvers import resolve, identify_input

app = FastAPI(
    title="any2bibtex API",
    description="将 DOI、arXiv ID 或论文标题转换为 BibTeX",
    version="0.1.0"
)

# 允许跨域（Electron 前端需要）
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class ResolveResponse(BaseModel):
    success: bool
    type: str
    bibtex: Optional[str] = None
    error: Optional[str] = None


@app.get("/")
def root():
    return {"message": "any2bibtex API is running", "version": "0.1.0"}


@app.get("/resolve", response_model=ResolveResponse)
def resolve_query(q: str = Query(..., description="DOI, arXiv ID, or paper title")):
    """
    主解析接口
    自动识别输入类型并返回 BibTeX
    """
    result = resolve(q)
    return ResolveResponse(**result)


@app.get("/identify")
def identify_query(q: str = Query(..., description="Query to identify")):
    """
    识别输入类型（调试用）
    """
    input_type, normalized = identify_input(q)
    return {"type": input_type, "normalized": normalized}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8765)
