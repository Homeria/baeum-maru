
(() => {
  if (!("EventSource" in window)) {
    return;
  }
  if (location.pathname === "/login" || location.pathname.startsWith("/static/")) {
    return;
  }

  const reloadDelayMs = 700;
  let reloadTimer = 0;
  let formIsDirty = false;

  document.addEventListener("input", (event) => {
    if (event.target && event.target.closest("form")) {
      formIsDirty = true;
    }
  }, true);
  document.addEventListener("change", (event) => {
    if (event.target && event.target.closest("form")) {
      formIsDirty = true;
    }
  }, true);
  document.addEventListener("submit", () => {
    formIsDirty = false;
  }, true);

  const source = new EventSource("/events");
  source.onmessage = (event) => {
    let payload;
    try {
      payload = JSON.parse(event.data);
    } catch {
      return;
    }
    if (!payload) {
      return;
    }
    if (payload.action === "connected" || payload.scope === "system") {
      hideSyncConnectionNotice();
      return;
    }
    if (!shouldRefreshForScope(payload.scope, location.pathname)) {
      return;
    }
    if (formIsDirty || isEditableFocused()) {
      showSyncNotice();
      return;
    }
    window.clearTimeout(reloadTimer);
    reloadTimer = window.setTimeout(() => {
      location.reload();
    }, reloadDelayMs);
  };
  source.onerror = () => {
    showSyncConnectionNotice();
  };

  function shouldRefreshForScope(scope, path) {
    if (scope === "all") {
      return true;
    }
    const groups = {
      members: ["/admin/members", "/reception", "/admin/registrations", "/admin/attendance"],
      courses: ["/admin/courses", "/reception", "/admin/registrations", "/admin/lottery", "/admin/attendance"],
      locations: ["/admin/locations", "/admin/courses"],
      registrations: ["/reception", "/admin/registrations", "/admin/lottery", "/admin/attendance"],
      lottery: ["/admin/lottery", "/admin/registrations"],
      attendance: ["/admin/attendance"],
      settings: ["/admin/settings"],
      backups: ["/admin/backups"],
    };
    return (groups[scope] || []).some((prefix) => path === prefix || path.startsWith(prefix + "?"));
  }

  function isEditableFocused() {
    const element = document.activeElement;
    if (!element) {
      return false;
    }
    const tagName = element.tagName;
    return tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT" || element.isContentEditable;
  }

  function showSyncNotice() {
    const notice = ensureNotice();
    notice.classList.remove("is-muted");
    notice.querySelector("[data-sync-message]").textContent = "다른 사용자가 데이터를 변경했습니다. 입력 중인 내용을 확인한 뒤 새로고침하세요.";
    notice.hidden = false;
  }

  function showSyncConnectionNotice() {
    const notice = ensureNotice();
    notice.classList.add("is-muted");
    notice.querySelector("[data-sync-message]").textContent = "실시간 동기화 연결을 다시 시도하고 있습니다.";
    notice.hidden = false;
  }

  function hideSyncConnectionNotice() {
    const notice = document.querySelector("[data-sync-notice]");
    if (notice && notice.classList.contains("is-muted")) {
      notice.hidden = true;
    }
  }

  function ensureNotice() {
    let notice = document.querySelector("[data-sync-notice]");
    if (notice) {
      return notice;
    }
    notice = document.createElement("div");
    notice.className = "sync-notice";
    notice.dataset.syncNotice = "true";
    notice.hidden = true;
    notice.innerHTML = `
      <span data-sync-message></span>
      <button type="button" data-sync-refresh>새로고침</button>
    `;
    notice.querySelector("[data-sync-refresh]").addEventListener("click", () => {
      location.reload();
    });
    document.body.appendChild(notice);
    return notice;
  }
})();
