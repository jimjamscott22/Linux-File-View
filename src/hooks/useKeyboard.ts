import { useEffect } from "react";
import { useFileStore } from "../store";

export function useKeyboard() {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.altKey || event.metaKey) return;
      const state = useFileStore.getState();
      let handled = true;
      if (event.ctrlKey) {
        if (event.key === "d") state.moveCursor(12);
        else if (event.key === "u") state.moveCursor(-12);
        else if (event.key.toLowerCase() === "r") void state.refresh();
        else handled = false;
      } else {
        switch (event.key) {
          case "j": case "ArrowDown": state.moveCursor(1); break;
          case "k": case "ArrowUp": state.moveCursor(-1); break;
          case "g": case "Home": state.selectFirst(); break;
          case "G": case "End": state.selectLast(); break;
          case "l": case "ArrowRight": case "Enter": void state.enter(); break;
          case "h": case "ArrowLeft": case "Backspace": void state.goParent(); break;
          case "~": void state.goHome(); break;
          case ".": void state.toggleHidden(); break;
          case "s": void state.cycleSort(); break;
          case "S": void state.toggleSortDirection(); break;
          case "F5": void state.refresh(); break;
          default: handled = false;
        }
      }
      if (handled) event.preventDefault();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
