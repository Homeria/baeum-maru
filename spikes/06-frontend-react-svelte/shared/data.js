// 버린 코드: 두 앱이 똑같이 쓰는 가짜 회원 데이터. 실제 사람과 무관하게 지어낸 값이다.
const SURNAMES = ['김', '이', '박', '최', '정', '강', '조', '윤', '장', '임', '한', '오', '서', '신', '권'];
const GIVEN = ['가람', '나래', '다솜', '라온', '마루', '바다', '사랑', '아름', '자람', '하늘', '보람', '새봄', '여울', '한결', '누리'];

/** 항상 같은 결과가 나오는 가짜 회원 n명 */
export function makeMembers(n) {
  const list = [];
  for (let i = 0; i < n; i++) {
    const no = String(10001 + i).padStart(5, '0');
    list.push({
      id: i,
      memberNo: no,
      name: SURNAMES[(i * 7) % SURNAMES.length] + GIVEN[(i * 11 + Math.floor(i / 15)) % GIVEN.length],
      phone: `010-${String(1000 + ((i * 37) % 9000)).padStart(4, '0')}-${String(1000 + ((i * 53) % 9000)).padStart(4, '0')}`,
      birthYear: 1940 + ((i * 13) % 25),
      gender: i % 3 === 0 ? '남' : '여',
      courses: (i * 5) % 5,
      changed: false,
    });
  }
  return list;
}

/** 이름, 회원번호, 연락처에 검색어가 들어간 회원 */
export function filterMembers(list, query) {
  const q = query.trim();
  if (q === '') return list;
  return list.filter((m) => m.name.includes(q) || m.memberNo.includes(q) || m.phone.includes(q));
}

/** k번째 갱신: 회원 한 명의 신청 수가 바뀐 것처럼 만든다 */
export function changeOne(list, k) {
  const target = (k * 97) % list.length;
  return list.map((m) => (m.id === target ? { ...m, courses: (m.courses + 1) % 5, changed: true } : { ...m, changed: false }));
}
