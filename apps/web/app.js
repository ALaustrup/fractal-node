// PH0 walking skeleton.
//
// P3, and the reason this file has no fetch call in it: the client is GENERATED
// from crates/support/schema by `cargo xtask codegen`. A client cannot reach an
// endpoint the contract does not describe, and it cannot fall behind one the
// contract adds — the build fails on drift before anyone notices.

import { createClient, FractalError } from '/api-client.js';

// The warning sink. Anything the Runtime flags on a successful response lands
// here, for every operation, without a call site having to ask for it.
const warnings = new Set();
const api = createClient({
  onWarning: (w) => {
    warnings.add(w);
    renderWarnings();
  },
});

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

// PH0 puts one real thing in the warnings channel — that the founder's identity
// was asserted rather than proven — and showing it is deliberate: docs/00 P12
// asks the system to be honest about what it is, and a walking skeleton that
// looks finished is the dishonest kind.
function renderWarnings() {
  const el = $('warnings');
  el.innerHTML = '';
  for (const w of warnings) {
    const li = document.createElement('li');
    li.textContent = w;
    el.appendChild(li);
  }
  el.hidden = warnings.size === 0;
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
      // Sent RAW. Trimming here is normalisation, normalisation is the
      // contract's job, and doing it in one front end is how this surface
      // came to accept a handle the CLI and the API both refused.
      name: $('name').value,
      handle: $('handle').value,
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
