// 버린 코드: Chrome을 화면 없이(headless) 띄워 두 앱의 자동 측정을 돌리고 결과를 모은다.
// 사용: node run-bench.mjs [반복 횟수] [CPU 감속 배수]   (google-chrome, Node 22 이상 필요)
//   예: node run-bench.mjs 3 4  → CPU를 4배 느리게 해서 오래된 PC를 흉내 낸다
//   예: node run-bench.mjs 3 4 100  → 거기에 표를 100줄만 보여준다 (페이지 나누기 흉내)
import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync, statSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { gzipSync } from 'node:zlib';

const RUNS = Number(process.argv[2] ?? 3);
const THROTTLE = Number(process.argv[3] ?? 1);
const LIMIT = process.argv[4] ? `&limit=${process.argv[4]}` : '';
const APPS = ['react', 'svelte'];
const SIZES = [1000, 5000];
const PORT = 9333;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function measure(file, n) {
  const profile = mkdtempSync(join(tmpdir(), 'spike06-'));
  const chrome = spawn('google-chrome', [
    '--headless=new', `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`,
    '--no-first-run', '--window-size=1280,900', 'about:blank',
  ], { stdio: 'ignore' });
  try {
    let target;
    for (let i = 0; i < 50 && !target; i++) {
      await sleep(200);
      try {
        const list = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
        target = list.find((t) => t.type === 'page');
      } catch { /* 아직 안 뜸 */ }
    }
    if (!target) throw new Error('Chrome에 연결하지 못함');
    const ws = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((r) => ws.addEventListener('open', r, { once: true }));
    let id = 0;
    const evaluate = (expression) => new Promise((res) => {
      const myId = ++id;
      const onMsg = (ev) => {
        const msg = JSON.parse(ev.data);
        if (msg.id === myId) { ws.removeEventListener('message', onMsg); res(msg.result?.result?.value); }
      };
      ws.addEventListener('message', onMsg);
      ws.send(JSON.stringify({ id: myId, method: 'Runtime.evaluate', params: { expression, returnByValue: true } }));
    });
    // 빈 페이지에서 CPU 감속을 먼저 걸고, 그다음 앱을 연다 (첫 표시 시간도 감속 상태로 재기 위해)
    const send = (method, params = {}) => new Promise((res) => {
      const myId = ++id;
      const onMsg = (ev) => { const msg = JSON.parse(ev.data); if (msg.id === myId) { ws.removeEventListener('message', onMsg); res(msg); } };
      ws.addEventListener('message', onMsg);
      ws.send(JSON.stringify({ id: myId, method, params }));
    });
    if (THROTTLE > 1) await send('Emulation.setCPUThrottlingRate', { rate: THROTTLE });
    await send('Page.navigate', { url: `file://${file}?n=${n}&auto=1${LIMIT}` });
    for (let i = 0; i < 1500; i++) {
      const r = await evaluate('window.__benchResult ?? null');
      if (r) { ws.close(); return r; }
      await sleep(200);
    }
    throw new Error('측정이 끝나지 않음');
  } finally {
    chrome.kill();
    await sleep(300);
    rmSync(profile, { recursive: true, force: true });
  }
}

const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];

console.log('| 앱 | 파일 크기 (압축) | 회원 수 | 첫 표시 | 검색→화면 평균/95%/최대 | 알림→화면 평균/95%/최대 | 검색 DOM 처리 평균/최대 | 알림 DOM 처리 평균/최대 |');
console.log('|---|---|---|---|---|---|---|---|');
for (const app of APPS) {
  const file = resolve(app, 'dist/index.html');
  const size = statSync(file).size;
  const gz = gzipSync(readFileSync(file)).length;
  for (const n of SIZES) {
    const runs = [];
    for (let i = 0; i < RUNS; i++) runs.push(await measure(file, n));
    const m = (pick) => median(runs.map(pick));
    console.log(`| ${runs[0].framework} | ${(size / 1024).toFixed(0)}KB (${(gz / 1024).toFixed(0)}KB) | ${n} | ${m((r) => r.firstPaintMs)}ms | ` +
      `${m((r) => r.searchMs.avg)} / ${m((r) => r.searchMs.p95)} / ${m((r) => r.searchMs.max)}ms | ` +
      `${m((r) => r.refreshMs.avg)} / ${m((r) => r.refreshMs.p95)} / ${m((r) => r.refreshMs.max)}ms | ` +
      `${m((r) => r.searchDomMs.avg)} / ${m((r) => r.searchDomMs.max)}ms | ` +
      `${m((r) => r.refreshDomMs.avg)} / ${m((r) => r.refreshDomMs.max)}ms |`);
  }
}
console.log(`\n(각 칸은 ${RUNS}번 실행의 중앙값, CPU 감속 ${THROTTLE}배${LIMIT ? `, 표 ${process.argv[4]}줄만 표시` : ''})`);
