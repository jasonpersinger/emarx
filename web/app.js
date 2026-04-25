const state = {
  catalog: [],
  works: new Map(),
  activeWorkId: null,
  activeSectionIndex: 0,
  query: "",
  theme: localStorage.getItem("emarx.theme") || "dark",
  bookmarks: JSON.parse(localStorage.getItem("emarx.bookmarks") || "[]"),
};

const el = {
  splash: document.querySelector("#splash"),
  splashBook: document.querySelector(".splash-book"),
  reader: document.querySelector("#reader"),
  workList: document.querySelector("#workList"),
  sectionList: document.querySelector("#sectionList"),
  workTitle: document.querySelector("#workTitle"),
  workMeta: document.querySelector("#workMeta"),
  sourceLink: document.querySelector("#sourceLink"),
  sectionNumber: document.querySelector("#sectionNumber"),
  sectionTitle: document.querySelector("#sectionTitle"),
  paragraphs: document.querySelector("#paragraphs"),
  searchInput: document.querySelector("#searchInput"),
  searchResults: document.querySelector("#searchResults"),
  progressLabel: document.querySelector("#progressLabel"),
  progressBar: document.querySelector("#progressBar"),
  themeButton: document.querySelector("#themeButton"),
  bookmarkButton: document.querySelector("#bookmarkButton"),
};

async function init() {
  if (!["dark", "red", "mono"].includes(state.theme)) state.theme = "dark";
  document.documentElement.dataset.theme = state.theme === "dark" ? "" : state.theme;
  state.catalog = await fetchJson("./data/catalog.json");
  renderWorks();
  await selectWork(state.catalog[0].id, 0);
  bindEvents();
}

async function fetchJson(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Failed to load ${url}`);
  return response.json();
}

function bindEvents() {
  const openReader = () => {
    el.splash.hidden = true;
    el.reader.hidden = false;
    el.workList.querySelector(".active")?.focus();
  };

  const openSplash = () => {
    el.reader.hidden = true;
    el.splash.hidden = false;
    el.splashBook.focus();
  };

  el.splash.addEventListener("click", openReader);
  el.splash.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      openReader();
    }
  });

  document.addEventListener("keydown", (event) => {
    if (el.reader.hidden) return;
    const activeTag = document.activeElement?.tagName?.toLowerCase();
    if (activeTag === "input") return;

    if (event.key.toLowerCase() === "q" && !el.reader.hidden) {
      openSplash();
      return;
    }

    if (event.key === "/") {
      event.preventDefault();
      el.searchInput.focus();
      return;
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      focusNextPane(1);
      return;
    }

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      focusNextPane(-1);
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      const button = document.activeElement?.closest?.(".work-button, .section-button");
      if (!button) return;
      event.preventDefault();
      focusAdjacentButton(button, event.key === "ArrowDown" ? 1 : -1);
    }
  });

  el.searchInput.addEventListener("input", async (event) => {
    state.query = event.target.value.trim();
    if (state.query.length < 2) {
      el.searchResults.hidden = true;
      renderText();
      return;
    }
    await loadAllWorks();
    renderSearchResults(search(state.query));
    renderText();
  });

  el.themeButton.addEventListener("click", () => {
    const themes = ["dark", "red", "mono"];
    state.theme = themes[(themes.indexOf(state.theme) + 1) % themes.length];
    localStorage.setItem("emarx.theme", state.theme);
    document.documentElement.dataset.theme = state.theme === "dark" ? "" : state.theme;
  });

  el.bookmarkButton.addEventListener("click", toggleBookmark);

  if (window.location.hash === "#reader") openReader();
}

function focusNextPane(direction) {
  const panes = [el.workList, el.sectionList, document.querySelector(".text-scroll")];
  const activePane = document.activeElement?.closest?.(".works-pane")
    ? 0
    : document.activeElement?.closest?.(".sections-pane")
      ? 1
      : 2;
  const next = (activePane + direction + panes.length) % panes.length;
  if (next === 0 || next === 1) {
    panes[next].querySelector(".active")?.focus();
  } else {
    panes[next].focus();
  }
}

function focusAdjacentButton(button, direction) {
  const buttons = [...button.parentElement.querySelectorAll("button")];
  const index = buttons.indexOf(button);
  const next = buttons[Math.max(0, Math.min(buttons.length - 1, index + direction))];
  next?.focus();
}

function renderWorks() {
  el.workList.replaceChildren(
    ...state.catalog.map((work) => {
      const button = document.createElement("button");
      button.className = "work-button";
      button.type = "button";
      button.innerHTML = `${escapeHtml(work.title)}<small>${work.year} · ${work.sections} sections · ${work.paragraphs} paragraphs</small>`;
      button.addEventListener("click", () => selectWork(work.id, 0));
      if (work.id === state.activeWorkId) button.classList.add("active");
      return button;
    }),
  );
}

async function selectWork(workId, sectionIndex = 0) {
  state.activeWorkId = workId;
  state.activeSectionIndex = sectionIndex;
  await loadWork(workId);
  renderWorks();
  renderHeader();
  renderSections();
  renderText();
}

async function loadWork(workId) {
  if (!state.works.has(workId)) {
    state.works.set(workId, await fetchJson(`./data/works/${workId}.json`));
  }
  return state.works.get(workId);
}

async function loadAllWorks() {
  await Promise.all(state.catalog.map((work) => loadWork(work.id)));
}

function activeWork() {
  return state.works.get(state.activeWorkId);
}

function activeMeta() {
  return state.catalog.find((work) => work.id === state.activeWorkId);
}

function renderHeader() {
  const meta = activeMeta();
  el.workTitle.textContent = meta.title;
  el.workMeta.textContent = `${meta.authors.join(", ")} · ${meta.year} · ${meta.paragraphs} paragraphs`;
  el.sourceLink.href = meta.source_url;
}

function renderSections() {
  const work = activeWork();
  el.sectionList.replaceChildren(
    ...work.sections.map((section, index) => {
      const button = document.createElement("button");
      button.className = "section-button";
      button.type = "button";
      button.innerHTML = `${index + 1}. ${escapeHtml(section.title)}<small>${section.paragraphs.length} paragraphs</small>`;
      button.addEventListener("click", () => {
        state.activeSectionIndex = index;
        renderSections();
        renderText();
      });
      if (index === state.activeSectionIndex) button.classList.add("active");
      return button;
    }),
  );
}

function renderText() {
  const work = activeWork();
  const section = work.sections[state.activeSectionIndex];
  const progress = Math.round(((state.activeSectionIndex + 1) / work.sections.length) * 100);
  el.progressLabel.textContent = `${progress}%`;
  el.progressBar.style.width = `${progress}%`;
  el.sectionNumber.textContent = `Section ${state.activeSectionIndex + 1} of ${work.sections.length}`;
  el.sectionTitle.textContent = section.title;
  el.paragraphs.replaceChildren(
    ...section.paragraphs.map((paragraph) => {
      const p = document.createElement("p");
      p.innerHTML = highlight(escapeHtml(paragraph), state.query);
      return p;
    }),
  );
  renderBookmarkButton();
}

function search(query) {
  const terms = normalize(query).split(" ").filter(Boolean);
  if (!terms.length) return [];
  const results = [];
  for (const work of state.works.values()) {
    work.sections.forEach((section, sectionIndex) => {
      section.paragraphs.forEach((paragraph, paragraphIndex) => {
        const haystack = normalize(`${work.title} ${section.title} ${paragraph}`);
        if (terms.every((term) => haystack.includes(term))) {
          results.push({
            work,
            section,
            sectionIndex,
            paragraphIndex,
            snippet: makeSnippet(paragraph, terms[0]),
          });
        }
      });
    });
  }
  return results.slice(0, 25);
}

function renderSearchResults(results) {
  el.searchResults.hidden = false;
  if (!results.length) {
    el.searchResults.textContent = "No passages found.";
    return;
  }
  el.searchResults.replaceChildren(
    ...results.map((result) => {
      const button = document.createElement("button");
      button.className = "result-button";
      button.type = "button";
      button.innerHTML = `<strong>${escapeHtml(result.work.title)}</strong><br><span>${escapeHtml(result.section.title)}</span><br>${highlight(escapeHtml(result.snippet), state.query)}`;
      button.addEventListener("click", async () => {
        await selectWork(result.work.id, result.sectionIndex);
        el.searchResults.hidden = true;
      });
      return button;
    }),
  );
}

function toggleBookmark() {
  const key = `${state.activeWorkId}:${state.activeSectionIndex}`;
  const existing = state.bookmarks.findIndex((bookmark) => bookmark.key === key);
  if (existing >= 0) {
    state.bookmarks.splice(existing, 1);
  } else {
    state.bookmarks.push({
      key,
      workId: state.activeWorkId,
      sectionIndex: state.activeSectionIndex,
      savedAt: new Date().toISOString(),
    });
  }
  localStorage.setItem("emarx.bookmarks", JSON.stringify(state.bookmarks));
  renderBookmarkButton();
}

function renderBookmarkButton() {
  const key = `${state.activeWorkId}:${state.activeSectionIndex}`;
  const marked = state.bookmarks.some((bookmark) => bookmark.key === key);
  el.bookmarkButton.textContent = marked ? "Remove bookmark" : "Bookmark section";
}

function normalize(text) {
  return text.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function makeSnippet(text, term) {
  const lower = text.toLowerCase();
  const index = lower.indexOf(term.toLowerCase());
  const start = Math.max(0, index - 80);
  const end = Math.min(text.length, (index < 0 ? 0 : index) + 160);
  return `${start > 0 ? "..." : ""}${text.slice(start, end)}${end < text.length ? "..." : ""}`;
}

function highlight(html, query) {
  const terms = normalize(query).split(" ").filter((term) => term.length > 1);
  if (!terms.length) return html;
  const pattern = new RegExp(`(${terms.map(escapeRegExp).join("|")})`, "gi");
  return html.replace(pattern, "<mark>$1</mark>");
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function escapeHtml(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

init().catch((error) => {
  document.body.innerHTML = `<pre>${escapeHtml(error.stack || error.message)}</pre>`;
});
