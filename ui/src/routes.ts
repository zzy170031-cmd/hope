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
    label: "项目创建 / 切换",
    description: "选择、创建或切换当前工作项目。",
  },
  {
    id: "writer",
    hash: "#/writer",
    label: "Writer 四层查看",
    description: "Synopsis -> Story -> Screenplay -> Storyboard。",
  },
  {
    id: "preview",
    hash: "#/preview",
    label: "Storyboard / RenderSegment / Cuts 预览",
    description: "查看三层预览入口与当前占位状态。",
  },
  {
    id: "export",
    hash: "#/export",
    label: "Export / Validation 面板",
    description: "查看导出与校验面板的最小入口。",
  },
];

export function resolveRoute(hash: string): ViewId {
  const match = ROUTES.find((route) => route.hash === hash);
  return match?.id ?? "projects";
}
