import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { searchMembers, type MemberSearch } from "./api";

/** 회원 찾기 결과. 조건이 바뀌는 동안에는 이전 결과를 계속 보여 준다. */
export function useMemberSearch(search: MemberSearch) {
  return useQuery({
    queryKey: ["members", "search", search],
    queryFn: () => searchMembers(search),
    placeholderData: keepPreviousData,
  });
}
