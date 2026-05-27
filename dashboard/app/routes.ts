import { type RouteConfig, index, layout, route } from "@react-router/dev/routes";

export default [
  layout("routes/_layout.tsx", [
    index("routes/home.tsx"),
    route("workspace", "routes/workspace.tsx"),
    route("workspace/:id", "routes/workspace_.$id.tsx"),
    route("deployments", "routes/deployments.tsx"),
    route("deployments/:id", "routes/deployments_.$id.tsx"),
    route("deployments/:id/deploy", "routes/deployments_.$id.deploy.tsx"),
  ]),
  route("*", "routes/$.tsx"),
] satisfies RouteConfig;
