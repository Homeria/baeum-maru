// 버린 코드: React 버전. 흔히 짜는 방식 그대로 (최적화용 memo 없이).
import { useEffect, useMemo, useState } from 'react';
import { flushSync } from 'react-dom';
import { makeMembers, filterMembers } from '../../shared/data.js';
import { registerBench, runBench, memberCount, rowLimit } from '../../shared/bench.js';

export default function App() {
  const [data, setData] = useState(() => ({ rev: 0, list: makeMembers(memberCount()) }));
  const [query, setQuery] = useState('');
  const matched = useMemo(() => filterMembers(data.list, query), [data, query]);
  const rows = useMemo(() => matched.slice(0, rowLimit()), [matched]);

  useEffect(() => {
    registerBench({ name: 'React', setQuery, replaceMembers: setData, flush: flushSync });
  }, []);

  return (
    <>
      <h1>실험 06 · React · 회원 {data.list.length}명</h1>
      <div className="bar">
        <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="이름, 회원번호, 연락처 검색" />
        <span>{matched.length}명 중 {rows.length}명 표시</span>
        <button onClick={() => runBench()}>자동 측정</button>
      </div>
      <pre id="bench-result">준비 중…</pre>
      <table data-q={query} data-rev={data.rev}>
        <thead>
          <tr><th>회원번호</th><th>이름</th><th>연락처</th><th>출생</th><th>성별</th><th>신청</th></tr>
        </thead>
        <tbody>
          {rows.map((m) => (
            <tr key={m.id} className={m.changed ? 'changed' : ''}>
              <td>{m.memberNo}</td><td>{m.name}</td><td>{m.phone}</td><td>{m.birthYear}</td><td>{m.gender}</td><td>{m.courses}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}
