package web

import (
	"fmt"
	"net/http"

	"github.com/Homeria/baeum-maru/internal/service"
)

const importUploadLimitBytes = 10 << 20

type importsPageData struct {
	DisplayName string
	Version     string
	Message     string
	Error       string
	Result      *service.ImportResult
}

var importsTemplate = templateMust("imports", `<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>엑셀 가져오기 - {{.DisplayName}}</title>
  <style>{{appStyles}}</style>
</head>
<body>
  <header class="topbar">
    <a class="brand" href="/admin">{{.DisplayName}}</a>
    <nav class="topnav">
      <a href="/admin/members">회원 관리</a>
      <a href="/admin/courses">강좌 관리</a>
      <a href="/admin/registrations">신청 현황</a>
      <a href="/admin/lottery">추첨</a>
      <a href="/admin/imports">엑셀 가져오기</a>
      <a href="/admin/exports">엑셀 내보내기</a>
      <a href="/admin/backups">백업</a>
      <a href="/admin/attendance">출석</a>
      <a href="/admin/settings">설정</a>
      <a href="/admin/audit-logs">감사 로그</a>
      <a href="/reception">접수 화면</a>
    </nav>
  </header>
  <main class="page">
    <section class="page-header">
      <div>
        <h1>엑셀 가져오기</h1>
      </div>
    </section>
    {{if .Message}}<p class="alert success" role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}

    <section class="grid-2">
      <form class="panel" method="post" action="/admin/imports/members" enctype="multipart/form-data">
        <h2>회원 명단</h2>
        <p class="subtle"><a href="/admin/imports/members/template">회원 표준 양식 다운로드</a></p>
        <label>엑셀 파일 <input name="file" type="file" accept=".xlsx" required></label>
        <button type="submit">회원 가져오기</button>
      </form>

      <form class="panel" method="post" action="/admin/imports/courses" enctype="multipart/form-data">
        <h2>강좌 목록</h2>
        <p class="subtle"><a href="/admin/imports/courses/template">강좌 표준 양식 다운로드</a></p>
        <label>엑셀 파일 <input name="file" type="file" accept=".xlsx" required></label>
        <button type="submit">강좌 가져오기</button>
      </form>
    </section>

    {{with .Result}}
      <section class="panel">
        <h2>{{.Kind}} 가져오기 결과</h2>
        <p><span class="badge confirmed">성공 {{.CreatedCount}}건</span> <span class="badge {{if .Errors}}failed{{else}}completed{{end}}">오류 {{len .Errors}}건</span></p>
        {{if .Errors}}
          <div class="table-wrap">
            <table>
              <thead><tr><th>행</th><th>오류</th></tr></thead>
              <tbody>
                {{range .Errors}}
                  <tr><td>{{.Row}}</td><td>{{.Message}}</td></tr>
                {{end}}
              </tbody>
            </table>
          </div>
        {{end}}
      </section>
    {{end}}

    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`)

func importsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/imports" {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		renderImports(w, opts, "", "", nil)
	}
}

func importMembersHandler(opts RouterOptions) http.HandlerFunc {
	return importUploadHandler(opts, func(imports ImportService, r *http.Request) (service.ImportResult, error) {
		file, _, err := r.FormFile("file")
		if err != nil {
			return service.ImportResult{}, fmt.Errorf("read uploaded member workbook: %w", err)
		}
		defer file.Close()
		return imports.ImportMembers(r.Context(), file)
	})
}

func importCoursesHandler(opts RouterOptions) http.HandlerFunc {
	return importUploadHandler(opts, func(imports ImportService, r *http.Request) (service.ImportResult, error) {
		file, _, err := r.FormFile("file")
		if err != nil {
			return service.ImportResult{}, fmt.Errorf("read uploaded course workbook: %w", err)
		}
		defer file.Close()
		return imports.ImportCourseOfferings(r.Context(), file)
	})
}

func importUploadHandler(opts RouterOptions, importWorkbook func(ImportService, *http.Request) (service.ImportResult, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Imports == nil {
			http.Error(w, "import service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodPost {
			w.Header().Set("Allow", http.MethodPost)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, importUploadLimitBytes)
		if err := r.ParseMultipartForm(importUploadLimitBytes); err != nil {
			renderImports(w, opts, "", "업로드 파일을 읽을 수 없습니다.", nil)
			return
		}
		result, err := importWorkbook(opts.Imports, r)
		if err != nil {
			renderImports(w, opts, "", err.Error(), nil)
			return
		}
		message := fmt.Sprintf("%s 가져오기를 처리했습니다.", result.Kind)
		recordAudit(r, opts, "excel.import", "import", 0, fmt.Sprintf("%s 가져오기 성공 %d건 오류 %d건", result.Kind, result.CreatedCount, len(result.Errors)))
		renderImports(w, opts, message, "", &result)
	}
}

func memberImportTemplateHandler(opts RouterOptions) http.HandlerFunc {
	return importTemplateHandler(opts, func(imports ImportService) (service.ImportTemplate, error) {
		return imports.MemberTemplate()
	})
}

func courseImportTemplateHandler(opts RouterOptions) http.HandlerFunc {
	return importTemplateHandler(opts, func(imports ImportService) (service.ImportTemplate, error) {
		return imports.CourseOfferingTemplate()
	})
}

func importTemplateHandler(opts RouterOptions, create func(ImportService) (service.ImportTemplate, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Imports == nil {
			http.Error(w, "import service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		template, err := create(opts.Imports)
		if err != nil {
			opts.Logger.Error("create import template failed", "path", r.URL.Path, "error", err)
			http.Error(w, "failed to create import template", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%q", template.FileName))
		w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
		_, _ = w.Write(template.Content)
	}
}

func renderImports(w http.ResponseWriter, opts RouterOptions, message string, errorMessage string, result *service.ImportResult) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := importsTemplate.Execute(w, importsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Result:      result,
	}); err != nil {
		opts.Logger.Error("render imports failed", "error", err)
	}
}
