package launcher

import (
	"errors"
	"net"
	"reflect"
	"testing"
)

func TestNetworkResolverSortsAndDeduplicatesPrivateAddresses(t *testing.T) {
	resolver := &NetworkResolver{discover: func() ([]net.IP, error) {
		return []net.IP{
			net.ParseIP("10.0.0.8"),
			net.ParseIP("192.168.0.20"),
			net.ParseIP("172.16.0.5"),
			net.ParseIP("192.168.0.20"),
			net.ParseIP("127.0.0.1"),
			net.ParseIP("8.8.8.8"),
		}, nil
	}}

	got, err := resolver.AccessURLs("0.0.0.0", 18080)
	if err != nil {
		t.Fatalf("AccessURLs() error = %v", err)
	}
	want := []string{
		"http://192.168.0.20:18080",
		"http://10.0.0.8:18080",
		"http://172.16.0.5:18080",
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("AccessURLs() = %v, want %v", got, want)
	}
}

func TestNetworkResolverUsesPrivateExplicitHostWithoutDiscovery(t *testing.T) {
	resolver := &NetworkResolver{discover: func() ([]net.IP, error) {
		return nil, errors.New("must not be called")
	}}

	got, err := resolver.AccessURLs("192.168.1.30", 8080)
	if err != nil {
		t.Fatalf("AccessURLs() error = %v", err)
	}
	want := []string{"http://192.168.1.30:8080"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("AccessURLs() = %v, want %v", got, want)
	}
}

func TestBrowserURLUsesLoopbackForWildcardBind(t *testing.T) {
	if got := BrowserURL("0.0.0.0", 18080); got != "http://127.0.0.1:18080" {
		t.Fatalf("BrowserURL() = %q", got)
	}
	if got := BrowserURL("::1", 18080); got != "http://[::1]:18080" {
		t.Fatalf("BrowserURL(IPv6) = %q", got)
	}
}
