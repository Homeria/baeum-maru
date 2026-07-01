package service

import (
	"bytes"
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/xuri/excelize/v2"
)

func TestImportServiceImportsMembers(t *testing.T) {
	var members fakeImportMembers
	service := NewImportService(&members, nil)
	content := workbookBytes(t, "회원 가져오기", []string{"회원번호", "이름", "성별", "생년월일", "연락처", "비고"}, [][]any{
		{"M-001", "김배움", "여", "1955-04-12", "010-0000-0000", "첫 행"},
		{"M-002", "", "남", "", "", "이름 없음"},
	})

	result, err := service.ImportMembers(context.Background(), bytes.NewReader(content))
	if err != nil {
		t.Fatalf("ImportMembers() error = %v", err)
	}
	if result.CreatedCount != 1 {
		t.Fatalf("CreatedCount = %d, want 1", result.CreatedCount)
	}
	if len(result.Errors) != 1 || result.Errors[0].Row != 3 {
		t.Fatalf("Errors = %#v, want row 3 error", result.Errors)
	}
	if got := members.created[0].GenderCode; got != "female" {
		t.Fatalf("GenderCode = %q, want female", got)
	}
}

func TestImportServiceImportsCourseOfferings(t *testing.T) {
	var courses fakeImportCourses
	service := NewImportService(nil, &courses)
	content := workbookBytes(t, "강좌 가져오기", []string{"회차", "분류", "강좌명", "강사", "강의실", "정원", "요일", "시작", "종료", "비고"}, [][]any{
		{"2026년 여름학기", "건강", "요가 기초", "이강사", "101호", 20, "월", "09:00", "10:00", ""},
		{"2026년 여름학기", "건강", "탁구", "", "", "많음", "화", "10:00", "11:00", ""},
	})

	result, err := service.ImportCourseOfferings(context.Background(), bytes.NewReader(content))
	if err != nil {
		t.Fatalf("ImportCourseOfferings() error = %v", err)
	}
	if result.CreatedCount != 1 {
		t.Fatalf("CreatedCount = %d, want 1", result.CreatedCount)
	}
	if len(result.Errors) != 1 || result.Errors[0].Row != 3 {
		t.Fatalf("Errors = %#v, want row 3 error", result.Errors)
	}
	if got := courses.created[0].Weekday; got != 1 {
		t.Fatalf("Weekday = %d, want 1", got)
	}
}

func TestImportServiceCreatesTemplates(t *testing.T) {
	service := NewImportService(nil, nil)
	template, err := service.MemberTemplate()
	if err != nil {
		t.Fatalf("MemberTemplate() error = %v", err)
	}
	workbook, err := excelize.OpenReader(bytes.NewReader(template.Content))
	if err != nil {
		t.Fatalf("OpenReader() error = %v", err)
	}
	defer workbook.Close()
	cell, err := workbook.GetCellValue("회원 가져오기", "B1")
	if err != nil {
		t.Fatalf("GetCellValue() error = %v", err)
	}
	if cell != "이름" {
		t.Fatalf("B1 = %q, want 이름", cell)
	}
}

func workbookBytes(t *testing.T, sheet string, headers []string, rows [][]any) []byte {
	t.Helper()
	file := excelize.NewFile()
	defer file.Close()
	if err := setupSheet(file, sheet, headers); err != nil {
		t.Fatalf("setupSheet() error = %v", err)
	}
	for index, row := range rows {
		if err := setRow(file, sheet, index+2, row); err != nil {
			t.Fatalf("setRow() error = %v", err)
		}
	}
	content, err := file.WriteToBuffer()
	if err != nil {
		t.Fatalf("WriteToBuffer() error = %v", err)
	}
	return content.Bytes()
}

type fakeImportMembers struct {
	created []MemberInput
}

func (f *fakeImportMembers) Create(_ context.Context, input MemberInput) (domain.Member, error) {
	f.created = append(f.created, input)
	return domain.Member{ID: int64(len(f.created)), Name: input.Name, GenderCode: input.GenderCode}, nil
}

type fakeImportCourses struct {
	created []CourseOfferingInput
}

func (f *fakeImportCourses) CreateOffering(_ context.Context, input CourseOfferingInput) (domain.CourseOffering, error) {
	f.created = append(f.created, input)
	return domain.CourseOffering{ID: int64(len(f.created)), CourseTitle: input.CourseTitle, Weekday: input.Weekday}, nil
}
