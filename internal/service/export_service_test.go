package service

import (
	"context"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/xuri/excelize/v2"
)

func TestExportServiceExportsMembersWorkbook(t *testing.T) {
	service := NewExportService(
		&exportMemberSource{members: []domain.Member{{ID: 1, MemberNo: "M-001", Name: "김배움", GenderCode: "female", BirthDate: "1960-01-02", Phone: "010-0000-0000"}}},
		&exportCourseSource{},
		&exportRegistrationSource{},
		t.TempDir(),
	)
	service.now = func() time.Time { return time.Date(2026, 7, 1, 9, 30, 0, 0, time.Local) }

	result, err := service.ExportMembers(context.Background())
	if err != nil {
		t.Fatalf("ExportMembers() error = %v", err)
	}

	if !strings.HasSuffix(result.FileName, ".xlsx") {
		t.Fatalf("FileName = %q, want .xlsx suffix", result.FileName)
	}
	workbook, err := excelize.OpenFile(result.Path)
	if err != nil {
		t.Fatalf("OpenFile() error = %v", err)
	}
	defer workbook.Close()

	assertCell(t, workbook, "회원", "A1", "ID")
	assertCell(t, workbook, "회원", "C2", "김배움")
	assertCell(t, workbook, "회원", "F2", "010-0000-0000")
}

func TestExportServiceExportsCourseOfferingsWorkbook(t *testing.T) {
	service := NewExportService(
		&exportMemberSource{},
		&exportCourseSource{offerings: []domain.CourseOffering{{ID: 2, TermName: "2026 여름", CategoryName: "건강", CourseTitle: "요가 기초", InstructorName: "박강사", ClassroomName: "101호", Capacity: 20, RegistrationCount: 3, Weekday: 1, StartTime: "09:00", EndTime: "10:00", Status: "open", RegistrationEnabled: true}}},
		&exportRegistrationSource{},
		t.TempDir(),
	)
	service.now = func() time.Time { return time.Date(2026, 7, 1, 9, 31, 0, 0, time.Local) }

	result, err := service.ExportCourseOfferings(context.Background())
	if err != nil {
		t.Fatalf("ExportCourseOfferings() error = %v", err)
	}

	workbook, err := excelize.OpenFile(filepath.Clean(result.Path))
	if err != nil {
		t.Fatalf("OpenFile() error = %v", err)
	}
	defer workbook.Close()

	assertCell(t, workbook, "강좌", "D2", "요가 기초")
	assertCell(t, workbook, "강좌", "I2", "월")
	assertCell(t, workbook, "강좌", "M2", "예")
}

func TestExportServiceExportsRegistrationsWorkbook(t *testing.T) {
	service := NewExportService(
		&exportMemberSource{},
		&exportCourseSource{},
		&exportRegistrationSource{registrations: []domain.Registration{{ID: 3, MemberID: 1, MemberNo: "M-001", MemberName: "김배움", OfferingID: 2, CourseTitle: "요가 기초", TermName: "2026 여름", Status: "applied", CreatedAt: "2026-07-01 09:00:00"}}},
		t.TempDir(),
	)
	service.now = func() time.Time { return time.Date(2026, 7, 1, 9, 32, 0, 0, time.Local) }

	result, err := service.ExportRegistrations(context.Background())
	if err != nil {
		t.Fatalf("ExportRegistrations() error = %v", err)
	}

	workbook, err := excelize.OpenFile(result.Path)
	if err != nil {
		t.Fatalf("OpenFile() error = %v", err)
	}
	defer workbook.Close()

	assertCell(t, workbook, "신청", "C2", "M-001")
	assertCell(t, workbook, "신청", "D2", "김배움")
	assertCell(t, workbook, "신청", "F2", "요가 기초")
}

func assertCell(t *testing.T, workbook *excelize.File, sheet string, cell string, want string) {
	t.Helper()

	got, err := workbook.GetCellValue(sheet, cell)
	if err != nil {
		t.Fatalf("GetCellValue(%s, %s) error = %v", sheet, cell, err)
	}
	if got != want {
		t.Fatalf("cell %s!%s = %q, want %q", sheet, cell, got, want)
	}
}

type exportMemberSource struct {
	members []domain.Member
}

func (s *exportMemberSource) Search(context.Context, string, int) ([]domain.Member, error) {
	return s.members, nil
}

type exportCourseSource struct {
	offerings []domain.CourseOffering
}

func (s *exportCourseSource) ListOfferings(context.Context, int) ([]domain.CourseOffering, error) {
	return s.offerings, nil
}

type exportRegistrationSource struct {
	registrations []domain.Registration
}

func (s *exportRegistrationSource) ListRecent(context.Context, int) ([]domain.Registration, error) {
	return s.registrations, nil
}
