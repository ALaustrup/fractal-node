// PH0 walking skeleton.
//
// P3, and the reason this file has no fetch call in it: the client is GENERATED
// from crates/support/schema by `cargo xtask codegen`. A client cannot reach an
// endpoint the contract does not describe, and it cannot fall behind one the
// contract adds — the build fails on drift before anyone notices.

import { createClient, FractalError } from '/api-client.js';

const api = createClient();

const $ = (id) => document.getElementById(id);

function setState(kind, label) {
  const dot = $('dot');
  dot.className = `dot ${kind}`;
  $('state').textContent = label;
  $('sync').textContent = label;
}

function showError(e) {
  const err = e instanceof FractalError ? e.error : e;
  const el = $('error');
  // Cause, then remedy, no apology (docs/33 §7.3).
  const remedy = err?.remedy?.human;
  el.innerHTML = '';
  el.appendChild(document.createTextNode(err?.detail || err?.title || 'Refused.'));
  if (remedy) {
    const span = document.createElement('span');
    span.className = 'remedy';
    span.textContent = remedy;
    el.appendChild(span);
  }
  el.hidden = false;
}

function clearError() {
  $('error').hidden = true;
}

function render(societies) {
  const list = $('list');
  list.innerHTML = '';
  $('count').textContent = societies.length
    ? `${societies.length} ${societies.length === 1 ? 'SOCIETY' : 'SOCIETIES'}`
    : '';
  if (!societies.length) {
    const p = document.createElement('p');
    p.className = 'empty';
    p.textContent = 'Nothing here yet. Found the first one above — it costs nothing and it is yours.';
    list.appendChild(p);
    return;
  }
  for (const s of societies) {
    const row = document.createElement('div');
    row.className = 'row';

    const left = document.createElement('div');
    const name = document.createElement('div');
    name.className = 'name';
    name.textContent = s.name;
    const id = document.createElement('div');
    id.className = 'id';
    id.textContent = s.society_id;
    left.append(name, id);

    const handle = document.createElement('div');
    handle.className = 'handle';
    handle.textContent = s.handle;

    const members = document.createElement('div');
    members.className = 'members';
    members.textContent = s.member_count;

    row.append(left, handle, members);
    list.appendChild(row);
  }
}

async function refresh() {
  try {
    const data = await api.societyList();
    render(data.societies ?? []);
  } catch (err) {
    showError(err);
  }
}

async function boot() {
  try {
    const h = await api.nodeStatus();
    setState('live', 'LIVE');
    $('runtime').textContent = `RUNTIME ${h.runtime} · API ${h.api_version}`;
    $('path').textContent = 'FN://NODE/SOCIETIES';
  } catch {
    setState('down', 'OFFLINE');
    $('runtime').textContent = 'NO RUNTIME';
    // P2 in miniature: the page still renders, it just says so.
  }
  await refresh();
}

$('form').addEventListener('submit', async (e) => {
  e.preventDefault();
  clearError();
  const btn = $('submit');
  btn.disabled = true;
  try {
    await api.societyCreate({
      name: $('name').value.trim(),
      handle: $('handle').value.trim(),
      visibility: new FormData(e.target).get('visibility'),
      // Makes the command safe to retry — the same key the CLI sends.
      idempotency_key: `web-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`,
    });
    e.target.reset();
    await refresh();
  } catch (err) {
    showError(err);
  } finally {
    btn.disabled = false;
  }
});

boot();
