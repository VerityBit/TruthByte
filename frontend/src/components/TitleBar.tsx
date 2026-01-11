import { getCurrentWindow } from "@tauri-apps/api/window";

const TitleBar = () => {
  const appWindow = getCurrentWindow();

  const handleMinimize = async () => {
    await appWindow.minimize();
  };

  const handleMaximize = async () => {
    await appWindow.toggleMaximize();
  };

  const handleClose = async () => {
    await appWindow.close();
  };

  return (
    <div className="titlebar fixed left-0 right-0 top-0 z-30">
      <div className="mx-auto flex h-full max-w-6xl items-center justify-between px-6">
        <div
          className="flex flex-1 items-center gap-3"
          data-tauri-drag-region
          onDoubleClick={handleMaximize}
        >
          <span className="titlebar-dot titlebar-dot--mint" />
          <span className="titlebar-dot titlebar-dot--sky" />
          <span className="titlebar-dot titlebar-dot--ember" />
          <span className="text-xs uppercase tracking-[0.35em] text-ink-300">
            TruthByte
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            className="titlebar-button"
            onClick={handleMinimize}
            type="button"
            aria-label="Minimize window"
          >
            -
          </button>
          <button
            className="titlebar-button"
            onClick={handleMaximize}
            type="button"
            aria-label="Maximize window"
          >
            []
          </button>
          <button
            className="titlebar-button titlebar-button--danger"
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
