package web

import (
	"html/template"
	"io/fs"
	"net/http"

	webassets "github.com/Homeria/baeum-maru/web"
)

func mustPageTemplate(name string, fileName string, extra template.FuncMap) *template.Template {
	patterns := []string{
		"templates/partials/*.html",
		"templates/pages/" + fileName,
	}
	return template.Must(template.New(name).Funcs(uiTemplateFuncs(extra)).ParseFS(webassets.Files, patterns...))
}

func staticFileHandler() http.Handler {
	staticFiles, err := fs.Sub(webassets.Files, "static")
	if err != nil {
		panic(err)
	}
	return http.FileServer(http.FS(staticFiles))
}
