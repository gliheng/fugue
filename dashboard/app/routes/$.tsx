import { Button } from "@heroui/react";
import { Icon } from "@iconify/react";
import { Link } from "react-router";

export function meta() {
  return [{ title: "Not Found — Fugue" }];
}

export default function NotFound() {
  return (
    <div className="flex flex-col items-center justify-center min-h-[70vh] p-8 text-center">
      <div className="relative mb-6">
        <div className="flex items-center justify-center w-24 h-24 rounded-3xl bg-accent-soft">
          <Icon icon="lucide:cloud-off" className="w-12 h-12 text-accent" />
        </div>
        <div className="absolute -top-2 -right-2 flex items-center justify-center w-10 h-10 rounded-full bg-danger-soft">
          <Icon icon="lucide:question-mark" className="w-5 h-5 text-danger" />
        </div>
      </div>

      <h1 className="text-6xl font-bold tracking-tight mb-2">404</h1>
      <p className="text-xl text-muted mb-1">Page not found</p>
      <p className="text-sm text-muted max-w-md mb-8">
        The page you're looking for doesn't exist or has been moved.
      </p>

      <div className="flex items-center gap-3">
        <Button as={Link} to="/" color="primary" variant="solid">
          <Icon icon="lucide:home" className="w-4 h-4" />
          Back to Dashboard
        </Button>
        <Button as={Link} to="/deployments" color="default" variant="flat">
          <Icon icon="lucide:boxes" className="w-4 h-4" />
          Deployments
        </Button>
      </div>
    </div>
  );
}
