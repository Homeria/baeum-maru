package service

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/xuri/excelize/v2"
)

const exportLimit = 10000

type ExportResult struct {
	Path     string
	FileName string
}

type ExportMemberSource interface {
	Search(context.Context, string, int) ([]domain.Member, error)
}

type ExportCourseSource interface {
	ListOfferings(context.Context, int) ([]domain.CourseOffering, error)
}

type ExportRegistrationSource interface {
	ListRecent(context.Context, int) ([]domain.Registration, error)
}

type ExportLotterySource interface {
	ListResultsByRun(context.Context, int64) ([]domain.LotteryResultRow, error)
}

type ExportAttendanceSource interface {
	ListSessions(context.Context, int64, int) ([]domain.AttendanceSession, error)
	ListRecordsBySession(context.Context, int64) ([]domain.AttendanceRecord, error)
}

type ExportService struct {
	members       ExportMemberSource
	courses       ExportCourseSource
	registrations ExportRegistrationSource
	lotteries     ExportLotterySource
	attendance    ExportAttendanceSource
	dir           string
	now           func() time.Time
}

func NewExportService(
	members ExportMemberSource,
	courses ExportCourseSource,
	registrations ExportRegistrationSource,
	dir string,
	optionalSources ...any,
) *ExportService {
	service := &ExportService{
		members:       members,
		courses:       courses,
		registrations: registrations,
		dir:           dir,
		now:           time.Now,
	}
	for _, source := range optionalSources {
		switch typed := source.(type) {
		case ExportLotterySource:
			service.lotteries = typed
		case ExportAttendanceSource:
			service.attendance = typed
		}
	}
	return service
}

func (s *ExportService) ExportMembers(ctx context.Context) (ExportResult, error) {
	members, err := s.members.Search(ctx, "", exportLimit)
	if err != nil {
		return ExportResult{}, fmt.Errorf("list members for export: %w", err)
	}

	file := excelize.NewFile()
	defer file.Close()

	sheet := "회원"
	if err := setupSheet(file, sheet, []string{"ID", "회원번호", "이름", "성별", "생년월일", "연락처", "비고", "등록일", "수정일"}); err != nil {
		return ExportResult{}, err
	}
	for i, member := range members {
		row := i + 2
		values := []any{member.ID, member.MemberNo, member.Name, member.GenderCode, member.BirthDate, member.Phone, member.Note, member.CreatedAt, member.UpdatedAt}
		if err := setRow(file, sheet, row, values); err != nil {
			return ExportResult{}, err
		}
	}

	return s.save(file, "members")
}

func (s *ExportService) ExportCourseOfferings(ctx context.Context) (ExportResult, error) {
	offerings, err := s.courses.ListOfferings(ctx, exportLimit)
	if err != nil {
		return ExportResult{}, fmt.Errorf("list course offerings for export: %w", err)
	}

	file := excelize.NewFile()
	defer file.Close()

	sheet := "강좌"
	if err := setupSheet(file, sheet, []string{"ID", "회차", "분류", "강좌명", "강사", "강의실", "정원", "신청수", "요일", "시작", "종료", "상태", "신청 가능"}); err != nil {
		return ExportResult{}, err
	}
	for i, offering := range offerings {
		row := i + 2
		values := []any{
			offering.ID,
			offering.TermName,
			offering.CategoryName,
			offering.CourseTitle,
			offering.InstructorName,
			offering.ClassroomName,
			offering.Capacity,
			offering.RegistrationCount,
			weekdayName(offering.Weekday),
			offering.StartTime,
			offering.EndTime,
			offering.Status,
			boolLabel(offering.RegistrationEnabled),
		}
		if err := setRow(file, sheet, row, values); err != nil {
			return ExportResult{}, err
		}
	}

	return s.save(file, "course-offerings")
}

func (s *ExportService) ExportRegistrations(ctx context.Context) (ExportResult, error) {
	registrations, err := s.registrations.ListRecent(ctx, exportLimit)
	if err != nil {
		return ExportResult{}, fmt.Errorf("list registrations for export: %w", err)
	}

	file := excelize.NewFile()
	defer file.Close()

	sheet := "신청"
	if err := setupSheet(file, sheet, []string{"ID", "회원 ID", "회원번호", "회원명", "강좌 ID", "강좌명", "회차", "상태", "신청일", "수정일", "취소일"}); err != nil {
		return ExportResult{}, err
	}
	for i, registration := range registrations {
		row := i + 2
		values := []any{
			registration.ID,
			registration.MemberID,
			registration.MemberNo,
			registration.MemberName,
			registration.OfferingID,
			registration.CourseTitle,
			registration.TermName,
			registration.Status,
			registration.CreatedAt,
			registration.UpdatedAt,
			registration.CancelledAt,
		}
		if err := setRow(file, sheet, row, values); err != nil {
			return ExportResult{}, err
		}
	}

	return s.save(file, "registrations")
}

func (s *ExportService) ExportLotteryResults(ctx context.Context, runID int64) (ExportResult, error) {
	if runID <= 0 {
		return ExportResult{}, errors.New("lottery run id is required")
	}
	if s.lotteries == nil {
		return ExportResult{}, errors.New("lottery export source is not configured")
	}
	results, err := s.lotteries.ListResultsByRun(ctx, runID)
	if err != nil {
		return ExportResult{}, fmt.Errorf("list lottery results for export: %w", err)
	}

	file := excelize.NewFile()
	defer file.Close()

	sheet := "추첨 결과"
	if err := setupSheet(file, sheet, []string{"추첨 ID", "회차", "강좌 ID", "강좌명", "결과", "순번", "신청 ID", "회원 ID", "회원번호", "회원명", "신청일", "완료일", "Seed"}); err != nil {
		return ExportResult{}, err
	}
	for i, result := range results {
		row := i + 2
		values := []any{
			result.RunID,
			result.TermName,
			result.OfferingID,
			result.CourseTitle,
			result.Result,
			result.ResultOrder,
			result.RegistrationID,
			result.MemberID,
			result.MemberNo,
			result.MemberName,
			result.RegistrationCreatedAt,
			result.CompletedAt,
			result.Seed,
		}
		if err := setRow(file, sheet, row, values); err != nil {
			return ExportResult{}, err
		}
	}

	return s.save(file, fmt.Sprintf("lottery-results-%d", runID))
}

func (s *ExportService) ExportAttendanceSession(ctx context.Context, sessionID int64) (ExportResult, error) {
	if sessionID <= 0 {
		return ExportResult{}, errors.New("attendance session id is required")
	}
	if s.attendance == nil {
		return ExportResult{}, errors.New("attendance export source is not configured")
	}
	records, err := s.attendance.ListRecordsBySession(ctx, sessionID)
	if err != nil {
		return ExportResult{}, fmt.Errorf("list attendance records for export: %w", err)
	}

	file := excelize.NewFile()
	defer file.Close()

	sheet := "출석"
	if err := setupAttendanceSheet(file, sheet); err != nil {
		return ExportResult{}, err
	}
	if err := writeAttendanceRecords(file, sheet, 2, records); err != nil {
		return ExportResult{}, err
	}
	return s.save(file, fmt.Sprintf("attendance-session-%d", sessionID))
}

func (s *ExportService) ExportAttendanceOffering(ctx context.Context, offeringID int64) (ExportResult, error) {
	if offeringID <= 0 {
		return ExportResult{}, errors.New("course offering id is required")
	}
	if s.attendance == nil {
		return ExportResult{}, errors.New("attendance export source is not configured")
	}
	sessions, err := s.attendance.ListSessions(ctx, offeringID, exportLimit)
	if err != nil {
		return ExportResult{}, fmt.Errorf("list attendance sessions for export: %w", err)
	}

	file := excelize.NewFile()
	defer file.Close()

	sheet := "출석"
	if err := setupAttendanceSheet(file, sheet); err != nil {
		return ExportResult{}, err
	}
	row := 2
	for _, session := range sessions {
		records, err := s.attendance.ListRecordsBySession(ctx, session.ID)
		if err != nil {
			return ExportResult{}, fmt.Errorf("list attendance records for session %d: %w", session.ID, err)
		}
		if err := writeAttendanceRecords(file, sheet, row, records); err != nil {
			return ExportResult{}, err
		}
		row += len(records)
	}
	return s.save(file, fmt.Sprintf("attendance-offering-%d", offeringID))
}

func setupAttendanceSheet(file *excelize.File, sheet string) error {
	return setupSheet(file, sheet, []string{"회차 ID", "강좌 ID", "수업일", "신청 ID", "회원 ID", "회원번호", "회원명", "출석 상태", "비고"})
}

func writeAttendanceRecords(file *excelize.File, sheet string, startRow int, records []domain.AttendanceRecord) error {
	for i, record := range records {
		row := startRow + i
		values := []any{
			record.SessionID,
			record.OfferingID,
			record.SessionDate,
			record.RegistrationID,
			record.MemberID,
			record.MemberNo,
			record.MemberName,
			record.Status,
			record.Note,
		}
		if err := setRow(file, sheet, row, values); err != nil {
			return err
		}
	}
	return nil
}

func setupSheet(file *excelize.File, sheet string, headers []string) error {
	defaultSheet := file.GetSheetName(0)
	if err := file.SetSheetName(defaultSheet, sheet); err != nil {
		return fmt.Errorf("rename export sheet: %w", err)
	}
	if err := setRow(file, sheet, 1, toAny(headers)); err != nil {
		return err
	}
	style, err := file.NewStyle(&excelize.Style{Font: &excelize.Font{Bold: true}})
	if err != nil {
		return fmt.Errorf("create export header style: %w", err)
	}
	lastCell, err := excelize.CoordinatesToCellName(len(headers), 1)
	if err != nil {
		return fmt.Errorf("resolve export header range: %w", err)
	}
	if err := file.SetCellStyle(sheet, "A1", lastCell, style); err != nil {
		return fmt.Errorf("style export header: %w", err)
	}
	if err := file.SetColWidth(sheet, "A", "Z", 14); err != nil {
		return fmt.Errorf("set export column width: %w", err)
	}
	return nil
}

func setRow(file *excelize.File, sheet string, row int, values []any) error {
	for column, value := range values {
		cell, err := excelize.CoordinatesToCellName(column+1, row)
		if err != nil {
			return fmt.Errorf("resolve export cell: %w", err)
		}
		if err := file.SetCellValue(sheet, cell, value); err != nil {
			return fmt.Errorf("write export cell %s: %w", cell, err)
		}
	}
	return nil
}

func toAny(values []string) []any {
	converted := make([]any, len(values))
	for i, value := range values {
		converted[i] = value
	}
	return converted
}

func (s *ExportService) save(file *excelize.File, prefix string) (ExportResult, error) {
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		return ExportResult{}, fmt.Errorf("create export directory: %w", err)
	}
	fileName := fmt.Sprintf("%s-%s.xlsx", prefix, s.now().Format("20060102-150405"))
	path := filepath.Join(s.dir, fileName)
	if err := file.SaveAs(path); err != nil {
		return ExportResult{}, fmt.Errorf("save export file: %w", err)
	}
	return ExportResult{Path: path, FileName: fileName}, nil
}

func weekdayName(weekday int) string {
	switch weekday {
	case 0:
		return "일"
	case 1:
		return "월"
	case 2:
		return "화"
	case 3:
		return "수"
	case 4:
		return "목"
	case 5:
		return "금"
	case 6:
		return "토"
	default:
		return ""
	}
}

func boolLabel(value bool) string {
	if value {
		return "예"
	}
	return "아니오"
}
