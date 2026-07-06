package service

import (
	"context"
	"errors"
	"fmt"
	"io"
	"strconv"
	"strings"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/xuri/excelize/v2"
)

var memberImportHeaders = []string{"member_no", "name", "gender", "birth_date", "phone", "note"}
var courseImportHeaders = []string{"term", "category", "course", "instructor", "location", "capacity", "weekday", "start_time", "end_time", "note"}

type ImportMemberTarget interface {
	Create(context.Context, MemberInput) (domain.Member, error)
}

type ImportCourseTarget interface {
	CreateOffering(context.Context, CourseOfferingInput) (domain.CourseOffering, error)
}

type ImportService struct {
	members ImportMemberTarget
	courses ImportCourseTarget
}

type ImportResult struct {
	Kind         string
	CreatedCount int
	Errors       []ImportRowError
}

type ImportRowError struct {
	Row     int
	Message string
}

type ImportTemplate struct {
	FileName string
	Content  []byte
}

func NewImportService(members ImportMemberTarget, courses ImportCourseTarget) *ImportService {
	return &ImportService{members: members, courses: courses}
}

func (s *ImportService) ImportMembers(ctx context.Context, reader io.Reader) (ImportResult, error) {
	if s.members == nil {
		return ImportResult{}, errors.New("member import target is not configured")
	}
	rows, err := readWorkbookRows(reader)
	if err != nil {
		return ImportResult{}, err
	}

	result := ImportResult{Kind: "members"}
	if !validateHeaders(rows, memberImportHeaders, &result) {
		return result, nil
	}

	for index, row := range rows[1:] {
		rowNumber := index + 2
		if blankRow(row) {
			continue
		}
		input := MemberInput{
			MemberNo:   cell(row, 0),
			Name:       cell(row, 1),
			GenderCode: normalizeGender(cell(row, 2)),
			BirthDate:  cell(row, 3),
			Phone:      cell(row, 4),
			Note:       cell(row, 5),
		}
		if strings.TrimSpace(input.Name) == "" {
			addImportError(&result, rowNumber, "name is required")
			continue
		}
		if _, err := s.members.Create(ctx, input); err != nil {
			addImportError(&result, rowNumber, err.Error())
			continue
		}
		result.CreatedCount++
	}
	return result, nil
}

func (s *ImportService) ImportCourseOfferings(ctx context.Context, reader io.Reader) (ImportResult, error) {
	if s.courses == nil {
		return ImportResult{}, errors.New("course import target is not configured")
	}
	rows, err := readWorkbookRows(reader)
	if err != nil {
		return ImportResult{}, err
	}

	result := ImportResult{Kind: "course_offerings"}
	if !validateHeaders(rows, courseImportHeaders, &result) {
		return result, nil
	}

	for index, row := range rows[1:] {
		rowNumber := index + 2
		if blankRow(row) {
			continue
		}
		capacity, err := parseCapacity(cell(row, 5))
		if err != nil {
			addImportError(&result, rowNumber, err.Error())
			continue
		}
		weekday, err := parseWeekday(cell(row, 6))
		if err != nil {
			addImportError(&result, rowNumber, err.Error())
			continue
		}
		input := CourseOfferingInput{
			TermName:       cell(row, 0),
			CategoryName:   cell(row, 1),
			CourseTitle:    cell(row, 2),
			DisplayName:    cell(row, 2),
			InstructorName: cell(row, 3),
			ClassroomName:  cell(row, 4),
			Capacity:       capacity.Total,
			CapacityType:   capacity.Type,
			MaleCapacity:   capacity.Male,
			FemaleCapacity: capacity.Female,
			Weekday:        weekday,
			StartTime:      cell(row, 7),
			EndTime:        cell(row, 8),
			Note:           cell(row, 9),
		}
		if strings.TrimSpace(input.CourseTitle) == "" {
			addImportError(&result, rowNumber, "course is required")
			continue
		}
		if _, err := s.courses.CreateOffering(ctx, input); err != nil {
			addImportError(&result, rowNumber, err.Error())
			continue
		}
		result.CreatedCount++
	}
	return result, nil
}

func (s *ImportService) MemberTemplate() (ImportTemplate, error) {
	file := excelize.NewFile()
	defer file.Close()

	sheet := "members"
	if err := setupSheet(file, sheet, memberImportHeaders); err != nil {
		return ImportTemplate{}, err
	}
	if err := setRow(file, sheet, 2, []any{"M-0001", "Kim Baeum", "female", "1955-04-12", "010-0000-0000", "sample"}); err != nil {
		return ImportTemplate{}, err
	}
	content, err := file.WriteToBuffer()
	if err != nil {
		return ImportTemplate{}, fmt.Errorf("write member import template: %w", err)
	}
	return ImportTemplate{FileName: "member-import-template.xlsx", Content: content.Bytes()}, nil
}

func (s *ImportService) CourseOfferingTemplate() (ImportTemplate, error) {
	file := excelize.NewFile()
	defer file.Close()

	sheet := "course_offerings"
	if err := setupSheet(file, sheet, courseImportHeaders); err != nil {
		return ImportTemplate{}, err
	}
	if err := setRow(file, sheet, 2, []any{"2026-2", "lifelong", "Korean basic", "Instructor", "Room 101", 20, "mon", "09:00", "10:00", "sample"}); err != nil {
		return ImportTemplate{}, err
	}
	content, err := file.WriteToBuffer()
	if err != nil {
		return ImportTemplate{}, fmt.Errorf("write course import template: %w", err)
	}
	return ImportTemplate{FileName: "course-import-template.xlsx", Content: content.Bytes()}, nil
}

func readWorkbookRows(reader io.Reader) ([][]string, error) {
	file, err := excelize.OpenReader(reader)
	if err != nil {
		return nil, fmt.Errorf("open import workbook: %w", err)
	}
	defer file.Close()

	sheets := file.GetSheetList()
	if len(sheets) == 0 {
		return nil, errors.New("import workbook has no sheets")
	}
	rows, err := file.GetRows(sheets[0])
	if err != nil {
		return nil, fmt.Errorf("read import sheet: %w", err)
	}
	return rows, nil
}

func validateHeaders(rows [][]string, want []string, result *ImportResult) bool {
	if len(rows) == 0 {
		addImportError(result, 1, "header row is missing")
		return false
	}
	for index, header := range want {
		if cell(rows[0], index) != header {
			addImportError(result, 1, fmt.Sprintf("column %d must be %q", index+1, header))
			return false
		}
	}
	return true
}

type capacityImportValue struct {
	Type   string
	Total  int
	Male   int
	Female int
}

func parseCapacity(value string) (capacityImportValue, error) {
	value = strings.TrimSpace(value)
	if value == "" {
		return capacityImportValue{Type: "fixed"}, nil
	}
	if value == "개방" || strings.EqualFold(value, "open") {
		return capacityImportValue{Type: "open"}, nil
	}
	if strings.Contains(value, "남") || strings.Contains(value, "여") {
		male, female, ok := parseGenderSplitCapacity(value)
		if !ok {
			return capacityImportValue{}, errors.New("gender split capacity must look like 남7/여7")
		}
		return capacityImportValue{Type: "gender_split", Total: male + female, Male: male, Female: female}, nil
	}
	capacity, err := strconv.Atoi(value)
	if err != nil {
		return capacityImportValue{}, errors.New("capacity must be a number, open, or gender split value")
	}
	if capacity < 0 {
		return capacityImportValue{}, errors.New("capacity must be zero or greater")
	}
	return capacityImportValue{Type: "fixed", Total: capacity}, nil
}

func parseGenderSplitCapacity(value string) (int, int, bool) {
	normalized := strings.NewReplacer(" ", "", ",", "/", "·", "/", "，", "/").Replace(value)
	parts := strings.Split(normalized, "/")
	male := -1
	female := -1
	for _, part := range parts {
		if strings.HasPrefix(part, "남") {
			parsed, err := strconv.Atoi(strings.TrimPrefix(part, "남"))
			if err != nil {
				return 0, 0, false
			}
			male = parsed
		}
		if strings.HasPrefix(part, "여") {
			parsed, err := strconv.Atoi(strings.TrimPrefix(part, "여"))
			if err != nil {
				return 0, 0, false
			}
			female = parsed
		}
	}
	return male, female, male >= 0 && female >= 0
}

func parseWeekday(value string) (int, error) {
	value = strings.ToLower(strings.TrimSpace(value))
	switch value {
	case "0", "sun", "sunday", "일", "일요일":
		return 0, nil
	case "1", "mon", "monday", "월", "월요일":
		return 1, nil
	case "2", "tue", "tuesday", "화", "화요일":
		return 2, nil
	case "3", "wed", "wednesday", "수", "수요일":
		return 3, nil
	case "4", "thu", "thursday", "목", "목요일":
		return 4, nil
	case "5", "fri", "friday", "금", "금요일":
		return 5, nil
	case "6", "sat", "saturday", "토", "토요일":
		return 6, nil
	default:
		return 0, errors.New("weekday must be 0-6 or a weekday label")
	}
}

func normalizeGender(value string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "m", "male", "남", "남성":
		return "male"
	case "f", "female", "여", "여성":
		return "female"
	case "unknown", "미상":
		return "unknown"
	default:
		return strings.TrimSpace(value)
	}
}

func addImportError(result *ImportResult, row int, message string) {
	result.Errors = append(result.Errors, ImportRowError{Row: row, Message: message})
}

func cell(row []string, index int) string {
	if index >= len(row) {
		return ""
	}
	return strings.TrimSpace(row[index])
}

func blankRow(row []string) bool {
	for _, value := range row {
		if strings.TrimSpace(value) != "" {
			return false
		}
	}
	return true
}
