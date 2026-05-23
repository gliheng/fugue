import { type RouteConfig, index, route } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  route("apps", "routes/apps.tsx"),
  route("apps/:id", "routes/apps_.$id.tsx"),
  route("apps/:id/deploy", "routes/apps_.$id.deploy.tsx"),
] satisfies RouteConfig;
