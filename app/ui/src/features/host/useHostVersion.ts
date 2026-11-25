import { useQuery } from "@tanstack/react-query";
import { fetchHostVersion } from "../../api/version";

export function useHostVersion() {
  return useQuery({
    queryKey: ["hostVersion"],
    queryFn: fetchHostVersion,
  });
}
