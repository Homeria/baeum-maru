import { useState } from "react";
import type { RunMode } from "../../api/generated/RunMode";
import { ChangeRunModeDialog } from "./ChangeRunModeDialog";
import styles from "./runMode.module.css";

/** {@link RunModeSwitch}의 속성. */
export interface RunModeSwitchProps {
  /** 지금 모드. 다른 쪽으로 바꾸는 버튼을 보여 준다. */
  current: RunMode;
  /** 관리자 PIN을 확인해야 하는지(BR-K17). */
  needsPin: boolean;
}

/** 로그인 전 화면 구석의 "실행 모드 바꾸기" 버튼(ADR 0021). */
export function RunModeSwitch({ current, needsPin }: RunModeSwitchProps) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className={styles.switch}
        onClick={() => setOpen(true)}
      >
        {current === "host"
          ? "다른 PC에 접속하는 모드로 바꾸기"
          : "이 PC를 호스트로 바꾸기"}
      </button>
      {open && (
        <ChangeRunModeDialog
          target={current === "host" ? "remote" : "host"}
          needsPin={needsPin}
          onCancel={() => setOpen(false)}
        />
      )}
    </>
  );
}
