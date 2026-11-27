import { useQuery } from "@tanstack/react-query";
import { Drawer } from "../../../components/overlay/Drawer";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { getMember } from "./api";
import { MemberForm } from "./MemberForm";

/** {@link MemberEditor}의 속성. */
export interface MemberEditorProps {
  id: number;
  onSaved: (notice: string) => void;
  onDeleted: (notice: string) => void;
  onClose: () => void;
}

/** 고른 회원을 호스트에서 새로 받아 편집 서랍에 채운다. */
export function MemberEditor({
  id,
  onSaved,
  onDeleted,
  onClose,
}: MemberEditorProps) {
  const { data, isPending, isError, error, refetch } = useQuery({
    queryKey: ["members", "detail", id],
    queryFn: () => getMember(id),
  });

  if (isPending) {
    return (
      <Drawer
        open
        title="회원 정보 고치기"
        onClose={onClose}
        footerRight={<Button onClick={onClose}>닫기</Button>}
      >
        <NoticeBanner message="회원 정보를 불러오는 중입니다…" />
      </Drawer>
    );
  }

  if (isError) {
    return (
      <Drawer
        open
        title="회원 정보 고치기"
        onClose={onClose}
        footerRight={<Button onClick={onClose}>닫기</Button>}
      >
        <NoticeBanner
          tone="error"
          message={error.message}
          actionLabel="다시 해 보기"
          onAction={() => void refetch()}
        />
      </Drawer>
    );
  }

  return (
    <MemberForm
      editing={data}
      onSaved={onSaved}
      onDeleted={onDeleted}
      onClose={onClose}
    />
  );
}
