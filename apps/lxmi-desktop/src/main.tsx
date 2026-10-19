import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./app/App";
import "./styles/fonts";
import "./styles/tokens.css";
import "./styles/base.css";
import { isDemoMode } from "./api/client";

if (!isDemoMode) document.documentElement.dataset.windowFrame = "custom";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
