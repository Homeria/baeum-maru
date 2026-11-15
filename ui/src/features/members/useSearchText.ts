import { useEffect, useState, type ChangeEvent } from "react";

/** 입력이 멈춘 뒤 검색하기까지 기다리는 시간. */
const DELAY_MS = 300;

/** 한글 낱자(ㄱ~ㅎ, ㅏ~ㅣ). 글자를 조합하는 중에 끝에 남는다. */
const TRAILING_JAMO = /[ㄱ-ㆎ]+$/;

/**
 * 검색창 글자에서 검색어를 만든다. 앞뒤 공백과 끝에 남은 한글 낱자를 뺀다.
 *
 * Windows 한글 입력기는 마지막 글자를 다음 키를 누를 때까지 조합 중으로 둔다.
 * 조합이 끝나기를 기다리면 "홍길동"을 치고 멈춰도 찾지 않으므로, 조합 중인 글자도 쓰되
 * `홍길ㄷ`처럼 덜 된 낱자만 뺀다.
 */
export function searchQueryFrom(text: string): string {
  return text.trim().replace(TRAILING_JAMO, "").trim();
}

/** 검색창 글자와, 입력이 0.3초 멈췄을 때만 바뀌는 검색어. */
export function useSearchText() {
  const [text, setText] = useState("");
  const [query, setQuery] = useState("");

  useEffect(() => {
    const timer = setTimeout(() => setQuery(searchQueryFrom(text)), DELAY_MS);
    return () => clearTimeout(timer);
  }, [text]);

  return {
    query,
    inputProps: {
      value: text,
      onChange: (event: ChangeEvent<HTMLInputElement>) =>
        setText(event.target.value),
    },
  };
}
