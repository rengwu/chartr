// An agent's args are a *list* of strings on the wire — that is what execve
// takes, and what the chartr passes through untouched — but a list is a
// miserable thing to edit. So the surface edits one text field and this module is
// the seam between the two.
//
// It reads the field the way a shell would, which is the convention every
// operator typing `--model sonnet --add-dir ./vendor` already has in their
// fingers: whitespace separates arguments, and quotes hold one together. What it
// deliberately does *not* do is anything else a shell does — no globbing, no
// variable expansion, no operators. The text is split and unquoted, never
// interpreted, so a `$HOME` or a `*` in a flag reaches the agent exactly as
// typed.
//
// parse and format are inverses: format(parse(s)) is s up to whitespace
// normalisation, and parse(format(a)) is a for every list a. That round trip is
// what lets an agent registered with a space in one of its flags be edited
// without quietly losing it.

/** Split a command-line-ish string into arguments. Quotes group; nothing expands. */
export function parseArgs(text: string): string[] {
  const out: string[] = []
  let cur = ''
  let started = false // distinguishes an empty quoted arg from no arg at all
  let quote: '"' | "'" | null = null

  for (let i = 0; i < text.length; i++) {
    const c = text[i]
    if (quote) {
      // Inside double quotes a backslash escapes the quote and itself, so a
      // literal " can be typed. Single quotes are literal throughout, as in a
      // shell.
      if (quote === '"' && c === '\\' && (text[i + 1] === '"' || text[i + 1] === '\\')) {
        cur += text[++i]
      } else if (c === quote) {
        quote = null
      } else {
        cur += c
      }
      continue
    }
    if (c === '"' || c === "'") {
      quote = c
      started = true
      continue
    }
    if (/\s/.test(c)) {
      if (started) out.push(cur)
      cur = ''
      started = false
      continue
    }
    cur += c
    started = true
  }
  if (started) out.push(cur)
  return out
}

/** Render an argument list back into one editable line. */
export function formatArgs(args: string[] | undefined): string {
  return (args ?? []).map(quoteArg).join(' ')
}

// An argument needs quoting when it would otherwise come back as something else:
// empty, split on its own whitespace, or read as a quote of its own.
function quoteArg(arg: string): string {
  if (arg !== '' && !/[\s"'\\]/.test(arg)) return arg
  return '"' + arg.replace(/([\\"])/g, '\\$1') + '"'
}
