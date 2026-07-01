package web

import (
	"context"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
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

func TestRouterServesMemberManagement(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Members: &fakeMemberService{
			members: []domain.Member{{ID: 1, Name: "김배움"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/members?q=김", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "김배움") {
		t.Fatalf("body = %q, want member name", rec.Body.String())
	}
}

func TestRouterCreatesMember(t *testing.T) {
	members := &fakeMemberService{}
	router := NewRouter(RouterOptions{
		Members: members,
	})
	form := url.Values{"name": {"김배움"}}
	req := httptest.NewRequest(http.MethodPost, "/admin/members", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if members.created.Name != "김배움" {
		t.Fatalf("created.Name = %q, want 김배움", members.created.Name)
	}
}

func TestRouterServesCourseManagement(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Courses: &fakeCourseService{
			offerings: []domain.CourseOffering{{ID: 1, CourseTitle: "요가 기초", Weekday: 1, StartTime: "09:00", EndTime: "10:00"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/courses", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "요가 기초") {
		t.Fatalf("body = %q, want course title", rec.Body.String())
	}
}

func TestRouterCreatesCourseOffering(t *testing.T) {
	courses := &fakeCourseService{}
	router := NewRouter(RouterOptions{
		Courses: courses,
	})
	form := url.Values{
		"course_title": {"요가 기초"},
		"capacity":     {"20"},
		"weekday":      {"1"},
		"start_time":   {"09:00"},
		"end_time":     {"10:00"},
	}
	req := httptest.NewRequest(http.MethodPost, "/admin/courses", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if courses.created.CourseTitle != "요가 기초" {
		t.Fatalf("created.CourseTitle = %q, want 요가 기초", courses.created.CourseTitle)
	}
}

type fakeMemberService struct {
	created service.MemberInput
	members []domain.Member
}

func (f *fakeMemberService) Create(_ context.Context, input service.MemberInput) (domain.Member, error) {
	f.created = input
	return domain.Member{ID: 1, Name: input.Name}, nil
}

func (f *fakeMemberService) Search(_ context.Context, _ string, _ int) ([]domain.Member, error) {
	return f.members, nil
}

type fakeCourseService struct {
	created   service.CourseOfferingInput
	offerings []domain.CourseOffering
}

func (f *fakeCourseService) CreateOffering(_ context.Context, input service.CourseOfferingInput) (domain.CourseOffering, error) {
	f.created = input
	return domain.CourseOffering{ID: 1, CourseTitle: input.CourseTitle}, nil
}

func (f *fakeCourseService) ListOfferings(_ context.Context, _ int) ([]domain.CourseOffering, error) {
	return f.offerings, nil
}
