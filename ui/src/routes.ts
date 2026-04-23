import type { ViewId } from "./types";

export interface RouteEntry {
  id: ViewId;
  hash: string;
  label: string;
  description: string;
}

export const ROUTES: RouteEntry[] = [
  {
    id: "workbench",
    hash: "#/workbench",
    label: "Hope 工作台",
    description: "单页中文工作台，融合脚本整理、分镜产出和桌面端导出。",
  },
];

export function resolveRoute(hash: string): ViewId {
  const match = ROUTES.find((route) => route.hash === hash);
  return match?.id ?? "workbench";
}