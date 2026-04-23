import type { ViewId } from "./types";

export interface RouteEntry {
  id: ViewId;
  hash: string;
  label: string;
  description: string;
}

export const ROUTES: RouteEntry[] = [
  {
    id: "projects",
    hash: "#/projects",
    label: "Projects",
    description: "Select the working project and confirm the active desktop context.",
  },
  {
    id: "writer",
    hash: "#/writer",
    label: "Script Workbench",
    description: "Capture the brief, review writing layers, and prepare storyboard input.",
  },
  {
    id: "preview",
    hash: "#/preview",
    label: "Storyboard Review",
    description: "Check frames, motion, and cut continuity before delivery.",
  },
  {
    id: "export",
    hash: "#/export",
    label: "Export Center",
    description: "Review validation and keep Excel delivery on track.",
  },
];

export function resolveRoute(hash: string): ViewId {
  const match = ROUTES.find((route) => route.hash === hash);
  return match?.id ?? "projects";
}
