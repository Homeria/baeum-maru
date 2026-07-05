package web

import (
	"context"
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

type authContextKey struct{}

type AuthIdentity struct {
	UserID       int64
	AccessCodeID int64
	DisplayName  string
	Role         string
}

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
			identity, err := authenticateLogin(r, opts)
			if err != nil {
				renderLogin(w, opts, next, "접속 코드가 올바르지 않거나 만료되었습니다.", http.StatusUnauthorized)
				return
			}
			setSessionCookie(w, opts, identity)
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
		identity, ok := validSessionIdentity(r, opts)
		if ok {
			ctx := context.WithValue(r.Context(), authContextKey{}, identity)
			next.ServeHTTP(w, r.WithContext(ctx))
			return
		}
		redirect := "/login?next=" + url.QueryEscape(safeRedirectPath(r.URL.RequestURI()))
		http.Redirect(w, r, redirect, http.StatusSeeOther)
	})
}

func authEnabled(opts RouterOptions) bool {
	return !opts.Auth.Disabled && opts.Auth.SessionSecret != "" && (opts.Authenticator != nil || opts.Auth.AdminPassword != "")
}

func authenticateLogin(r *http.Request, opts RouterOptions) (AuthIdentity, error) {
	code := r.FormValue("access_code")
	if code == "" {
		code = r.FormValue("password")
	}
	if opts.Authenticator != nil {
		user, err := opts.Authenticator.AuthenticateAccessCode(r.Context(), code)
		if err != nil {
			return AuthIdentity{}, err
		}
		return AuthIdentity{
			UserID:       user.UserID,
			AccessCodeID: user.AccessCodeID,
			DisplayName:  user.DisplayName,
			Role:         user.Role,
		}, nil
	}
	if !passwordMatches(code, opts.Auth.AdminPassword) {
		return AuthIdentity{}, http.ErrNoCookie
	}
	return AuthIdentity{Role: "launcher"}, nil
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

func setSessionCookie(w http.ResponseWriter, opts RouterOptions, identity AuthIdentity) {
	issuedAt := time.Now().Unix()
	value := sessionPayload(issuedAt, identity)
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

func validSessionIdentity(r *http.Request, opts RouterOptions) (AuthIdentity, bool) {
	cookie, err := r.Cookie(authCookieName)
	if err != nil {
		return AuthIdentity{}, false
	}
	payload, signature, ok := strings.Cut(cookie.Value, ".")
	if !ok || payload == "" || signature == "" {
		return AuthIdentity{}, false
	}
	if !hmac.Equal([]byte(signature), []byte(sessionSignature(opts, payload))) {
		return AuthIdentity{}, false
	}
	issuedUnix, identity, ok := parseSessionPayload(payload)
	if !ok {
		return AuthIdentity{}, false
	}
	maxAge := time.Duration(opts.Auth.SessionMaxAgeMinutes) * time.Minute
	if time.Since(time.Unix(issuedUnix, 0)) > maxAge {
		return AuthIdentity{}, false
	}
	if opts.Authenticator != nil && identity.UserID > 0 {
		validator, ok := opts.Authenticator.(interface {
			ValidateAccessSession(context.Context, int64, int64) error
		})
		if ok && validator.ValidateAccessSession(r.Context(), identity.UserID, identity.AccessCodeID) != nil {
			return AuthIdentity{}, false
		}
	}
	return identity, true
}

func sessionPayload(issuedAt int64, identity AuthIdentity) string {
	return strings.Join([]string{
		strconv.FormatInt(issuedAt, 10),
		strconv.FormatInt(identity.UserID, 10),
		strconv.FormatInt(identity.AccessCodeID, 10),
		identity.Role,
	}, "|")
}

func parseSessionPayload(payload string) (int64, AuthIdentity, bool) {
	parts := strings.Split(payload, "|")
	if len(parts) == 1 {
		issuedUnix, err := strconv.ParseInt(parts[0], 10, 64)
		return issuedUnix, AuthIdentity{}, err == nil
	}
	if len(parts) == 3 {
		issuedUnix, err := strconv.ParseInt(parts[0], 10, 64)
		if err != nil {
			return 0, AuthIdentity{}, false
		}
		userID, err := strconv.ParseInt(parts[1], 10, 64)
		if err != nil {
			return 0, AuthIdentity{}, false
		}
		return issuedUnix, AuthIdentity{UserID: userID, Role: parts[2]}, true
	}
	if len(parts) != 4 {
		return 0, AuthIdentity{}, false
	}
	issuedUnix, err := strconv.ParseInt(parts[0], 10, 64)
	if err != nil {
		return 0, AuthIdentity{}, false
	}
	userID, err := strconv.ParseInt(parts[1], 10, 64)
	if err != nil {
		return 0, AuthIdentity{}, false
	}
	accessCodeID, err := strconv.ParseInt(parts[2], 10, 64)
	if err != nil {
		return 0, AuthIdentity{}, false
	}
	return issuedUnix, AuthIdentity{UserID: userID, AccessCodeID: accessCodeID, Role: parts[3]}, true
}

func currentAuthIdentity(r *http.Request) (AuthIdentity, bool) {
	identity, ok := r.Context().Value(authContextKey{}).(AuthIdentity)
	return identity, ok
}

func sessionSignature(opts RouterOptions, value string) string {
	mac := hmac.New(sha256.New, []byte(opts.Auth.SessionSecret))
	_, _ = mac.Write([]byte(value))
	return hex.EncodeToString(mac.Sum(nil))
}
