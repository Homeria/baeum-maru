import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// 매 테스트 뒤 DOM을 비워 테스트끼리 상태가 남지 않게 한다
afterEach(() => {
  cleanup();
});
