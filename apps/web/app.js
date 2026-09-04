// PH0 walking skeleton.
//
// P3, and the reason this file is boring: the web GUI talks to the SAME public
// API the CLI uses, with no private path and no privileged endpoint. In PH1 this
// is replaced by the GENERATED client from fractal-schema (docs/30), at which
// point hand-written fetch calls become a lint failure.

const api = {
  async call(method, path, body) {
    const res = await fetch(path, {
      method,
      headers: body ? { 'content-type': 'application/json' } : {},
      body: body ? JSON.stringify(body) : undefined,
    });
    const json = await res.json().catch(() => ({
      ok: false,
      error: { code: 'internal', title: 'The Node sent something that is not JSON', detail: '' },
    }));
    if (json.ok) return json.data;
    throw json.error ?? { code: 'internal', title: 'Refused', detail: '' };
  },
  health: () => api.call('GET', '/health'),
  list: () => api.call('GET', '/v1/societies'),
  create: (b) => api.call('POST', '/v1/societies', b),
};

const $ = (id) => document.getElementById(id);

function setState(kind, label) {
  const dot = $('dot');
  dot.className = `dot ${kind}`;
  $('state').textContent = label;
  $('sync').textContent = label;
}

function showError(err) {
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
    const data = await api.list();
    render(data.societies ?? []);
  } catch (err) {
    showError(err);
  }
}

async function boot() {
  try {
    const h = await api.health();
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
    await api.create({
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
