import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import type { MemberResponse } from "../../../api/generated/MemberResponse";
import { setSession, type Session } from "../../../api/session";
import {
  createMember,
  deleteMember,
  getMember,
  searchMembers,
  updateMember,
} from "./api";
import { MembersPage } from "./MembersPage";

vi.mock("./api", () => ({
  searchMembers: vi.fn(),
  getMember: vi.fn(),
  createMember: vi.fn(),
  updateMember: vi.fn(),
  deleteMember: vi.fn(),
}));

const SAMPLE_MEMBER: MemberResponse = {
  id: 1,
  memberNo: "M001",
  name: "홍길동",
  phoneNumber: "01012345678",
  birthDate: "1955-03-15",
  gender: "male",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

function renderPage(role: Session["staff"]["role"] = "admin") {
  setSession({
    token: "tok",
    staff: { id: 1, name: "직원1", role },
  });
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MembersPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(searchMembers).mockResolvedValue({
    items: [SAMPLE_MEMBER],
    total: 1,
    page: 1,
    pageSize: 20,
  });
  vi.mocked(getMember).mockResolvedValue(SAMPLE_MEMBER);
  vi.mocked(createMember).mockResolvedValue(SAMPLE_MEMBER);
  vi.mocked(updateMember).mockResolvedValue(SAMPLE_MEMBER);
  vi.mocked(deleteMember).mockResolvedValue(undefined);
});

test("회원 등록 서랍에서 입력 중 닫기를 누르면 확인하고, 등록하면 알림을 띄운다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  expect(await screen.findByText("홍길동")).toBeInTheDocument();
  expect(screen.getByText("010-1234-5678")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "회원 등록" }));
  const drawer = screen.getByRole("dialog", { name: "새 회원 등록" });

  await user.type(within(drawer).getByLabelText("회원번호"), "M002");
  await user.type(within(drawer).getByLabelText("이름"), "김영희");
  await user.type(within(drawer).getByLabelText("연락처"), "010-9876-5432");

  // 바꾼 내용이 있으면 닫기 시 확인 창이 뜬다
  await user.click(within(drawer).getByRole("button", { name: "그만두기" }));
  expect(
    screen.getByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "계속 편집" }));

  await user.click(within(drawer).getByRole("button", { name: "등록" }));

  expect(createMember).toHaveBeenCalledWith({
    memberNo: "M002",
    name: "김영희",
    phoneNumber: "010-9876-5432",
    birthDate: null,
    gender: null,
  });
  expect(await screen.findByRole("status")).toHaveTextContent(
    "김영희님을 등록했습니다",
  );
});

test("회원 고치기 서랍에서 지우기를 누르면 확인 대화상자 뒤에 삭제한다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  expect(await screen.findByText("홍길동")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "고치기" }));

  const drawer = await screen.findByRole("dialog", {
    name: "회원 정보 고치기",
  });
  expect(within(drawer).getByLabelText("이름")).toHaveValue("홍길동");

  await user.click(within(drawer).getByRole("button", { name: "지우기" }));
  const confirm = screen.getByRole("dialog", { name: "홍길동님을 지울까요?" });
  expect(
    within(confirm).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();

  await user.click(within(confirm).getByRole("button", { name: "지우기" }));
  expect(deleteMember).toHaveBeenCalledWith(1, expect.anything());
  expect(await screen.findByRole("status")).toHaveTextContent(
    "홍길동님을 지웠습니다",
  );
});

test("조회 전용 역할에게는 회원 등록과 고치기 동작을 숨긴다", async () => {
  const user = userEvent.setup();
  renderPage("read_only");

  expect(await screen.findByText("홍길동")).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "회원 등록" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "고치기" }),
  ).not.toBeInTheDocument();

  await user.click(screen.getByText("홍길동"));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});
