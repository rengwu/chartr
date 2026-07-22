// A small, safe markdown renderer for the detail pane (ticket 07). The pane
// shows ticket bodies and map material — the operator's own repo markdown — and
// the codebase favours hand-rolled over pulling a dependency (ADR 0010's ethos),
// so rather than add a markdown library plus a sanitiser this renders the narrow
// subset wayfinder docs actually use.
//
// Safety is by construction: every character is HTML-escaped first, then a fixed
// set of block and inline transforms is applied to the already-escaped text, so
// no markup in the source can ever reach the DOM as markup. Link targets are
// additionally restricted to http(s), mailto, and relative URLs.

const SENT = String.fromCharCode(0) // NUL placeholder delimiter, never present in rendered text

function escapeHTML(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

// A safe href from an already-escaped candidate: http(s)/mailto/relative pass;
// anything else (javascript:, data:, …) collapses to '#'.
function safeHref(escaped: string): string {
  const raw = escaped.replace(/&amp;/g, '&')
  if (/^(https?:\/\/|mailto:|\/|#|\.\/|\.\.\/)/i.test(raw)) return escaped
  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) return '#' // some other scheme — refuse
  return escaped // bare relative path
}

// Inline formatting over one already-escaped line: code spans first (their
// contents are then immune to further formatting via a sentinel placeholder),
// then links, bold, italic.
function inline(escaped: string): string {
  const codes: string[] = []
  let s = escaped.replace(/`([^`]+)`/g, (_m, c) => {
    codes.push('<code>' + c + '</code>')
    return SENT + (codes.length - 1) + SENT
  })

  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_m, text, href) => {
    return '<a href="' + safeHref(href) + '" target="_blank" rel="noopener noreferrer">' + text + '</a>'
  })
  s = s.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  s = s.replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1<em>$2</em>')
  s = s.replace(/(^|[^\w`])_([^_\n]+)_(?![\w])/g, '$1<em>$2</em>')

  s = s.replace(new RegExp(SENT + '(\\d+)' + SENT, 'g'), (_m, i) => codes[Number(i)])
  return s
}

/**
 * Render a markdown subset to safe HTML: headings, paragraphs, unordered lists
 * (one nesting level), fenced code blocks, blockquotes, horizontal rules, and
 * the inline set. Headings are demoted (`#`→h3 … capped at h6) so a rendered
 * body never competes with the pane's own title.
 */
export function renderMarkdown(src: string): string {
  const lines = (src ?? '').replace(/\r\n?/g, '\n').split('\n')
  const out: string[] = []

  let para: string[] = []
  let listDepth = 0 // 0 = no list, 1 = <ul>, 2 = nested <ul><ul>
  let inFence = false
  let fence: string[] = []

  const flushPara = () => {
    if (para.length) {
      out.push('<p>' + inline(escapeHTML(para.join(' '))) + '</p>')
      para = []
    }
  }
  const closeLists = (to: number) => {
    while (listDepth > to) {
      out.push('</ul>')
      listDepth--
    }
  }

  for (const line of lines) {
    const fenceMatch = /^\s*(```|~~~)/.test(line)
    if (inFence) {
      if (fenceMatch) {
        out.push('<pre><code>' + escapeHTML(fence.join('\n')) + '</code></pre>')
        fence = []
        inFence = false
      } else {
        fence.push(line)
      }
      continue
    }
    if (fenceMatch) {
      flushPara()
      closeLists(0)
      inFence = true
      continue
    }

    if (line.trim() === '') {
      flushPara()
      closeLists(0)
      continue
    }

    const heading = /^(#{1,6})\s+(.*)$/.exec(line)
    if (heading) {
      flushPara()
      closeLists(0)
      const level = Math.min(6, heading[1].length + 2)
      out.push('<h' + level + '>' + inline(escapeHTML(heading[2].trim())) + '</h' + level + '>')
      continue
    }

    if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(line)) {
      flushPara()
      closeLists(0)
      out.push('<hr>')
      continue
    }

    const bullet = /^(\s*)[-*+]\s+(.*)$/.exec(line)
    if (bullet) {
      flushPara()
      const depth = bullet[1].length >= 2 ? 2 : 1
      while (listDepth < depth) {
        out.push('<ul>')
        listDepth++
      }
      closeLists(depth)
      out.push('<li>' + inline(escapeHTML(bullet[2])) + '</li>')
      continue
    }

    const quote = /^>\s?(.*)$/.exec(line)
    if (quote) {
      flushPara()
      closeLists(0)
      out.push('<blockquote>' + inline(escapeHTML(quote[1])) + '</blockquote>')
      continue
    }

    // A continuation line indented under a list extends the last item's text.
    if (listDepth > 0 && /^\s+\S/.test(line)) {
      const last = out.length - 1
      if (out[last] && out[last].startsWith('<li>')) {
        out[last] = out[last].replace(/<\/li>$/, ' ' + inline(escapeHTML(line.trim())) + '</li>')
        continue
      }
    }

    para.push(line.trim())
  }

  if (inFence) out.push('<pre><code>' + escapeHTML(fence.join('\n')) + '</code></pre>')
  flushPara()
  closeLists(0)
  return out.join('\n')
}

/**
 * Extract the raw markdown body under the first matching `## <name>` heading, up
 * to the next `## ` heading. Used to pull a blocker's answer (Answer / Ruled
 * out) out of its inlined body for the blockers-inline section.
 * Returns '' when none of the names are present.
 */
export function sectionOf(body: string, names: string[]): string {
  const lines = (body ?? '').split('\n')
  const wanted = names.map((n) => '## ' + n)
  let start = -1
  for (let i = 0; i < lines.length; i++) {
    if (wanted.includes(lines[i].trim())) {
      start = i + 1
      break
    }
  }
  if (start < 0) return ''
  let end = lines.length
  for (let i = start; i < lines.length; i++) {
    if (/^##\s/.test(lines[i])) {
      end = i
      break
    }
  }
  return lines.slice(start, end).join('\n').trim()
}
