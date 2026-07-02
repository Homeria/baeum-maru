package web

import (
	"fmt"
	"html/template"
	"net/http"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type coursesPageData struct {
	DisplayName string
	Version     string
	Permissions permissionSet
	Error       string
	Offerings   []domain.CourseOffering
}

var coursesTemplate = mustPageTemplate("courses", "courses.html", template.FuncMap{"weekdayLabel": weekdayLabel})

func coursesHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Courses == nil {
			http.Error(w, "course service is not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderCourses(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			input, err := courseInputFromRequest(r)
			if err != nil {
				renderCourses(w, r, opts, err.Error())
				return
			}
			if r.FormValue("action") == "update" {
				id, err := strconv.ParseInt(r.FormValue("offering_id"), 10, 64)
				if err != nil {
					renderCourses(w, r, opts, "강좌 선택이 올바르지 않습니다.")
					return
				}
				updated, err := opts.Courses.UpdateOffering(r.Context(), id, input)
				if err != nil {
					renderCourses(w, r, opts, err.Error())
					return
				}
				recordAudit(r, opts, "course.update", "course_offering", updated.ID, "강좌 개설 수정 #"+strconv.FormatInt(updated.ID, 10))
				http.Redirect(w, r, "/admin/courses", http.StatusSeeOther)
				return
			}
			created, err := opts.Courses.CreateOffering(r.Context(), input)
			if err != nil {
				renderCourses(w, r, opts, err.Error())
				return
			}
			recordAudit(r, opts, "course.create", "course_offering", created.ID, "강좌 개설 등록 #"+strconv.FormatInt(created.ID, 10))
			http.Redirect(w, r, "/admin/courses", http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func courseInputFromRequest(r *http.Request) (service.CourseOfferingInput, error) {
	capacity, err := strconv.Atoi(r.FormValue("capacity"))
	if err != nil {
		return service.CourseOfferingInput{}, fmt.Errorf("정원은 숫자로 입력해야 합니다.")
	}
	weekday, err := strconv.Atoi(r.FormValue("weekday"))
	if err != nil {
		return service.CourseOfferingInput{}, fmt.Errorf("요일 값이 올바르지 않습니다.")
	}
	return service.CourseOfferingInput{
		TermName:       r.FormValue("term_name"),
		CategoryName:   r.FormValue("category_name"),
		CourseTitle:    r.FormValue("course_title"),
		InstructorName: r.FormValue("instructor_name"),
		ClassroomName:  r.FormValue("classroom_name"),
		Capacity:       capacity,
		Weekday:        weekday,
		StartTime:      r.FormValue("start_time"),
		EndTime:        r.FormValue("end_time"),
		Note:           r.FormValue("note"),
	}, nil
}

func renderCourses(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	offerings, err := opts.Courses.ListOfferings(r.Context(), 100)
	if err != nil {
		message = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := coursesTemplate.ExecuteTemplate(w, "courses", coursesPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Permissions: pagePermissions(r),
		Error:       message,
		Offerings:   offerings,
	}); err != nil {
		opts.Logger.Error("render courses failed", "error", err)
	}
}
