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

	result := ImportResult{Kind: "회원"}
	if !validateHeaders(rows, []string{"회원번호", "이름", "성별", "생년월일", "연락처", "비고"}, &result) {
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
			addImportError(&result, rowNumber, "이름은 필수입니다.")
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

	result := ImportResult{Kind: "강좌"}
	if !validateHeaders(rows, []string{"회차", "분류", "강좌명", "강사", "강의실", "정원", "요일", "시작", "종료", "비고"}, &result) {
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
			InstructorName: cell(row, 3),
			ClassroomName:  cell(row, 4),
			Capacity:       capacity,
			Weekday:        weekday,
			StartTime:      cell(row, 7),
			EndTime:        cell(row, 8),
			Note:           cell(row, 9),
		}
		if strings.TrimSpace(input.CourseTitle) == "" {
			addImportError(&result, rowNumber, "강좌명은 필수입니다.")
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

	sheet := "회원 가져오기"
	if err := setupSheet(file, sheet, []string{"회원번호", "이름", "성별", "생년월일", "연락처", "비고"}); err != nil {
		return ImportTemplate{}, err
	}
	if err := setRow(file, sheet, 2, []any{"M-0001", "김배움", "female", "1955-04-12", "010-0000-0000", "샘플 행"}); err != nil {
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

	sheet := "강좌 가져오기"
	if err := setupSheet(file, sheet, []string{"회차", "분류", "강좌명", "강사", "강의실", "정원", "요일", "시작", "종료", "비고"}); err != nil {
		return ImportTemplate{}, err
	}
	if err := setRow(file, sheet, 2, []any{"2026년 여름학기", "건강", "요가 기초", "이강사", "101호", 20, "월", "09:00", "10:00", "샘플 행"}); err != nil {
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
		addImportError(result, 1, "헤더 행이 없습니다.")
		return false
	}
	for index, header := range want {
		if cell(rows[0], index) != header {
			addImportError(result, 1, fmt.Sprintf("%d번째 열은 %q이어야 합니다.", index+1, header))
			return false
		}
	}
	return true
}

func parseCapacity(value string) (int, error) {
	value = strings.TrimSpace(value)
	if value == "" {
		return 0, nil
	}
	capacity, err := strconv.Atoi(value)
	if err != nil {
		return 0, errors.New("정원은 숫자로 입력해야 합니다.")
	}
	if capacity < 0 {
		return 0, errors.New("정원은 0 이상이어야 합니다.")
	}
	return capacity, nil
}

func parseWeekday(value string) (int, error) {
	value = strings.TrimSpace(value)
	switch value {
	case "0", "일", "일요일":
		return 0, nil
	case "1", "월", "월요일":
		return 1, nil
	case "2", "화", "화요일":
		return 2, nil
	case "3", "수", "수요일":
		return 3, nil
	case "4", "목", "목요일":
		return 4, nil
	case "5", "금", "금요일":
		return 5, nil
	case "6", "토", "토요일":
		return 6, nil
	default:
		return 0, errors.New("요일은 일, 월, 화, 수, 목, 금, 토 또는 0~6으로 입력해야 합니다.")
	}
}

func normalizeGender(value string) string {
	switch strings.TrimSpace(value) {
	case "남", "남성", "male":
		return "male"
	case "여", "여성", "female":
		return "female"
	case "미상", "unknown":
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
