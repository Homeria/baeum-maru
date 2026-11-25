import { useQuery } from "@tanstack/react-query";
import { getMember } from "../../api/members";
import { MemberForm } from "./MemberForm";
import styles from "../../components/form/form.module.css";

/** {@link MemberEditor}의 속성. */
export interface MemberEditorProps {
  id: number;
  onClose: (notice?: string) => void;
}

/** 고를 회원을 호스트에서 새로 받아 고치기 칸에 채운다. */
export function MemberEditor({ id, onClose }: MemberEditorProps) {
  const { data, isPending, isError, error } = useQuery({
    queryKey: ["members", "detail", id],
    queryFn: () => getMember(id),
  });

  if (isPending) {
    return <p className={styles.loading}>회원 정보를 불러오는 중입니다…</p>;
  }
  if (isError) {
    return (
      <div className={styles.form}>
        <p className={styles.formError} role="alert">
          {error.message}
        </p>
        <button
          type="button"
          className={styles.secondary}
          onClick={() => onClose()}
        >
          새 회원 등록으로
        </button>
      </div>
    );
  }
  return <MemberForm editing={data} onClose={onClose} />;
}
