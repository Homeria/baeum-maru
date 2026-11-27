import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Drawer } from "../../../components/overlay/Drawer";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { SelectField } from "../../../components/ui/SelectField";
import { TextField } from "../../../components/ui/TextField";
import { createRoom, deleteRoom, updateRoom } from "./api";
import { FLOORS, floorLabel } from "./floor";
import {
  buildingNames,
  firstErrorField,
  lastUsedBuilding,
  roomFormErrorsFrom,
  roomFormValuesFrom,
  toRoomRequest,
  type RoomField,
  type RoomFormErrors,
  type RoomFormValues,
} from "./roomFormData";
import styles from "./RoomsPage.module.css";

const BUILDING_LIST_ID = "room-building-names";

/** {@link RoomForm}의 속성. */
export interface RoomFormProps {
  /** 지금 있는 강의실. 건물 칸의 처음 값과 자동완성에 쓴다. */
  rooms: readonly RoomResponse[];
  /** 고칠 강의실. 없으면 새 강의실 등록. */
  editing?: RoomResponse;
  /** 저장이나 등록을 끝낸 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onSaved: (notice: string) => void;
  /** 강의실을 지운 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onDeleted?: (notice: string) => void;
  /** 저장하지 않고 서랍을 닫는다. */
  onClose: () => void;
}

/** 강의실 등록과 고치기를 맡는 오른쪽 편집 서랍. */
export function RoomForm({
  rooms,
  editing,
  onSaved,
  onDeleted,
  onClose,
}: RoomFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const initialValues: RoomFormValues = editing
    ? roomFormValuesFrom(editing)
    : { name: "", building: lastUsedBuilding(rooms), floor: "" };
  const [values, setValues] = useState<RoomFormValues>(() => initialValues);
  const [errors, setErrors] = useState<RoomFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const cancelDeleteRef = useRef<HTMLButtonElement>(null);
  const inputs = useRef<
    Partial<Record<RoomField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const isDirty =
    values.name !== initialValues.name ||
    values.building !== initialValues.building ||
    values.floor !== initialValues.floor;

  const showErrors = (error: unknown) => {
    const next = roomFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (nextValues: RoomFormValues) => {
      const request = toRoomRequest(nextValues);
      return editing
        ? updateRoom(editing.id, request).then((room) => room.name)
        : createRoom(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
      setErrors(NO_FORM_ERRORS);
      onSaved(editing ? `저장했습니다: ${name}` : `등록했습니다: ${name}`);
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteRoom,
    onSuccess: () => {
      setConfirmingDelete(false);
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
      (onDeleted ?? onSaved)(`지웠습니다: ${editing?.name ?? ""}`);
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (field: RoomField, value: string) => {
    setValues((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    save.mutate(values);
  };

  return (
    <Drawer
      open
      title={editing ? "강의실 고치기" : "새 강의실 등록"}
      subtitle={
        editing
          ? `${editing.building} · ${editing.name}`
          : "강의실 이름과 건물을 입력해 주세요"
      }
      isDirty={isDirty}
      busy={busy}
      onClose={onClose}
      cancelLabel="그만두기"
      footerLeft={
        editing ? (
          <Button
            variant="danger"
            disabled={busy}
            onClick={() => setConfirmingDelete(true)}
          >
            지우기
          </Button>
        ) : undefined
      }
      footerRight={
        <Button
          type="submit"
          form={formId}
          variant="primary"
          busy={save.isPending}
          busyLabel={editing ? "저장하는 중…" : "등록하는 중…"}
          disabled={remove.isPending}
        >
          {editing ? "저장" : "등록"}
        </Button>
      }
    >
      <form id={formId} className={styles.form} onSubmit={submit} noValidate>
        <TextField
          ref={(element) => {
            inputs.current.name = element;
          }}
          label="이름"
          required
          autoComplete="off"
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
          error={errors.fields.name}
        />

        <div>
          <TextField
            ref={(element) => {
              inputs.current.building = element;
            }}
            label="건물"
            required
            autoComplete="off"
            list={BUILDING_LIST_ID}
            value={values.building}
            onChange={(event) => change("building", event.target.value)}
            error={errors.fields.building}
          />
          <datalist id={BUILDING_LIST_ID}>
            {buildingNames(rooms).map((building) => (
              <option key={building} value={building} />
            ))}
          </datalist>
        </div>

        <SelectField
          ref={(element) => {
            inputs.current.floor = element;
          }}
          label="층"
          value={values.floor}
          onChange={(event) => change("floor", event.target.value)}
          error={errors.fields.floor}
        >
          <option value="">(층 없음)</option>
          {FLOORS.map((floor) => (
            <option key={floor} value={String(floor)}>
              {floorLabel(floor)}
            </option>
          ))}
        </SelectField>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>

      <ModalDialog
        open={Boolean(editing && confirmingDelete)}
        size="sm"
        title={`${editing?.name ?? ""} 강의실을 지울까요?`}
        busy={remove.isPending}
        initialFocusRef={cancelDeleteRef}
        onClose={() => setConfirmingDelete(false)}
        footer={
          <>
            <Button
              ref={cancelDeleteRef}
              variant="secondary"
              disabled={remove.isPending}
              onClick={() => setConfirmingDelete(false)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={remove.isPending}
              busyLabel="지우는 중…"
              onClick={() => {
                if (editing) {
                  remove.mutate(editing.id);
                }
              }}
            >
              지우기
            </Button>
          </>
        }
      >
        <p className={styles.confirmText}>
          지우면 되돌릴 수 없습니다. 계속할까요?
        </p>
      </ModalDialog>
    </Drawer>
  );
}
