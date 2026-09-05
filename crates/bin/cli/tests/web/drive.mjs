// Drive the real web app against a running Node, then report what happened.
//
// argv: <api-client.js> <app.js> <base-url> <name> <handle> <visibility>
import { resolveShim } from './loader.mjs';
import * as dom from './dom.mjs';

const [client, app, base, name, handle, visibility] = process.argv.slice(2);

resolveShim(client);

// The page is served from the Node's origin, so its client defaults to a
// relative base. Under `node` there is no origin, so `fetch` needs an absolute
// one — supplied here and nowhere else, because a base URL is the one thing a
// page gets from the browser rather than from its own code.
const realFetch = globalThis.fetch;
globalThis.fetch = (url, init) => realFetch(url.startsWith('http') ? url : base + url, init);

dom.fields({ name, handle, visibility });

await import(app);          // registers the submit handler, runs boot()
await new Promise((r) => setTimeout(r, 50));
await dom.submit();

const err = dom.errorText();
if (dom.missing.length) {
  console.error('the DOM shim is missing: ' + [...new Set(dom.missing)].join(', ') +
    ' — app.js reached for something this harness does not implement, so the run below proves nothing');
  process.exit(2);
}
if (err.trim()) {
  console.error('web surface reported an error: ' + err);
  process.exit(1);
}
console.log('ok');
