const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { webcrypto } = require('node:crypto');
const bridge = fs.readFileSync(path.join(__dirname, '../src/web_plugin/bridge.js'), 'utf8');

function page() {
  const sent = [], events = new Map(), timers = new Map();
  let timerId = 0;
  const window = {
    ipc: { postMessage(encoded) { sent.push(JSON.parse(encoded)); } },
    addEventListener(event, callback) { events.set(event, callback); },
  };
  vm.runInNewContext(bridge, {
    window, TextEncoder, crypto: webcrypto,
    setTimeout(callback) { timers.set(++timerId, callback); return timerId; },
    clearTimeout(id) { timers.delete(id); },
  });
  return { window, sent, events, timers };
}

test('replies resolve or reject their request and release its timer', async () => {
  const p = page();
  const success = p.window.chartr.invoke('data.read', { path: 'note' });
  p.window.__chartrReply({ ...p.sent[0], ok: true, value: 'contents' });
  assert.equal(await success, 'contents');
  const failure = p.window.chartr.invoke('data.read', { path: null });
  p.window.__chartrReply({ ...p.sent[1], ok: false, error: 'Invalid host request' });
  await assert.rejects(failure, /Invalid host request/);
  assert.equal(p.timers.size, 0);
});

test('a reply from an old document cannot resolve a new request with the same id', async () => {
  const before = page(), after = page();
  const old = before.window.chartr.invoke('data.read', { path: 'old' });
  const current = after.window.chartr.invoke('data.read', { path: 'new' });
  assert.equal(before.sent[0].id, after.sent[0].id);
  assert.notEqual(before.sent[0].document, after.sent[0].document);
  after.window.__chartrReply({ ...before.sent[0], ok: true, value: 'wrong' });
  assert.equal(after.timers.size, 1);
  after.window.__chartrReply({ ...after.sent[0], ok: true, value: 'right' });
  assert.equal(await current, 'right');
  before.events.get('pagehide')();
  await assert.rejects(old, /document closed/);
  assert.equal(before.timers.size, 0);
});

test('missing replies and transport failures reject rather than leak promises', async () => {
  const p = page();
  const stalled = p.window.chartr.invoke('data.read', { path: 'note' });
  const [timer, expire] = p.timers.entries().next().value;
  p.timers.delete(timer);
  expire();
  await assert.rejects(stalled, /timed out/);
  p.window.ipc.postMessage = () => { throw new Error('transport closed'); };
  assert.equal(p.timers.size, 0);
  await assert.rejects(p.window.chartr.invoke('data.read', { path: 'note' }), /transport closed/);
  assert.equal(p.timers.size, 0);
});

test('invalid and oversized calls do not reach the host', async () => {
  const p = page();
  await assert.rejects(p.window.chartr.invoke(null), /must be a string/);
  await assert.rejects(p.window.chartr.invoke('data.write', { data: 'x'.repeat(1024 * 1024) }), /exceeds/);
  const circular = {}; circular.self = circular;
  await assert.rejects(p.window.chartr.invoke('data.write', circular), /circular/i);
  assert.equal(p.sent.length, 0);
  assert.equal(p.timers.size, 0);
});
