export function formatBibtex(bibtex: string): string {
  if (!bibtex) return "";

  let text = bibtex.trim();
  const lines = text.split("\n");
  
  if (lines.length <= 2) {
    const entryMatch = text.match(/^(@\w+\{[^,]+,)/);
    if (!entryMatch) return text;

    const entryStart = entryMatch[1];
    let rest = text.slice(entryStart.length);
    if (rest.endsWith("}")) rest = rest.slice(0, -1);

    const fields: string[] = [];
    let current = "";
    let braceDepth = 0;

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

  // Already multi-line
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
