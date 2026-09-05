// A resolve hook, so `apps/web/app.js` can be executed UNMODIFIED.
//
// The page imports from '/api-client.js' — an absolute path on the Node that
// serves it. Under `node` that would resolve to the filesystem root. Rewriting
// the import in a copy would mean the test exercises a near-copy of the web
// app, and "near" is exactly where a divergence would hide. So the specifier
// is remapped instead, and the file under test is the file that ships.
import { register } from 'node:module';
import { pathToFileURL } from 'node:url';

export function resolveShim(clientPath) {
  register(
    `data:text/javascript,
     const target = ${JSON.stringify(pathToFileURL(clientPath).href)};
     export function resolve(spec, ctx, next) {
       if (spec === '/api-client.js') return { url: target, shortCircuit: true };
       return next(spec, ctx);
     }`,
    import.meta.url,
  );
}
