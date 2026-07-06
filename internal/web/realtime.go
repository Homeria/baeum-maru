package web

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"
)

type SyncEvent struct {
	ID         int64  `json:"id"`
	Scope      string `json:"scope"`
	Action     string `json:"action"`
	EntityType string `json:"entity_type,omitempty"`
	EntityID   int64  `json:"entity_id,omitempty"`
	Path       string `json:"path,omitempty"`
	OccurredAt string `json:"occurred_at"`
}

type SyncHub struct {
	mu          sync.Mutex
	nextID      int64
	subscribers map[chan SyncEvent]struct{}
}

func NewSyncHub() *SyncHub {
	return &SyncHub{subscribers: map[chan SyncEvent]struct{}{}}
}

func (h *SyncHub) Subscribe(ctx context.Context) <-chan SyncEvent {
	events := make(chan SyncEvent, 16)
	h.mu.Lock()
	h.subscribers[events] = struct{}{}
	h.mu.Unlock()

	go func() {
		<-ctx.Done()
		h.mu.Lock()
		delete(h.subscribers, events)
		h.mu.Unlock()
	}()
	return events
}

func (h *SyncHub) Publish(event SyncEvent) {
	if h == nil {
		return
	}
	h.mu.Lock()
	h.nextID++
	event.ID = h.nextID
	if event.OccurredAt == "" {
		event.OccurredAt = time.Now().UTC().Format(time.RFC3339)
	}
	subscribers := make([]chan SyncEvent, 0, len(h.subscribers))
	for subscriber := range h.subscribers {
		subscribers = append(subscribers, subscriber)
	}
	h.mu.Unlock()

	for _, subscriber := range subscribers {
		select {
		case subscriber <- event:
		default:
		}
	}
}

func syncEventsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		flusher, ok := w.(http.Flusher)
		if !ok {
			http.Error(w, "streaming is not supported", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "text/event-stream")
		w.Header().Set("Cache-Control", "no-cache")
		w.Header().Set("Connection", "keep-alive")
		w.Header().Set("X-Accel-Buffering", "no")

		writeSyncEvent(w, SyncEvent{
			Scope:      "system",
			Action:     "connected",
			Path:       r.URL.Path,
			OccurredAt: time.Now().UTC().Format(time.RFC3339),
		})
		flusher.Flush()

		events := opts.Sync.Subscribe(r.Context())
		heartbeat := time.NewTicker(25 * time.Second)
		defer heartbeat.Stop()

		for {
			select {
			case <-r.Context().Done():
				return
			case event, ok := <-events:
				if !ok {
					return
				}
				writeSyncEvent(w, event)
				flusher.Flush()
			case <-heartbeat.C:
				_, _ = fmt.Fprint(w, ": keep-alive\n\n")
				flusher.Flush()
			}
		}
	}
}

func writeSyncEvent(w http.ResponseWriter, event SyncEvent) {
	payload, err := json.Marshal(event)
	if err != nil {
		return
	}
	_, _ = fmt.Fprintf(w, "data: %s\n\n", payload)
}

func publishDataChange(r *http.Request, opts RouterOptions, action string, entityType string, entityID int64) {
	if opts.Sync == nil {
		return
	}
	scope := syncScopeForAudit(action, entityType)
	if scope == "" {
		return
	}
	path := ""
	if r != nil && r.URL != nil {
		path = r.URL.Path
	}
	opts.Sync.Publish(SyncEvent{
		Scope:      scope,
		Action:     action,
		EntityType: entityType,
		EntityID:   entityID,
		Path:       path,
	})
}

func syncScopeForAudit(action string, entityType string) string {
	switch {
	case strings.HasPrefix(action, "member."):
		return "members"
	case strings.HasPrefix(action, "course."):
		return "courses"
	case strings.HasPrefix(action, "location."):
		return "locations"
	case strings.HasPrefix(action, "registration."):
		return "registrations"
	case strings.HasPrefix(action, "lottery."):
		return "lottery"
	case strings.HasPrefix(action, "attendance."):
		return "attendance"
	case strings.HasPrefix(action, "settings."):
		return "settings"
	case strings.HasPrefix(action, "backup.restore"):
		return "all"
	case strings.HasPrefix(action, "backup."):
		return "backups"
	case strings.HasPrefix(action, "excel.import"):
		return "all"
	}
	switch entityType {
	case "member":
		return "members"
	case "course", "course_offering":
		return "courses"
	case "location":
		return "locations"
	case "registration":
		return "registrations"
	case "lottery_run":
		return "lottery"
	case "attendance_session", "attendance_record":
		return "attendance"
	case "settings":
		return "settings"
	case "backup":
		return "backups"
	default:
		return ""
	}
}
