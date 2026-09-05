// The smallest DOM that `apps/web/app.js` actually touches.
//
// Not a browser and not pretending to be one. The point is to run the real
// module's real submit handler, so that WHAT THE WEB SURFACE SENDS is observed
// rather than assumed — the field names, the trimming, the visibility lookup.
// Rendering is stubbed; rendering is not what criterion 5 is about.
//
// Every element is a Proxy that RECORDS any property the app reaches for and
// this shim does not implement. Without that, a missing method surfaces as
// app.js's own catch-all — the literal string "Refused." with no cause — and
// the first version of this file lost twenty minutes to exactly that: `append`
// (variadic) was implemented as `appendChild` only, and the gap appeared not on
// create but on the refresh AFTER a create, once there was a row to render.
// A test harness that can fail silently is a test harness that will.

export const missing = [];

const IMPLEMENTED = new Set([
  'id', 'children', 'value', 'textContent', 'className', 'innerHTML', 'hidden',
  'disabled', 'appendChild', 'append', 'addEventListener', 'reset',
  '_listeners', '_fields', 'nodeValue',
]);

class El {
  constructor(id) {
    this.id = id;
    this.children = [];
    this.value = '';
    this.textContent = '';
    this.className = '';
    this.innerHTML = '';
    this.hidden = false;
    this.disabled = false;
    this._listeners = {};
  }
  appendChild(c) { this.children.push(c); return c; }
  append(...cs) { this.children.push(...cs); }
  addEventListener(kind, fn) { this._listeners[kind] = fn; }
  reset() { this.value = ''; }
}

function node(id) {
  return new Proxy(new El(id), {
    get(t, p) {
      if (typeof p === 'string' && !IMPLEMENTED.has(p) && !(p in t)) {
        missing.push(p);
      }
      return t[p];
    },
  });
}

const els = new Map();
function el(id) {
  if (!els.has(id)) els.set(id, node(id));
  return els.get(id);
}

globalThis.document = {
  getElementById: (id) => el(id),
  createElement: () => node('<created>'),
  createTextNode: (t) => ({ nodeValue: t }),
};

// Visibility reaches the handler the way the real one does: a named form field
// read through FormData, not a property the handler reads directly.
globalThis.FormData = class {
  constructor(form) { this._f = form; }
  get(name) { return this._f._fields?.[name] ?? null; }
};

export function fields(values) {
  el('name').value = values.name;
  el('handle').value = values.handle;
  el('form')._fields = { visibility: values.visibility };
}

export async function submit() {
  const f = el('form');
  const handler = f._listeners.submit;
  if (!handler) {
    throw new Error('app.js registered no submit handler — the web surface cannot create a Society');
  }
  await handler({ preventDefault() {}, target: f });
}

export function errorText() {
  return el('error').children.map((c) => c.nodeValue ?? c.textContent).join(' ');
}
