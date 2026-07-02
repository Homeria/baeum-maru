package web

import (
	"crypto/hmac"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"
)

const authCookieName = "baeum_maru_session"

type loginPageData struct {
	DisplayName string
	Version     string
	Next        string
	Error       string
}

var loginTemplate = mustPageTemplate("login", "login.html", nil)

func loginHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/login" {
			http.NotFound(w, r)
			return
		}
		if !authEnabled(opts) {
			http.Redirect(w, r, "/admin", http.StatusSeeOther)
			return
		}
		switch r.Method {
		case http.MethodGet:
			renderLogin(w, opts, loginNext(r), "", http.StatusOK)
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			next := safeRedirectPath(r.FormValue("next"))
			if !passwordMatches(r.FormValue("password"), opts.Auth.AdminPassword) {
				renderLogin(w, opts, next, "비밀번호가 올바르지 않습니다.", http.StatusUnauthorized)
				return
			}
			setSessionCookie(w, opts)
			http.Redirect(w, r, next, http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func logoutHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/logout" {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodPost {
			w.Header().Set("Allow", http.MethodPost)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		clearSessionCookie(w)
		http.Redirect(w, r, "/login", http.StatusSeeOther)
	}
}

func renderLogin(w http.ResponseWriter, opts RouterOptions, next string, errorMessage string, status int) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.WriteHeader(status)
	if err := loginTemplate.ExecuteTemplate(w, "login", loginPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Next:        next,
		Error:       errorMessage,
	}); err != nil {
		opts.Logger.Error("render login failed", "error", err)
	}
}

func requireAuth(opts RouterOptions, next http.Handler) http.Handler {
	if !authEnabled(opts) {
		return next
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if hasValidSession(r, opts) {
			next.ServeHTTP(w, r)
			return
		}
		redirect := "/login?next=" + url.QueryEscape(safeRedirectPath(r.URL.RequestURI()))
		http.Redirect(w, r, redirect, http.StatusSeeOther)
	})
}

func authEnabled(opts RouterOptions) bool {
	return !opts.Auth.Disabled && opts.Auth.AdminPassword != "" && opts.Auth.SessionSecret != ""
}

func passwordMatches(got string, want string) bool {
	return subtle.ConstantTimeCompare([]byte(got), []byte(want)) == 1
}

func loginNext(r *http.Request) string {
	return safeRedirectPath(r.URL.Query().Get("next"))
}

func safeRedirectPath(path string) string {
	if path == "" || !strings.HasPrefix(path, "/") || strings.HasPrefix(path, "//") {
		return "/admin"
	}
	if strings.HasPrefix(path, "/login") || strings.HasPrefix(path, "/logout") {
		return "/admin"
	}
	return path
}

func setSessionCookie(w http.ResponseWriter, opts RouterOptions) {
	issuedAt := time.Now().Unix()
	value := strconv.FormatInt(issuedAt, 10)
	http.SetCookie(w, &http.Cookie{
		Name:     authCookieName,
		Value:    value + "." + sessionSignature(opts, value),
		Path:     "/",
		MaxAge:   opts.Auth.SessionMaxAgeMinutes * 60,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	})
}

func clearSessionCookie(w http.ResponseWriter) {
	http.SetCookie(w, &http.Cookie{
		Name:     authCookieName,
		Value:    "",
		Path:     "/",
		MaxAge:   -1,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	})
}

func hasValidSession(r *http.Request, opts RouterOptions) bool {
	cookie, err := r.Cookie(authCookieName)
	if err != nil {
		return false
	}
	issuedAt, signature, ok := strings.Cut(cookie.Value, ".")
	if !ok || issuedAt == "" || signature == "" {
		return false
	}
	if !hmac.Equal([]byte(signature), []byte(sessionSignature(opts, issuedAt))) {
		return false
	}
	issuedUnix, err := strconv.ParseInt(issuedAt, 10, 64)
	if err != nil {
		return false
	}
	maxAge := time.Duration(opts.Auth.SessionMaxAgeMinutes) * time.Minute
	return time.Since(time.Unix(issuedUnix, 0)) <= maxAge
}

func sessionSignature(opts RouterOptions, value string) string {
	mac := hmac.New(sha256.New, []byte(opts.Auth.SessionSecret))
	_, _ = mac.Write([]byte(value))
	return hex.EncodeToString(mac.Sum(nil))
}
