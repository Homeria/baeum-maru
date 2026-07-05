//go:build fyne

package main

import (
	"fmt"
	"net"
	"sort"
	"strings"
)

func networkAccessURLs(host string, port int) []string {
	if strings.TrimSpace(host) != "" && host != "0.0.0.0" && host != "::" {
		ip := net.ParseIP(host)
		if ip == nil || ip.IsLoopback() || ip.IsUnspecified() || !isPrivateIPv4(ip) {
			return nil
		}
		return []string{fmt.Sprintf("http://%s:%d", host, port)}
	}

	interfaces, err := net.Interfaces()
	if err != nil {
		return nil
	}

	seen := map[string]struct{}{}
	for _, iface := range interfaces {
		if iface.Flags&net.FlagUp == 0 || iface.Flags&net.FlagLoopback != 0 {
			continue
		}
		addrs, err := iface.Addrs()
		if err != nil {
			continue
		}
		for _, addr := range addrs {
			ip := addressIP(addr)
			if ip == nil || ip.IsLoopback() || !ip.IsGlobalUnicast() || !isPrivateIPv4(ip) {
				continue
			}
			ipv4 := ip.To4()
			if ipv4 == nil {
				continue
			}
			seen[ipv4.String()] = struct{}{}
		}
	}

	hosts := make([]string, 0, len(seen))
	for host := range seen {
		hosts = append(hosts, host)
	}
	sort.Slice(hosts, func(i int, j int) bool {
		leftRank := privateIPv4Rank(net.ParseIP(hosts[i]))
		rightRank := privateIPv4Rank(net.ParseIP(hosts[j]))
		if leftRank != rightRank {
			return leftRank < rightRank
		}
		return hosts[i] < hosts[j]
	})

	urls := make([]string, 0, len(hosts))
	for _, host := range hosts {
		urls = append(urls, fmt.Sprintf("http://%s:%d", host, port))
	}
	return urls
}

func isPrivateIPv4(ip net.IP) bool {
	return privateIPv4Rank(ip) < 3
}

func privateIPv4Rank(ip net.IP) int {
	ipv4 := ip.To4()
	if ipv4 == nil {
		return 3
	}
	switch {
	case ipv4[0] == 192 && ipv4[1] == 168:
		return 0
	case ipv4[0] == 10:
		return 1
	case ipv4[0] == 172 && ipv4[1] >= 16 && ipv4[1] <= 31:
		return 2
	default:
		return 3
	}
}

func addressIP(addr net.Addr) net.IP {
	switch value := addr.(type) {
	case *net.IPNet:
		return value.IP
	case *net.IPAddr:
		return value.IP
	default:
		return nil
	}
}

func accessURLText(urls []string) string {
	if len(urls) == 0 {
		return "감지된 내부망 주소가 없습니다."
	}
	return strings.Join(urls, "\n")
}
