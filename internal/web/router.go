// Package web contains HTTP handlers, middleware, and route wiring.
package web

import (
	"html/template"
	"log/slog"
	"net/http"
)

type RouterOptions struct {
	DisplayName string
	Version     string
	Logger      *slog.Logger
}

type pageData struct {
	Title       string
	DisplayName string
	Version     string
	Heading     string
	Description string
}

var pageTemplate = template.Must(template.New("page").Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{.Title}} - {{.DisplayName}}</title>
</head>
<body>
  <main>
    <h1>{{.Heading}}</h1>
    <p>{{.Description}}</p>
    <small>{{.DisplayName}} {{.Version}}</small>
  </main>
</body>
</html>
`))

func NewRouter(opts RouterOptions) http.Handler {
	if opts.DisplayName == "" {
		opts.DisplayName = "배움마루"
	}
	if opts.Version == "" {
		opts.Version = "dev"
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", exactPath("/", renderPage(opts, pageData{
		Title:       "홈",
		Heading:     "배움마루",
		Description: "로컬 호스팅 수강신청 업무 도구가 실행 중입니다.",
	})))
	mux.HandleFunc("/admin", exactPath("/admin", renderPage(opts, pageData{
		Title:       "관리",
		Heading:     "관리 화면",
		Description: "회원, 강좌, 신청 현황, 추첨, 출력을 관리하는 화면입니다.",
	})))
	mux.HandleFunc("/reception", exactPath("/reception", renderPage(opts, pageData{
		Title:       "접수",
		Heading:     "접수 화면",
		Description: "회원 검색과 수강신청 입력을 진행하는 화면입니다.",
	})))
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok\n"))
	})

	return mux
}

func exactPath(path string, next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != path {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		next(w, r)
	}
}

func renderPage(opts RouterOptions, data pageData) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		data.DisplayName = opts.DisplayName
		data.Version = opts.Version

		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		if err := pageTemplate.Execute(w, data); err != nil {
			opts.Logger.Error("render page failed", "path", r.URL.Path, "error", err)
		}
	}
}
