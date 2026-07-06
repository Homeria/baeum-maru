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
	content := workbookBytes(t, "members", memberImportHeaders, [][]any{
		{"M-001", "Kim Baeum", "female", "1955-04-12", "010-0000-0000", "first"},
		{"M-002", "", "male", "", "", "missing name"},
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
	content := workbookBytes(t, "course_offerings", courseImportHeaders, [][]any{
		{"2026-2", "lifelong", "Korean basic", "Instructor", "Room 101", 20, "mon", "09:00", "10:00", ""},
		{"2026-2", "lifelong", "Dance", "", "", "many", "mon", "10:00", "11:00", ""},
		{"2026-2", "lifelong", "Social dance", "", "", "남7/여7", "thu", "13:00", "14:00", ""},
		{"2026-2", "lifelong", "Open singing", "", "", "open", "fri", "10:00", "11:00", ""},
	})

	result, err := service.ImportCourseOfferings(context.Background(), bytes.NewReader(content))
	if err != nil {
		t.Fatalf("ImportCourseOfferings() error = %v", err)
	}
	if result.CreatedCount != 3 {
		t.Fatalf("CreatedCount = %d, want 3", result.CreatedCount)
	}
	if len(result.Errors) != 1 || result.Errors[0].Row != 3 {
		t.Fatalf("Errors = %#v, want row 3 error", result.Errors)
	}
	if got := courses.created[0].Weekday; got != 1 {
		t.Fatalf("Weekday = %d, want 1", got)
	}
	if got := courses.created[1].CapacityType; got != "gender_split" {
		t.Fatalf("CapacityType = %q, want gender_split", got)
	}
	if courses.created[1].MaleCapacity != 7 || courses.created[1].FemaleCapacity != 7 {
		t.Fatalf("gender capacity = %d/%d, want 7/7", courses.created[1].MaleCapacity, courses.created[1].FemaleCapacity)
	}
	if got := courses.created[2].CapacityType; got != "open" {
		t.Fatalf("CapacityType = %q, want open", got)
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
	cell, err := workbook.GetCellValue("members", "B1")
	if err != nil {
		t.Fatalf("GetCellValue() error = %v", err)
	}
	if cell != "name" {
		t.Fatalf("B1 = %q, want name", cell)
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
