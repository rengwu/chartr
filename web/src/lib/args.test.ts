import { describe, expect, it } from 'vitest'
import { formatArgs, parseArgs } from './args'

describe('parseArgs', () => {
  it('splits on whitespace, which is the whole common case', () => {
    expect(parseArgs('--model sonnet --dangerously-skip-permissions')).toEqual([
      '--model',
      'sonnet',
      '--dangerously-skip-permissions',
    ])
  })

  it('treats runs of whitespace as one separator and ignores the edges', () => {
    expect(parseArgs('   -m   big \n --yolo  ')).toEqual(['-m', 'big', '--yolo'])
  })

  it('is empty for an empty field, not a list holding an empty string', () => {
    expect(parseArgs('')).toEqual([])
    expect(parseArgs('   ')).toEqual([])
  })

  it('keeps a quoted argument whole — the reason this is not a plain split', () => {
    expect(parseArgs('--add-dir "/Users/me/my projects" --fast')).toEqual([
      '--add-dir',
      '/Users/me/my projects',
      '--fast',
    ])
    expect(parseArgs("--system-prompt 'be terse'")).toEqual(['--system-prompt', 'be terse'])
  })

  it('lets a quote be typed inside another, or escaped', () => {
    expect(parseArgs(`--say "it's fine"`)).toEqual(['--say', "it's fine"])
    expect(parseArgs('--say "a \\"quoted\\" word"')).toEqual(['--say', 'a "quoted" word'])
  })

  it('joins quoted and bare halves of one argument, as a shell does', () => {
    expect(parseArgs('--dir="/my path"')).toEqual(['--dir=/my path'])
  })

  it('keeps an explicitly empty argument', () => {
    expect(parseArgs('--flag ""')).toEqual(['--flag', ''])
  })

  it('expands nothing — a flag reaches the agent exactly as typed', () => {
    expect(parseArgs('--path $HOME/*.md --price 100%')).toEqual([
      '--path',
      '$HOME/*.md',
      '--price',
      '100%',
    ])
  })

  it('does not lose the tail of an unterminated quote', () => {
    expect(parseArgs('--say "unfinished')).toEqual(['--say', 'unfinished'])
  })
})

describe('formatArgs', () => {
  it('leaves ordinary flags unquoted', () => {
    expect(formatArgs(['--model', 'sonnet'])).toBe('--model sonnet')
  })

  it('is empty for no args', () => {
    expect(formatArgs([])).toBe('')
    expect(formatArgs(undefined)).toBe('')
  })

  it('quotes only what would otherwise come back different', () => {
    expect(formatArgs(['--add-dir', '/my projects'])).toBe('--add-dir "/my projects"')
    expect(formatArgs(['a"b'])).toBe('"a\\"b"')
    expect(formatArgs([''])).toBe('""')
  })
})

// The round trip is the contract the editor rests on: an agent registered with a
// space (or a quote) in one of its flags can be opened and saved without quietly
// losing it.
describe('round trip', () => {
  const lists = [
    [],
    ['--model', 'sonnet'],
    ['--dangerously-skip-permissions'],
    ['--add-dir', '/Users/me/my projects', '--fast'],
    ['--say', "it's fine"],
    ['--say', 'a "quoted" word'],
    ['--path', '$HOME/*.md'],
    ['--empty', ''],
    ['back\\slash'],
  ]
  for (const list of lists) {
    it(`survives ${JSON.stringify(list)}`, () => {
      expect(parseArgs(formatArgs(list))).toEqual(list)
    })
  }
})
