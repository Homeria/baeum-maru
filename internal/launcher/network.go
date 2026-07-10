package launcher

import (
	"fmt"
	"net"
	"sort"
	"strconv"
	"strings"
)

type NetworkResolver struct {
	discover func() ([]net.IP, error)
}

func NewNetworkResolver() *NetworkResolver {
	return &NetworkResolver{discover: discoverPrivateIPv4}
}

func BrowserURL(host string, port int) string {
	host = strings.TrimSpace(host)
	if host == "" || host == "0.0.0.0" || host == "::" {
		host = "127.0.0.1"
	}
	return httpURL(host, port)
}

func (r *NetworkResolver) AccessURLs(host string, port int) ([]string, error) {
	host = strings.TrimSpace(host)
	if host != "" && host != "0.0.0.0" && host != "::" {
		ip := net.ParseIP(host)
		if ip == nil || ip.IsLoopback() || ip.IsUnspecified() || !isPrivateIPv4(ip) {
			return nil, nil
		}
		return []string{httpURL(host, port)}, nil
	}

	if r == nil || r.discover == nil {
		return nil, fmt.Errorf("network address discovery is not configured")
	}
	ips, err := r.discover()
	if err != nil {
		return nil, fmt.Errorf("discover network addresses: %w", err)
	}

	seen := make(map[string]struct{}, len(ips))
	for _, ip := range ips {
		if ip == nil || ip.IsLoopback() || !ip.IsGlobalUnicast() || !isPrivateIPv4(ip) {
			continue
		}
		if ipv4 := ip.To4(); ipv4 != nil {
			seen[ipv4.String()] = struct{}{}
		}
	}

	hosts := make([]string, 0, len(seen))
	for address := range seen {
		hosts = append(hosts, address)
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
	for _, address := range hosts {
		urls = append(urls, httpURL(address, port))
	}
	return urls, nil
}

func AccessURLText(urls []string) string {
	if len(urls) == 0 {
		return "감지된 내부망 주소가 없습니다."
	}
	return strings.Join(urls, "\n")
}

func discoverPrivateIPv4() ([]net.IP, error) {
	interfaces, err := net.Interfaces()
	if err != nil {
		return nil, err
	}

	var ips []net.IP
	for _, iface := range interfaces {
		if iface.Flags&net.FlagUp == 0 || iface.Flags&net.FlagLoopback != 0 {
			continue
		}
		addresses, err := iface.Addrs()
		if err != nil {
			continue
		}
		for _, address := range addresses {
			if ip := addressIP(address); ip != nil {
				ips = append(ips, ip)
			}
		}
	}
	return ips, nil
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

func addressIP(address net.Addr) net.IP {
	switch value := address.(type) {
	case *net.IPNet:
		return value.IP
	case *net.IPAddr:
		return value.IP
	default:
		return nil
	}
}

func httpURL(host string, port int) string {
	return "http://" + net.JoinHostPort(host, strconv.Itoa(port))
}
