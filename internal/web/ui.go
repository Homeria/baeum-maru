package web

import (
	"fmt"
	"html/template"
)

func uiTemplateFuncs(extra template.FuncMap) template.FuncMap {
	funcs := template.FuncMap{
		"humanBytes":  humanBytes,
		"statusLabel": statusLabel,
		"statusClass": statusClass,
	}
	for name, fn := range extra {
		funcs[name] = fn
	}
	return funcs
}

func humanBytes(size int64) string {
	const unit = 1024
	if size < unit {
		return formatByteSize(float64(size), "B")
	}
	value := float64(size)
	for _, suffix := range []string{"KiB", "MiB", "GiB"} {
		value /= unit
		if value < unit {
			return formatByteSize(value, suffix)
		}
	}
	return formatByteSize(value/unit, "TiB")
}

func formatByteSize(value float64, suffix string) string {
	if suffix == "B" {
		return formatWholeBytes(int64(value), suffix)
	}
	if value >= 10 {
		return formatWholeBytes(int64(value+0.5), suffix)
	}
	return formatOneDecimal(value, suffix)
}

func formatWholeBytes(value int64, suffix string) string {
	return fmt.Sprintf("%d %s", value, suffix)
}

func formatOneDecimal(value float64, suffix string) string {
	return fmt.Sprintf("%.1f %s", value, suffix)
}

func statusLabel(status string) string {
	switch status {
	case "applied":
		return "신청"
	case "selected":
		return "선정"
	case "waitlisted":
		return "대기"
	case "confirmed":
		return "확정"
	case "cancelled":
		return "취소"
	case "rejected":
		return "탈락"
	case "completed":
		return "완료"
	case "failed":
		return "실패"
	case "present":
		return "출석"
	case "absent":
		return "결석"
	case "late":
		return "지각"
	case "excused":
		return "공결"
	default:
		if status == "" {
			return "-"
		}
		return status
	}
}

func statusClass(status string) string {
	switch status {
	case "applied", "selected", "waitlisted", "confirmed", "cancelled", "rejected", "completed", "failed", "present", "absent", "late", "excused":
		return status
	default:
		return "pending"
	}
}
