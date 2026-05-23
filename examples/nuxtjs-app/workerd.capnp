using Workerd = import "/workerd/workerd.capnp";

# workerd config for Nuxt.js with static asset serving via ASSETS binding.
#
# This config defines two services:
#   - "ssr": the Nitro SSR handler (bundled from .output/server/)
#   - "static": serves embedded static files from .output/public/
#
# The SSR code checks env.ASSETS.fetch(request) for /_nuxt/* paths.
# The "static" service is bound as ASSETS so this delegation works.

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .entryWorker),
    (name = "ssr", worker = .ssrWorker),
    (name = "static", worker = .staticWorker),
  ],
  sockets = [
    ( name = "http",
      address = "*:8787",
      http = (),
      service = "main"
    ),
  ],
);

const entryWorker :Workerd.Worker = (
  modules = [
    (name = "entry.mjs", esModule = embed "entry.mjs"),
    (name = "static-assets.mjs", esModule = embed "static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "SSR", service = "ssr"),
    (name = "STATIC", service = "static"),
  ],
);

const ssrWorker :Workerd.Worker = (
  modules = [
    (name = "bundle.mjs", esModule = embed ".output/server/bundle.mjs"),
  ],
  compatibilityDate = "2026-04-21",
  compatibilityFlags = ["nodejs_compat"],
  bindings = [
    (name = "ASSETS", service = "static"),
  ],
);

const staticWorker :Workerd.Worker = (
  modules = [
    (name = "static-assets.mjs", esModule = embed "static-assets.mjs"),
  ],
  compatibilityDate = "2026-04-21",
);
