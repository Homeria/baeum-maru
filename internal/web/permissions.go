package web

import "net/http"

const (
	roleLauncher       = "launcher"
	roleStaff          = "staff"
	roleTemporaryStaff = "temporary_staff"
	roleViewer         = "viewer"
)

type permissionSet map[string]bool

func requirePermission(opts RouterOptions, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !authEnabled(opts) {
			next.ServeHTTP(w, r)
			return
		}
		identity, ok := currentAuthIdentity(r)
		if !ok {
			http.Error(w, "forbidden", http.StatusForbidden)
			return
		}
		if !isAllowed(identity.Role, r.Method, r.URL.Path) {
			http.Error(w, "forbidden", http.StatusForbidden)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func pagePermissions(r *http.Request) permissionSet {
	identity, ok := currentAuthIdentity(r)
	if !ok {
		return allPermissions()
	}
	return permissionsForRole(identity.Role)
}

func allPermissions() permissionSet {
	return permissionSet{
		"members":              true,
		"write_members":        true,
		"courses":              true,
		"write_courses":        true,
		"locations":            true,
		"write_locations":      true,
		"reception":            true,
		"write_reception":      true,
		"registrations":        true,
		"manage_registrations": true,
		"lottery":              true,
		"imports":              true,
		"exports":              true,
		"backups":              true,
		"attendance":           true,
		"write_attendance":     true,
		"settings":             true,
		"audit_logs":           true,
	}
}

func permissionsForRole(role string) permissionSet {
	if role == roleLauncher {
		return allPermissions()
	}
	permissions := permissionSet{
		"members":       true,
		"courses":       true,
		"locations":     true,
		"registrations": true,
		"attendance":    true,
	}
	if role == roleStaff {
		permissions["write_members"] = true
		permissions["write_courses"] = true
		permissions["write_locations"] = true
		permissions["reception"] = true
		permissions["write_reception"] = true
		permissions["manage_registrations"] = true
		permissions["lottery"] = true
		permissions["imports"] = true
		permissions["exports"] = true
		permissions["write_attendance"] = true
	}
	if role == roleTemporaryStaff {
		permissions["reception"] = true
		permissions["write_reception"] = true
		permissions["write_attendance"] = true
	}
	if role == roleViewer {
		permissions["attendance"] = true
	}
	return permissions
}

func isAllowed(role string, method string, path string) bool {
	if role == roleLauncher {
		return true
	}
	if method == http.MethodPost && role == roleViewer {
		return false
	}
	switch path {
	case "/", "/admin":
		return method == http.MethodGet
	case "/admin/settings", "/admin/backups", "/admin/backups/create", "/admin/backups/download", "/admin/backups/restore", "/admin/audit-logs":
		return false
	case "/admin/imports", "/admin/imports/members", "/admin/imports/courses", "/admin/imports/members/template", "/admin/imports/courses/template":
		return role == roleStaff
	case "/admin/exports", "/admin/exports/members", "/admin/exports/courses", "/admin/exports/registrations", "/admin/exports/lottery-results", "/admin/exports/attendance-session", "/admin/exports/attendance-offering":
		return role == roleStaff
	case "/admin/lottery", "/admin/lottery/run":
		return role == roleStaff
	case "/reception", "/reception/cancel":
		return role == roleStaff || role == roleTemporaryStaff
	case "/admin/members", "/admin/courses", "/admin/locations", "/admin/registrations":
		return method == http.MethodGet || role == roleStaff
	case "/admin/registrations/status":
		return role == roleStaff
	case "/admin/attendance", "/admin/attendance/session", "/admin/attendance/record":
		return role == roleStaff || role == roleTemporaryStaff || (role == roleViewer && method == http.MethodGet)
	default:
		return false
	}
}
