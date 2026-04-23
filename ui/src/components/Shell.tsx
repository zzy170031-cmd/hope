import type { ReactNode } from "react";
interface ShellProps {
  children: ReactNode;
}

export function Shell({ children }: ShellProps) {
  return (
    <div className="app-shell">
      <main className="workspace workspace--single-page">{children}</main>
    </div>
  );
}
