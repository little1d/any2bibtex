export function formatBibtex(bibtex: string): string {
  if (!bibtex) return "";

  let text = bibtex.trim();
  const lines = text.split("\n");

  // 单行 BibTeX 需要按字段重新断行，便于展示和复制。
  if (lines.length <= 2) {
    const entryMatch = text.match(/^(@\w+\{[^,]+,)/);
    if (!entryMatch) return text;

    const entryStart = entryMatch[1];
    let rest = text.slice(entryStart.length);
    if (rest.endsWith("}")) rest = rest.slice(0, -1);

    const fields: string[] = [];
    let current = "";
    let braceDepth = 0;

    // 只在最外层逗号处分割字段，避免 title={Hello, World} 被错误切开。
    for (let i = 0; i < rest.length; i++) {
      const char = rest[i];
      if (char === "{") braceDepth++;
      if (char === "}") braceDepth--;

      if (char === "," && braceDepth === 0) {
        const field = current.trim();
        if (field) fields.push(field);
        current = "";
      } else {
        current += char;
      }
    }
    if (current.trim()) fields.push(current.trim());

    const formattedLines = [entryStart];
    for (const field of fields) {
      formattedLines.push("  " + field + ",");
    }
    formattedLines.push("}");

    return formattedLines.join("\n");
  }

  // 多行 BibTeX 只做缩进规范化。
  const formattedLines: string[] = [];
  for (let line of lines) {
    line = line.trim();
    if (!line) continue;
    if (line.startsWith("@")) formattedLines.push(line);
    else if (line === "}") formattedLines.push("}");
    else formattedLines.push("  " + line);
  }

  return formattedLines.join("\n");
}
