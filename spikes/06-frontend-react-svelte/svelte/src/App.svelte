<!-- 버린 코드: Svelte 버전. 흔히 짜는 방식 그대로. -->
<script>
  import { onMount, flushSync } from 'svelte';
  import { makeMembers, filterMembers } from '../../shared/data.js';
  import { registerBench, runBench, memberCount, rowLimit } from '../../shared/bench.js';

  let data = $state.raw({ rev: 0, list: makeMembers(memberCount()) });
  let query = $state('');
  let matched = $derived(filterMembers(data.list, query));
  let rows = $derived(matched.slice(0, rowLimit()));

  onMount(() => {
    registerBench({
      name: 'Svelte',
      setQuery: (q) => (query = q),
      replaceMembers: (d) => (data = d),
      flush: flushSync,
    });
  });
</script>

<h1>실험 06 · Svelte · 회원 {data.list.length}명</h1>
<div class="bar">
  <input bind:value={query} placeholder="이름, 회원번호, 연락처 검색" />
  <span>{matched.length}명 중 {rows.length}명 표시</span>
  <button onclick={() => runBench()}>자동 측정</button>
</div>
<pre id="bench-result">준비 중…</pre>
<table data-q={query} data-rev={data.rev}>
  <thead>
    <tr><th>회원번호</th><th>이름</th><th>연락처</th><th>출생</th><th>성별</th><th>신청</th></tr>
  </thead>
  <tbody>
    {#each rows as m (m.id)}
      <tr class={m.changed ? 'changed' : ''}>
        <td>{m.memberNo}</td><td>{m.name}</td><td>{m.phone}</td><td>{m.birthYear}</td><td>{m.gender}</td><td>{m.courses}</td>
      </tr>
    {/each}
  </tbody>
</table>
