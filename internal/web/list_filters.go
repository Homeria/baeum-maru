package web

import (
	"sort"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
)

func filterMembers(members []domain.Member, gender string) []domain.Member {
	if gender == "" {
		return members
	}
	filtered := make([]domain.Member, 0, len(members))
	for _, member := range members {
		if member.GenderCode == gender {
			filtered = append(filtered, member)
		}
	}
	return filtered
}

func sortMembers(members []domain.Member, sortKey string) {
	sort.SliceStable(members, func(i, j int) bool {
		left := members[i]
		right := members[j]
		switch sortKey {
		case "id_desc":
			return left.ID > right.ID
		case "member_no":
			if left.MemberNo == right.MemberNo {
				return left.ID < right.ID
			}
			return left.MemberNo < right.MemberNo
		case "created_desc":
			if left.CreatedAt == right.CreatedAt {
				return left.ID > right.ID
			}
			return left.CreatedAt > right.CreatedAt
		default:
			if left.Name == right.Name {
				return left.ID < right.ID
			}
			return left.Name < right.Name
		}
	})
}

func filterCourseOfferings(offerings []domain.CourseOffering, query string, term string, category string, status string) []domain.CourseOffering {
	query = strings.TrimSpace(query)
	term = strings.TrimSpace(term)
	category = strings.TrimSpace(category)
	status = strings.TrimSpace(status)
	if query == "" && term == "" && category == "" && status == "" {
		return offerings
	}

	filtered := make([]domain.CourseOffering, 0, len(offerings))
	for _, offering := range offerings {
		if query != "" && !containsAnyText(query, offering.CourseTitle, offering.InstructorName, offering.ClassroomName) {
			continue
		}
		if term != "" && !containsText(offering.TermName, term) {
			continue
		}
		if category != "" && !containsText(offering.CategoryName, category) {
			continue
		}
		if status != "" && offering.Status != status {
			continue
		}
		filtered = append(filtered, offering)
	}
	return filtered
}

func sortCourseOfferings(offerings []domain.CourseOffering, sortKey string) {
	sort.SliceStable(offerings, func(i, j int) bool {
		left := offerings[i]
		right := offerings[j]
		switch sortKey {
		case "id_desc":
			return left.ID > right.ID
		case "capacity_desc":
			if left.Capacity == right.Capacity {
				return left.ID < right.ID
			}
			return left.Capacity > right.Capacity
		case "registrations_desc":
			if left.RegistrationCount == right.RegistrationCount {
				return left.ID < right.ID
			}
			return left.RegistrationCount > right.RegistrationCount
		case "time":
			if left.Weekday == right.Weekday {
				if left.StartTime == right.StartTime {
					return left.ID < right.ID
				}
				return left.StartTime < right.StartTime
			}
			return left.Weekday < right.Weekday
		default:
			if left.TermName == right.TermName {
				if left.CourseTitle == right.CourseTitle {
					return left.ID < right.ID
				}
				return left.CourseTitle < right.CourseTitle
			}
			return left.TermName < right.TermName
		}
	})
}

func filterRegistrations(registrations []domain.Registration, query string, status string) []domain.Registration {
	query = strings.TrimSpace(query)
	status = strings.TrimSpace(status)
	if query == "" && status == "" {
		return registrations
	}

	filtered := make([]domain.Registration, 0, len(registrations))
	for _, registration := range registrations {
		if query != "" && !containsAnyText(query, registration.MemberName, registration.MemberNo, registration.CourseTitle, registration.TermName) {
			continue
		}
		if status != "" && registration.Status != status {
			continue
		}
		filtered = append(filtered, registration)
	}
	return filtered
}

func sortRegistrations(registrations []domain.Registration, sortKey string) {
	sort.SliceStable(registrations, func(i, j int) bool {
		left := registrations[i]
		right := registrations[j]
		switch sortKey {
		case "id_asc":
			return left.ID < right.ID
		case "member":
			if left.MemberName == right.MemberName {
				return left.ID > right.ID
			}
			return left.MemberName < right.MemberName
		case "course":
			if left.CourseTitle == right.CourseTitle {
				return left.ID > right.ID
			}
			return left.CourseTitle < right.CourseTitle
		default:
			if left.CreatedAt == right.CreatedAt {
				return left.ID > right.ID
			}
			return left.CreatedAt > right.CreatedAt
		}
	})
}

func containsAnyText(query string, values ...string) bool {
	for _, value := range values {
		if containsText(value, query) {
			return true
		}
	}
	return false
}

func containsText(value string, query string) bool {
	return strings.Contains(strings.ToLower(value), strings.ToLower(query))
}
