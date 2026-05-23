// Entry worker: routes requests between static assets and SSR.
// Static assets are embedded inline. SSR is delegated via service binding.

import assets from "static-assets.mjs";

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const pathname = url.pathname;

    // Serve static assets for known file types
    if (pathname.startsWith("/_nuxt/") || pathname.match(/\.(js|css|json|svg|png|jpg|jpeg|gif|ico|woff2?|ttf|eot|webp|avif|txt|xml)$/)) {
      const asset = assets.get(pathname);
      if (asset) {
        return new Response(
          Uint8Array.from(atob(asset.data), c => c.charCodeAt(0)),
          {
            headers: {
              "Content-Type": asset.mime,
              "Cache-Control": "public, max-age=31536000, immutable",
            },
          }
        );
      }
    }

    // Delegate to SSR handler via service binding
    return env.SSR.fetch(request);
  },
};
