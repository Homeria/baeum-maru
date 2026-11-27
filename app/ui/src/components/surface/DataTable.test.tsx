import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { Button } from "../ui/Button";
import { DataTable, type DataTableColumn } from "./DataTable";

interface SampleRoom {
  id: number;
  name: string;
  capacity: number;
}

const ROWS: readonly SampleRoom[] = [
  { id: 1, name: "101호", capacity: 20 },
  { id: 2, name: "202호", capacity: 15 },
];

test("표 이름·열 머리·행 내용을 시맨틱 테이블로 렌더링한다", () => {
  const columns: readonly DataTableColumn<SampleRoom>[] = [
    {
      key: "name",
      header: "강의실",
      render: (row) => row.name,
      isRowHeader: true,
    },
    {
      key: "capacity",
      header: "정원",
      render: (row) => `${row.capacity}명`,
      align: "right",
    },
  ];

  render(
    <DataTable
      caption="강의실 목록"
      columns={columns}
      rows={ROWS}
      getRowKey={(row) => row.id}
      selectedKey={2}
    />,
  );

  expect(screen.getByRole("region", { name: "강의실 목록" })).toHaveAttribute(
    "tabindex",
    "0",
  );
  const table = screen.getByRole("table", { name: "강의실 목록" });
  expect(
    within(table).getByRole("columnheader", { name: "강의실" }),
  ).toBeInTheDocument();
  expect(
    within(table).getByRole("rowheader", { name: "101호" }),
  ).toBeInTheDocument();

  const selectedRow = screen.getByText("202호").closest("tr");
  expect(selectedRow).toHaveAttribute("data-selected", "true");
});

test("행을 누르면 onRowClick이 호출되고, 동작 열의 버튼을 눌러도 행 클릭이 중복 호출되지 않는다", async () => {
  const user = userEvent.setup();
  const onRowClick = vi.fn();
  const onEdit = vi.fn();

  const columns: readonly DataTableColumn<SampleRoom>[] = [
    { key: "name", header: "강의실", render: (row) => row.name },
    {
      key: "actions",
      header: "",
      align: "right",
      isAction: true,
      render: (row) => (
        <Button size="sm" onClick={() => onEdit(row)}>
          고치기
        </Button>
      ),
    },
  ];

  render(
    <DataTable
      caption="강의실 목록"
      columns={columns}
      rows={ROWS}
      getRowKey={(row) => row.id}
      onRowClick={onRowClick}
    />,
  );

  await user.click(screen.getByText("101호"));
  expect(onRowClick).toHaveBeenCalledWith(ROWS[0]);
  expect(onEdit).not.toHaveBeenCalled();

  const editButtons = screen.getAllByRole("button", { name: "고치기" });
  await user.click(editButtons[1]);
  expect(onEdit).toHaveBeenCalledWith(ROWS[1]);
  expect(onRowClick).toHaveBeenCalledTimes(1);
});

test("빈 목록과 오류 상태를 안내 배너로 보여 준다", async () => {
  const user = userEvent.setup();
  const onRetry = vi.fn();
  const columns: readonly DataTableColumn<SampleRoom>[] = [
    { key: "name", header: "강의실", render: (row) => row.name },
  ];

  const { rerender } = render(
    <DataTable
      caption="강의실 목록"
      columns={columns}
      rows={[]}
      getRowKey={(row) => row.id}
      emptyMessage="등록된 강의실이 없습니다."
    />,
  );

  expect(screen.getByRole("status")).toHaveTextContent(
    "등록된 강의실이 없습니다.",
  );

  rerender(
    <DataTable
      caption="강의실 목록"
      columns={columns}
      rows={[]}
      getRowKey={(row) => row.id}
      error="강의실을 불러오지 못했습니다."
      onRetry={onRetry}
    />,
  );

  expect(screen.getByRole("alert")).toHaveTextContent(
    "강의실을 불러오지 못했습니다.",
  );
  await user.click(screen.getByRole("button", { name: "다시 해 보기" }));
  expect(onRetry).toHaveBeenCalledTimes(1);
});
