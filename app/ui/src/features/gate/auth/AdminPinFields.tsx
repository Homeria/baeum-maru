import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { TextField } from "../../../components/ui/TextField";
import authStyles from "./auth.module.css";

/** 고를 수 있는 관리자 한 명. */
export interface AdminChoice {
  id: number;
  name: string;
}

/** {@link AdminPinFields}의 속성. */
export interface AdminPinFieldsProps {
  /** 칸 id와 라디오 묶음 이름의 앞. 한 화면에 둘이 있어도 겹치지 않게. */
  idPrefix: string;
  admins: readonly AdminChoice[];
  staffId: number | null;
  onStaffId: (id: number) => void;
  pin: string;
  onPin: (pin: string) => void;
  /** 관리자나 PIN을 비운 채 눌렀다. */
  missing: boolean;
}

/** 관리자를 고르고 PIN을 넣는 칸들. 넘기기 취소, 넘겨받기(BR-H2, BR-H3), 실행 모드 바꾸기(BR-K17)에서 쓴다. */
export function AdminPinFields({
  idPrefix,
  admins,
  staffId,
  onStaffId,
  pin,
  onPin,
  missing,
}: AdminPinFieldsProps) {
  const pinId = `${idPrefix}-pin`;

  return (
    <div className={authStyles.stack}>
      <fieldset className={authStyles.fieldset}>
        <legend className={authStyles.legend}>관리자</legend>
        {admins.length === 0 ? (
          <NoticeBanner
            tone="error"
            message="쓸 수 있는 관리자 계정이 없습니다"
          />
        ) : (
          <div className={authStyles.names}>
            {admins.map((admin) => (
              <label key={admin.id} className={authStyles.name}>
                <input
                  type="radio"
                  name={`${idPrefix}-admin`}
                  checked={staffId === admin.id}
                  onChange={() => onStaffId(admin.id)}
                />
                {admin.name}
              </label>
            ))}
          </div>
        )}
      </fieldset>
      <TextField
        id={pinId}
        label="PIN"
        type="password"
        inputMode="numeric"
        autoComplete="off"
        maxLength={8}
        value={pin}
        error={missing ? "관리자를 고르고 PIN을 넣어 주세요" : undefined}
        onChange={(event) => onPin(event.target.value)}
      />
    </div>
  );
}
