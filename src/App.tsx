import { useEffect, useState } from "react";
import { StatusBar } from "./components/StatusBar/StatusBar";
import { MainWindow } from "./components/MainWindow/MainWindow";

function AppRouter() {
  const [label, setLabel] = useState("");

  useEffect(() => {
    import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      setLabel(getCurrentWindow().label);
    });
  }, []);

  if (label === "status-bar") return <StatusBar />;
  return <MainWindow />;
}

export default AppRouter;
