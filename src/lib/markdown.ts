// Markdown mínimo y seguro: se escapa todo el HTML antes de aplicar formato.

const ESC: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" };
const esc = (s: string) => s.replace(/[&<>"']/g, (c) => ESC[c]);

function inline(src: string): string {
  let s = esc(src);
  s = s.replace(/`([^`]+)`/g, "<code>$1</code>");
  s = s.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/(^|[^*])\*([^*\s][^*]*)\*/g, "$1<em>$2</em>");
  s = s.replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2">$1</a>');
  s = s.replace(/(^|[\s(])(https?:\/\/[^\s<]+)/g, '$1<a href="$2">$2</a>');
  return s;
}

const CHECK_RE = /^\s*[-*] \[( |x|X)\] (.*)$/;

/** Marca o desmarca la casilla `- [ ]` de la línea `idx` del texto. */
export function toggleCheckLine(md: string, idx: number): string {
  const lines = md.split(/\r?\n/);
  const line = lines[idx];
  if (line === undefined || !CHECK_RE.test(line)) return md;
  lines[idx] = line.replace(/\[( |x|X)\]/, (m) => (m === "[ ]" ? "[x]" : "[ ]"));
  return lines.join("\n");
}

/** Línea de la casilla pulsada (o null si el clic no fue en una casilla). */
export function checkLineFromEvent(e: Event): number | null {
  const el = e.target as HTMLElement | null;
  if (!el || el.tagName !== "INPUT" || !el.dataset.line) return null;
  return Number(el.dataset.line);
}

export function renderMarkdown(md: string): string {
  const out: string[] = [];
  let inList = false;
  let para: string[] = [];
  const flushPara = () => {
    if (para.length) out.push(`<p>${para.map(inline).join("<br>")}</p>`);
    para = [];
  };
  const closeList = () => {
    if (inList) out.push("</ul>");
    inList = false;
  };

  const lines = md.split(/\r?\n/);
  for (let idx = 0; idx < lines.length; idx++) {
    const line = lines[idx];
    const check = line.match(CHECK_RE);
    const bullet = line.match(/^\s*[-*] (.*)$/);
    const heading = line.match(/^(#{1,3}) (.*)$/);
    if (check || bullet) {
      flushPara();
      if (!inList) out.push("<ul>");
      inList = true;
      if (check) {
        const done = check[1] !== " ";
        // La casilla lleva su número de línea para poder marcarla con un clic.
        const box = `<input type="checkbox" data-line="${idx}"${done ? " checked" : ""} aria-label="Marcar">`;
        out.push(`<li class="check">${box}${done ? `<s>${inline(check[2])}</s>` : inline(check[2])}</li>`);
      } else {
        out.push(`<li>${inline(bullet![1])}</li>`);
      }
    } else if (heading) {
      flushPara();
      closeList();
      const tag = heading[1].length === 1 ? "h3" : "h4";
      out.push(`<${tag}>${inline(heading[2])}</${tag}>`);
    } else if (line.trim() === "") {
      flushPara();
      closeList();
    } else {
      closeList();
      para.push(line);
    }
  }
  flushPara();
  closeList();
  return out.join("");
}
