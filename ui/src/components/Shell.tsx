import type { ReactNode } from "react";
import { ROUTES } from "../routes";
import type { ViewId } from "../types";

interface ShellProps {
  activeView: ViewId;
  onNavigate: (view: ViewId) => void;
  children: ReactNode;
}

export function Shell({ activeView, onNavigate, children }: ShellProps) {
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div>
            <div className="brand__eyebrow">Hope</div>
            <h1>Desktop Workbench</h1>
          </div>
          <span className="brand__badge">Internal Trial</span>
        </div>

        <nav className="nav">
          {ROUTES.map((route) => (
            <button
              key={route.id}
              type="button"
              className={route.id === activeView ? "nav__item nav__item--active" : "nav__item"}
              onClick={() => onNavigate(route.id)}
            >
              <span className="nav__label">{route.label}</span>
              <span className="nav__description">{route.description}</span>
            </button>
          ))}
        </nav>

        <section className="sidebar__note">
          <strong>Trial Boundary</strong>
          <p>
            Desktop prefers packaged IPC when it is available. Browser preview falls back to local fixture data. This shell does not call live Qwen, Seedance, or SQLite directly.
          </p>
        </section>
      </aside>

      <main className="workspace">{children}</main>
    </div>
  );
}
