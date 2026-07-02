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

var importsTemplate = mustPageTemplate("imports", "imports.html", nil)

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
	if err := importsTemplate.ExecuteTemplate(w, "imports", importsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Result:      result,
	}); err != nil {
		opts.Logger.Error("render imports failed", "error", err)
	}
}
