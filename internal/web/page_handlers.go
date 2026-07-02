package web

import "net/http"

type pageData struct {
	Title       string
	DisplayName string
	Version     string
	Permissions permissionSet
	Heading     string
	Description string
}

var pageTemplate = mustPageTemplate("page", "page.html", nil)

func renderPage(opts RouterOptions, data pageData) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		data.DisplayName = opts.DisplayName
		data.Version = opts.Version
		data.Permissions = pagePermissions(r)

		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		if err := pageTemplate.ExecuteTemplate(w, "page", data); err != nil {
			opts.Logger.Error("render page failed", "path", r.URL.Path, "error", err)
		}
	}
}
