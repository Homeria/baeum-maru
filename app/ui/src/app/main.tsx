import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { getSession, subscribeSession } from "../api/session";
import { App } from "./App";
import "./global.css";

const queryClient = new QueryClient();

// 세션이 끝나면 받아 둔 회원 정보 등을 메모리에서 지운다. 다음 직원이 보지 않게
subscribeSession(() => {
  if (getSession() === null) {
    queryClient.clear();
  }
});

const root = document.getElementById("root");
if (!root) {
  throw new Error("index.html에 #root가 없습니다");
}

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </StrictMode>,
);
