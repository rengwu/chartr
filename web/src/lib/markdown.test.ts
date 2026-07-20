import { describe, it, expect } from 'vitest'
import { renderMarkdown, sectionOf } from './markdown'

describe('renderMarkdown', () => {
  it('renders headings demoted, paragraphs, and lists', () => {
    const html = renderMarkdown('## Question\n\nDo the thing.\n\n- one\n- two')
    expect(html).toContain('<h4>Question</h4>')
    expect(html).toContain('<p>Do the thing.</p>')
    expect(html).toContain('<ul>')
    expect(html).toContain('<li>one</li>')
    expect(html).toContain('<li>two</li>')
  })

  it('applies inline bold, italic, code, and links', () => {
    const html = renderMarkdown('a **bold** and *em* and `code` and [w](https://x.io)')
    expect(html).toContain('<strong>bold</strong>')
    expect(html).toContain('<em>em</em>')
    expect(html).toContain('<code>code</code>')
    expect(html).toContain('<a href="https://x.io" target="_blank" rel="noopener noreferrer">w</a>')
  })

  it('escapes HTML in the source — no markup can reach the DOM', () => {
    const html = renderMarkdown('a <script>alert(1)</script> & <b>x</b>')
    expect(html).not.toContain('<script>')
    expect(html).toContain('&lt;script&gt;')
    expect(html).toContain('&amp;')
  })

  it('refuses dangerous link schemes', () => {
    const html = renderMarkdown('[x](javascript:alert(1))')
    expect(html).not.toContain('javascript:')
    expect(html).toContain('href="#"')
  })

  it('does not mistake plain spaced numbers for code placeholders', () => {
    // The code-span placeholder must not collide with prose like " 0 and".
    const html = renderMarkdown('roots at rank 0 and deeper at 5 too')
    expect(html).toBe('<p>roots at rank 0 and deeper at 5 too</p>')
  })

  it('renders fenced code verbatim and escaped', () => {
    const html = renderMarkdown('```\n<x> **not bold**\n```')
    expect(html).toContain('<pre><code>&lt;x&gt; **not bold**</code></pre>')
  })
})

describe('sectionOf', () => {
  const body =
    '## Question\nWhat to do.\n\nDone when: it works.\n\n## Answer\nWe did it.\nAcross two lines.\n'

  it('extracts a named section up to the next heading', () => {
    expect(sectionOf(body, ['Answer'])).toBe('We did it.\nAcross two lines.')
  })

  it('prefers the first matching name and returns empty when none match', () => {
    expect(sectionOf(body, ['Proposed Answer', 'Answer'])).toBe('We did it.\nAcross two lines.')
    expect(sectionOf(body, ['Ruled out'])).toBe('')
  })
})
