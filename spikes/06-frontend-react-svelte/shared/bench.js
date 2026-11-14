// 버린 코드: 자동 측정. 두 앱에서 같은 조작을 같은 순서로 하고 시간을 잰다.
//
// 앱은 registerBench({ name, setQuery, replaceMembers })를 부른다.
// - setQuery(q): 검색어 바꾸기
// - replaceMembers({ rev, list }): 서버 알림 뒤 목록을 새로 받은 것처럼 바꾸기
// - flush(fn): fn 안의 상태 변경을 즉시 DOM에 반영 (화면 도구 자체의 처리 시간을 프레임과 상관없이 재기 위해)
// 앱은 표에 data-q(검색어)와 data-rev(목록 번호)를 달아 둔다. 측정은 그 값이 바뀔 때까지 기다린다.
import { makeMembers, filterMembers, changeOne } from './data.js';

const nextFrame = () => new Promise((r) => requestAnimationFrame(() => r()));
const afterPaint = () => new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));

async function waitTable(q, rev) {
  for (;;) {
    const t = document.querySelector('table');
    if (t && t.dataset.q === q && t.dataset.rev === String(rev)) break;
    await nextFrame();
  }
  await afterPaint(); // 바뀐 표가 실제로 화면에 그려질 때까지
}

function stats(samples) {
  const s = [...samples].sort((a, b) => a - b);
  const avg = s.reduce((a, b) => a + b, 0) / s.length;
  const p95 = s[Math.min(s.length - 1, Math.round((s.length - 1) * 0.95))];
  return { avg: +avg.toFixed(1), p95: +p95.toFixed(1), max: +s[s.length - 1].toFixed(1) };
}

const QUERIES = ['김', '김가', '김가람', '', '이', '이나', '', '0012', '00123', '', '하늘', '', '010-1', ''];

export function memberCount() {
  const n = Number(new URLSearchParams(location.search).get('n'));
  return Number.isFinite(n) && n > 0 ? n : 1000;
}

/** ?limit=100 처럼 주면 표에 그 수만큼만 보여준다 (페이지 나누기 흉내) */
export function rowLimit() {
  const n = Number(new URLSearchParams(location.search).get('limit'));
  return Number.isFinite(n) && n > 0 ? n : Infinity;
}

export async function registerBench(api) {
  await waitTable('', 0);
  const firstPaint = performance.now(); // 페이지를 열고 표가 처음 보일 때까지
  window.__bench = { api, firstPaint };
  const out = document.getElementById('bench-result');
  if (out) out.textContent = `첫 표시 ${firstPaint.toFixed(0)}ms. "자동 측정"을 누르세요.`;
  if (new URLSearchParams(location.search).get('auto') === '1') setTimeout(runBench, 300);
}

export async function runBench() {
  const { api, firstPaint } = window.__bench;
  const out = document.getElementById('bench-result');
  if (out) out.textContent = '측정 중… (화면을 건드리지 마세요)';
  let current = makeMembers(memberCount());
  let rev = 0;

  // 검색: 검색어를 바꾸고 표가 다시 그려질 때까지
  const search = [];
  for (let round = 0; round < 4; round++) {
    for (const q of QUERIES) {
      const t0 = performance.now();
      api.setQuery(q);
      await waitTable(q, rev);
      if (round > 0) search.push(performance.now() - t0); // 첫 바퀴는 예열
    }
  }

  // 알림 갱신: 전체 목록(검색어 없음)을 새로 받아 다시 그릴 때까지. JSON 해석 시간 포함.
  api.setQuery('');
  await waitTable('', rev);
  const refresh = [];
  for (let k = 1; k <= 23; k++) {
    const json = JSON.stringify(changeOne(current, k)); // 서버가 보낸 응답이라고 침
    const t0 = performance.now();
    current = JSON.parse(json);
    rev += 1;
    api.replaceMembers({ rev, list: current });
    await waitTable('', rev);
    if (k > 3) refresh.push(performance.now() - t0);
  }

  // 화면 도구의 순수 처리 시간: 상태를 바꾸고 DOM이 고쳐질 때까지 (그리기 전, 프레임 대기 없음)
  const searchDom = [];
  for (let round = 0; round < 4; round++) {
    for (const q of QUERIES) {
      const t0 = performance.now();
      api.flush(() => api.setQuery(q));
      if (round > 0) searchDom.push(performance.now() - t0);
      await afterPaint();
    }
  }
  api.flush(() => api.setQuery(''));
  await afterPaint();
  const refreshDom = [];
  for (let k = 101; k <= 123; k++) {
    const json = JSON.stringify(changeOne(current, k));
    const t0 = performance.now();
    current = JSON.parse(json);
    rev += 1;
    const d = { rev, list: current };
    api.flush(() => api.replaceMembers(d));
    if (k > 103) refreshDom.push(performance.now() - t0);
    await afterPaint();
  }

  const result = {
    framework: api.name,
    members: current.length,
    firstPaintMs: +firstPaint.toFixed(0),
    searchMs: stats(search),
    refreshMs: stats(refresh),
    searchDomMs: stats(searchDom),
    refreshDomMs: stats(refreshDom),
    shownRowsAfterSearch김: filterMembers(current, '김').length,
    userAgent: navigator.userAgent,
  };
  window.__benchResult = result;
  if (out) out.textContent = JSON.stringify(result, null, 2);
  return result;
}
