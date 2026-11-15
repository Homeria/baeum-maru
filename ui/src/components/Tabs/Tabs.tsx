import styles from "./Tabs.module.css";

/** 탭 하나. */
export interface TabItem<K extends string> {
  key: K;
  label: string;
}

/** {@link Tabs}의 속성. */
export interface TabsProps<K extends string> {
  items: readonly TabItem<K>[];
  selected: K;
  onSelect: (key: K) => void;
}

/** 가로로 늘어선 메뉴 탭. 고른 탭만 강조한다. */
export function Tabs<K extends string>({
  items,
  selected,
  onSelect,
}: TabsProps<K>) {
  return (
    <div role="tablist" className={styles.tabList}>
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
