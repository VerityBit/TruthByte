import { getCurrentWindow } from "@tauri-apps/api/window";

const TitleBar = () => {
  const appWindow = getCurrentWindow();

  const handleMinimize = async () => {
    try {
      await appWindow.minimize();
    } catch (error) {
      console.error("Failed to minimize:", error);
    }
  };

  const handleMaximize = async () => {
    try {
      await appWindow.toggleMaximize();
    } catch (error) {
      console.error("Failed to maximize:", error);
    }
  };

  const handleClose = async () => {
    try {
      await appWindow.close();
    } catch (error) {
      console.error("Failed to close:", error);
    }
  };

  return (
      <div
          className="titlebar !fixed !left-0 !right-0 !top-0 !z-50 flex h-[var(--titlebar-height)] !w-screen items-center justify-center select-none"
      >

        <div
            className="absolute inset-0 h-full w-full cursor-default"
            data-tauri-drag-region
            onDoubleClick={handleMaximize}
        />

        <div className="relative z-10 mx-auto flex h-full w-full max-w-6xl items-center justify-between px-6 pointer-events-none">

          <div className="flex items-center gap-3 pointer-events-auto">
            <span className="titlebar-dot titlebar-dot--mint" />
            <span className="titlebar-dot titlebar-dot--sky" />
            <span className="titlebar-dot titlebar-dot--ember" />
            <span className="text-xs uppercase tracking-[0.35em] text-ink-300">
            TruthByte
          </span>
          </div>

          <div className="flex items-center gap-2 pointer-events-auto">
            <button
                className="titlebar-button flex items-center justify-center hover:bg-ink-800/50 active:bg-ink-800 cursor-pointer transition-colors"
                onClick={handleMinimize}
                type="button"
                aria-label="Minimize window"
            >
              <span className="mb-2">_</span>
            </button>
            <button
                className="titlebar-button flex items-center justify-center hover:bg-ink-800/50 active:bg-ink-800 cursor-pointer transition-colors"
                onClick={handleMaximize}
                type="button"
                aria-label="Maximize window"
            >
              <span className="text-[10px]">□</span>
            </button>
            <button
                className="titlebar-button titlebar-button--danger flex items-center justify-center hover:bg-red-500/20 active:bg-red-500/30 cursor-pointer transition-colors"
                onClick={handleClose}
                type="button"
                aria-label="Close window"
            >
              x
            </button>
          </div>
        </div>
      </div>
  );
};

export default TitleBar;
