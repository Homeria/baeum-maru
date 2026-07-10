package launcher

import (
	"strings"
	"sync"
	"time"
)

type ActivityEntry struct {
	OccurredAt time.Time
	Message    string
}

type ActivityLog struct {
	mu      sync.RWMutex
	limit   int
	now     func() time.Time
	entries []ActivityEntry
}

func NewActivityLog(limit int) *ActivityLog {
	return newActivityLog(limit, time.Now)
}

func newActivityLog(limit int, now func() time.Time) *ActivityLog {
	if limit <= 0 {
		limit = 100
	}
	if now == nil {
		now = time.Now
	}
	return &ActivityLog{limit: limit, now: now}
}

func (l *ActivityLog) Append(message string) ActivityEntry {
	if l == nil {
		return ActivityEntry{}
	}
	entry := ActivityEntry{OccurredAt: l.now(), Message: message}

	l.mu.Lock()
	l.entries = append([]ActivityEntry{entry}, l.entries...)
	if len(l.entries) > l.limit {
		l.entries = l.entries[:l.limit]
	}
	l.mu.Unlock()
	return entry
}

func (l *ActivityLog) Entries() []ActivityEntry {
	if l == nil {
		return nil
	}
	l.mu.RLock()
	defer l.mu.RUnlock()
	return append([]ActivityEntry(nil), l.entries...)
}

func (l *ActivityLog) Text() string {
	entries := l.Entries()
	lines := make([]string, 0, len(entries))
	for _, entry := range entries {
		lines = append(lines, entry.OccurredAt.Format("15:04:05")+"  "+entry.Message)
	}
	return strings.Join(lines, "\n")
}

func (l *ActivityLog) Clear() {
	if l == nil {
		return
	}
	l.mu.Lock()
	l.entries = nil
	l.mu.Unlock()
}
