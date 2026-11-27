import styles from "./Tabs.module.css";

/** 탭 하나. */
export interface TabItem<K extends string> {
  key: K;
  label: string;
}

/** {@link Tabs}의 속성. */
export interface TabsProps<K extends string> {
  /** 탭 묶음의 이름. 탭 안에 탭이 또 있을 때 화면 읽기 프로그램이 구분하게 한다. */
  label?: string;
  items: readonly TabItem<K>[];
  selected: K;
  onSelect: (key: K) => void;
}

/** 가로로 늘어선 메뉴 탭. 고른 탭만 강조한다. */
export function Tabs<K extends string>({
  label,
  items,
  selected,
  onSelect,
}: TabsProps<K>) {
  return (
    <div role="tablist" aria-label={label} className={styles.tabList}>
      {items.map((item) => (
        <button
          key={item.key}
          type="button"
          role="tab"
          aria-selected={item.key === selected}
          className={styles.tab}
          onClick={() => onSelect(item.key)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
