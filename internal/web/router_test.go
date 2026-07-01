package web

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestRouterServesBasicPages(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
	})

	tests := []struct {
		path string
		want string
	}{
		{path: "/", want: "로컬 호스팅 수강신청 업무 도구"},
		{path: "/admin", want: "관리 화면"},
		{path: "/reception", want: "접수 화면"},
	}

	for _, tt := range tests {
		t.Run(tt.path, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tt.path, nil)
			rec := httptest.NewRecorder()

			router.ServeHTTP(rec, req)

			if rec.Code != http.StatusOK {
				t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
			}
			if !strings.Contains(rec.Body.String(), tt.want) {
				t.Fatalf("body = %q, want substring %q", rec.Body.String(), tt.want)
			}
		})
	}
}

func TestRouterServesHealthCheck(t *testing.T) {
	router := NewRouter(RouterOptions{})
	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if rec.Body.String() != "ok\n" {
		t.Fatalf("body = %q, want ok", rec.Body.String())
	}
}

func TestRouterRejectsUnsupportedMethod(t *testing.T) {
	router := NewRouter(RouterOptions{})
	req := httptest.NewRequest(http.MethodPost, "/admin", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusMethodNotAllowed {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusMethodNotAllowed)
	}
}
