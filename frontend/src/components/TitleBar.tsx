
import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export default function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const updateState = async () => {
      setIsMaximized(await appWindow.isMaximized());
    };
    updateState();

    
    const interval = setInterval(updateState, 1000);
    return () => clearInterval(interval);
  }, [appWindow]);

  return (
      <div
          data-tauri-drag-region
          className="titlebar fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-3 bg-slate-900 border-b border-slate-700"
      >
        {}
        <div className="flex items-center gap-3 pointer-events-none">
          <div className="w-3 h-3 rounded-sm bg-sky-500 shadow-[0_0_8px_rgba(14,165,233,0.4)]" />
          <span className="text-xs font-bold tracking-widest text-slate-300 uppercase">
          TruthByte <span className="text-slate-600">v1.0</span>
        </span>
        </div>

        {}
        <div className="flex items-center gap-1">
          <button
              onClick={() => appWindow.minimize()}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded transition-colors"
              title="Minimize"
          >
            <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M5 12h14" />
            </svg>
          </button>
          <button
              onClick={() => appWindow.toggleMaximize()}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded transition-colors"
              title="Maximize"
          >
            {isMaximized ? (
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect width="14" height="14" x="5" y="5" rx="2" ry="2" />
                </svg>
            ) : (
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect width="18" height="18" x="3" y="3" rx="2" ry="2" />
                </svg>
            )}
          </button>
          <button
              onClick={() => appWindow.close()}
              className="p-2 text-slate-400 hover:text-white hover:bg-rose-600 rounded transition-colors"
              title="Close"
          >
            <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        </div>
      </div>
  );
}