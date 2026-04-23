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
    label: "Hope 动漫分镜脚本生成工作台",
    description: "严格参考图对齐的单页中文工作台，涵盖脚本区、任务条、分镜表格、分页和导出区。",
  },
];

export function resolveRoute(hash: string): ViewId {
  const match = ROUTES.find((route) => route.hash === hash);
  return match?.id ?? "workbench";
}
