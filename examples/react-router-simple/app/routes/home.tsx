import type { Route } from "./+types/home";
import { Welcome } from "../welcome/welcome";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "React Router on Fugue" },
    { name: "description", content: "Welcome to React Router on Fugue!" },
  ];
}

export function loader({ context }: Route.LoaderArgs) {
  const env = (context.cloudflare?.env ?? context.env ?? {}) as Record<string, string>;
  return { message: env.VALUE_FROM_CLOUDFLARE ?? "Hello from Fugue!" };
}

export default function Home({ loaderData }: Route.ComponentProps) {
  return <Welcome message={loaderData.message} />;
}