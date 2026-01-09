interface MediaMetadata {
  id: number;
  rating: string;
  score: number;
  fav_count: number;
  artists: string[];
  tags: string[];
  character_tags: string[];
  species_tags: string[];
  created_at: string;
}

interface MediaItem {
  name: string;
  path: string;
  size: number;
  media_type: "image" | "video";
  metadata?: MediaMetadata;
}

interface AdvancedFilters {
  rating?: string;
  artist?: string;
  tag?: string;
  extension?: string;
  min_score?: string;
  max_score?: string;
  min_fav?: string;
  post_id?: string;
  pool_id?: string;
  min_width?: string;
  min_height?: string;
  aspect_ratio?: string;
}

type MediaFilter = "all" | "image" | "video";
type ViewMode = "grid" | "list" | "compact";
type SortOption = "random" | "name" | "size" | "date" | "score" | "fav_count";
type SortOrder = "asc" | "desc";
type ToastType = "info" | "success" | "error";

interface AppState {
  filtered_media: MediaItem[];
  current_filter: MediaFilter;
  current_search: string;
  current_modal_idx: number;
  current_view: ViewMode;
  current_sort: SortOption;
  sort_order: SortOrder;
  advanced_filters: AdvancedFilters;
  selected_items: Set<number>;
  last_selected_index: number;
  current_page: number;
  items_per_page: number;
  is_loading: boolean;
  image_observer: IntersectionObserver | null;
}

interface DOMElements {
  gallery: HTMLElement;
  modal: HTMLDialogElement;
  modal_media: HTMLImageElement | HTMLVideoElement;
  modal_info: HTMLElement;
  modal_counter: HTMLElement;
  search_box: HTMLInputElement;
  search_clear: HTMLButtonElement;
  pagination: HTMLElement;
  stats: HTMLElement;
  bulk_actions: HTMLElement;
  selected_count: HTMLElement;
  scroll_top: HTMLButtonElement;
  adv_filters: HTMLElement;
  adv_filter_toggle: HTMLButtonElement;
  toast_container: HTMLElement;
  kb_help: HTMLDialogElement;
  items_per_page: HTMLSelectElement;
  sort_by: HTMLSelectElement;
  sort_order: HTMLButtonElement;
}

interface Config {
  readonly SEARCH_DEBOUNCE_MS: number;
  readonly TOAST_DURATION_MS: number;
  readonly SCROLL_THRESHOLD: number;
  readonly LAZY_LOAD_MARGIN: string;
  readonly DEFAULT_ITEMS_PER_PAGE: number;
}

const CONFIG: Config = Object.freeze({
  SEARCH_DEBOUNCE_MS: 300,
  TOAST_DURATION_MS: 1000,
  SCROLL_THRESHOLD: 300,
  LAZY_LOAD_MARGIN: "100px",
  DEFAULT_ITEMS_PER_PAGE: 20,
});

const DOM: DOMElements = {
  gallery: document.getElementById("gallery") as HTMLElement,
  modal: document.getElementById("modal") as HTMLDialogElement,
  modal_media: document.getElementById("modalMedia") as HTMLImageElement,
  modal_info: document.getElementById("modalInfo") as HTMLElement,
  modal_counter: document.getElementById("modalCounter") as HTMLElement,
  search_box: document.getElementById("searchBox") as HTMLInputElement,
  search_clear: document.getElementById("searchClear") as HTMLButtonElement,
  pagination: document.getElementById("pagination") as HTMLElement,
  stats: document.getElementById("stats") as HTMLElement,
  bulk_actions: document.getElementById("bulkActions") as HTMLElement,
  selected_count: document.getElementById("selectedCount") as HTMLElement,
  scroll_top: document.getElementById("scrollTop") as HTMLButtonElement,
  adv_filters: document.getElementById("advancedFilters") as HTMLElement,
  adv_filter_toggle: document.getElementById(
    "advancedFilterToggle",
  ) as HTMLButtonElement,
  toast_container: document.getElementById("toastContainer") as HTMLElement,
  kb_help: document.getElementById("keyboardHelp") as HTMLDialogElement,
  items_per_page: document.getElementById("itemsPerPage") as HTMLSelectElement,
  sort_by: document.getElementById("sortBy") as HTMLSelectElement,
  sort_order: document.getElementById("sortOrder") as HTMLButtonElement,
};

const state: AppState = {
  filtered_media: [],
  current_filter: "all",
  current_search: "",
  current_modal_idx: -1,
  current_view: "grid",
  current_sort: "name",
  sort_order: "asc",
  advanced_filters: {},
  selected_items: new Set<number>(),
  last_selected_index: -1,
  current_page: 1,
  items_per_page: CONFIG.DEFAULT_ITEMS_PER_PAGE,
  is_loading: false,
  image_observer: null,
};

function debounce<T extends (...args: unknown[]) => void>(
  fn: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let timeout_id: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>): void => {
    clearTimeout(timeout_id);
    timeout_id = setTimeout(() => fn(...args), delay);
  };
}

function format_fsize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"] as const;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`;
}

function escape_html(str: string): string {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

function show_toast(
  message: string,
  type: ToastType = "info",
  duration: number = CONFIG.TOAST_DURATION_MS,
): void {
  const toast = document.createElement("div");
  toast.className = `toast ${type}`;
  toast.setAttribute("role", "alert");

  const icons: Record<ToastType, string> = {
    success: "✓",
    error: "✗",
    info: "ℹ",
  };

  toast.innerHTML = `
    <span class="toast-icon" aria-hidden="true">${icons[type]}</span>
    <div class="toast-content">${escape_html(message)}</div>
    <button class="toast-close" aria-label="Dismiss notification">×</button>
  `;

  const close_btn = toast.querySelector(".toast-close");
  close_btn?.addEventListener("click", () => remove_toast(toast));

  DOM.toast_container.appendChild(toast);

  const timeoutId = setTimeout(() => remove_toast(toast), duration);
  toast.dataset.timeoutId = String(timeoutId);
}

function remove_toast(toast: HTMLElement): void {
  if (!toast.parentNode) return;
  clearTimeout(Number(toast.dataset.timeoutId));
  toast.style.animation = "slideInRight 0.3s ease reverse";
  toast.addEventListener("animationend", () => toast.remove(), { once: true });
}

function setup_lazy_loading(): void {
  if (state.image_observer) {
    state.image_observer.disconnect();
  }

  state.image_observer = new IntersectionObserver(
    (entries: IntersectionObserverEntry[]) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          const img = entry.target as HTMLImageElement;
          if (img.dataset.src) {
            img.src = img.dataset.src;
            img.removeAttribute("data-src");
            img.classList.remove("loading");
            img.addEventListener("load", () => img.classList.add("loaded"), {
              once: true,
            });
            img.addEventListener(
              "error",
              () => {
                img.classList.remove("loading");
                img.alt = "Failed to load image";
              },
              { once: true },
            );
          }
          state.image_observer?.unobserve(img);
        }
      });
    },
    { rootMargin: CONFIG.LAZY_LOAD_MARGIN },
  );

  document
    .querySelectorAll<HTMLImageElement>("img[data-src]")
    .forEach((img) => {
      state.image_observer?.observe(img);
    });
}

async function load_media(): Promise<void> {
  if (state.is_loading) return;
  state.is_loading = true;

  try {
    const params = new URLSearchParams();

    if (state.current_search) params.append("search", state.current_search);
    if (state.current_filter !== "all") {
      params.append("media_type", state.current_filter);
    }

    const filterMappings: Record<keyof AdvancedFilters, string> = {
      rating: "rating",
      artist: "artist",
      tag: "tag",
      extension: "extension",
      min_score: "min_score",
      max_score: "max_score",
      min_fav: "min_fav",
      post_id: "post_id",
      pool_id: "pool_id",
      min_width: "min_width",
      min_height: "min_height",
      aspect_ratio: "aspect_ratio",
    };

    for (const [key, param] of Object.entries(filterMappings)) {
      const filterKey = key as keyof AdvancedFilters;
      if (state.advanced_filters[filterKey]) {
        params.append(param, state.advanced_filters[filterKey]!);
      }
    }

    const response = await fetch(`/api/media?${params}`);

    if (!response.ok) {
      throw new Error(`Server error: ${response.status}`);
    }

    state.filtered_media = (await response.json()) as MediaItem[];
    apply_sorting();
    render_gallery();
    update_stats();
    render_pagination_controls();
  } catch (error) {
    console.error("Error loading media:", error);
    const error_msg = error instanceof Error ? error.message : "Unknown error";
    DOM.gallery.innerHTML = `
      <div class="empty">
        <h2>Error loading downloads</h2>
        <p>${escape_html(error_msg)}</p>
        <button class="btn" id="retryLoad">Retry</button>
      </div>
    `;
    document.getElementById("retryLoad")?.addEventListener("click", load_media);
    show_toast("Failed to load media", "error");
  } finally {
    state.is_loading = false;
  }
}

function apply_sorting(): void {
  if (state.current_sort === "random") {
    for (let i = state.filtered_media.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [state.filtered_media[i], state.filtered_media[j]] = [
        state.filtered_media[j],
        state.filtered_media[i],
      ];
    }
    return;
  }

  const sort_funcs: Record<
    Exclude<SortOption, "random">,
    (a: MediaItem, b: MediaItem) => number
  > = {
    name: (a, b) => a.name.localeCompare(b.name),
    size: (a, b) => a.size - b.size,
    date: (a, b) =>
      (a.metadata?.created_at || "").localeCompare(
        b.metadata?.created_at || "",
      ),
    score: (a, b) => (a.metadata?.score || 0) - (b.metadata?.score || 0),
    fav_count: (a, b) =>
      (a.metadata?.fav_count || 0) - (b.metadata?.fav_count || 0),
  };

  const sort_func =
    sort_funcs[state.current_sort as Exclude<SortOption, "random">];
  if (sort_func) {
    state.filtered_media.sort((a, b) => {
      const result = sort_func(a, b);
      return state.sort_order === "asc" ? result : -result;
    });
  }
}

function render_gallery(): void {
  DOM.gallery.className = `gallery ${state.current_view}-view`;

  if (state.filtered_media.length === 0) {
    DOM.gallery.innerHTML = `
      <div class="empty">
        <h2>No downloads found</h2>
        <p>Try adjusting your filters or search query.</p>
      </div>
    `;
    return;
  }

  const start_idx = (state.current_page - 1) * state.items_per_page;
  const end_idx = Math.min(
    start_idx + state.items_per_page,
    state.filtered_media.length,
  );
  const page_media = state.filtered_media.slice(start_idx, end_idx);

  const fragment = document.createDocumentFragment();

  page_media.forEach((item, index) => {
    const global_idx = start_idx + index;
    const is_selected = state.selected_items.has(global_idx);
    const card = create_media_card(item, global_idx, is_selected);
    fragment.appendChild(card);
  });

  DOM.gallery.innerHTML = "";
  DOM.gallery.appendChild(fragment);

  setup_lazy_loading();
  setup_video_hover();
}

function create_media_card(
  item: MediaItem,
  global_idx: number,
  is_selected: boolean,
): HTMLDivElement {
  const card = document.createElement("div");
  card.className = `media-card${is_selected ? " selected" : ""}`;
  card.dataset.index = String(global_idx);
  card.setAttribute("role", "gridcell");
  card.setAttribute("tabindex", "0");

  const meta = item.metadata;
  const ratingClass = meta ? `rating-${meta.rating}` : "";

  card.innerHTML = `
    <div class="media-preview-container">
      ${
        item.media_type === "video"
          ? `<video class="media-preview" src="${escape_html(item.path)}" preload="metadata" loop muted></video>`
          : `<img class="media-preview loading" data-src="${escape_html(item.path)}" alt="${escape_html(item.name)}">`
      }
      <div class="media-overlay">
        <div class="quick-actions">
          <button class="quick-action select-btn" title="Select" aria-label="Select item">☑</button>
          <button class="quick-action download-btn" title="Download" aria-label="Download item">⬇</button>
        </div>
        <span class="media-type ${item.media_type}">${item.media_type}</span>
      </div>
    </div>
    <div class="media-info">
      <div class="media-name">${escape_html(item.name)}</div>
      <div class="media-meta">
        <span>${format_fsize(item.size)}</span>
      </div>
      ${
        meta
          ? `
        <div class="media-metadata">
          <div class="metadata-row">
            <span class="metadata-label">ID:</span>
            <span class="metadata-value">${meta.id}</span>
          </div>
          <div class="metadata-row">
            <span class="metadata-label">Artist:</span>
            <span class="metadata-value">${escape_html(meta.artists.join(", ") || "Unknown")}</span>
          </div>
          <div class="metadata-row">
            <span class="metadata-label">Rating:</span>
            <span class="rating-badge ${ratingClass}">${meta.rating.toUpperCase()}</span>
          </div>
          <div class="metadata-row">
            <span class="metadata-label">Score:</span>
            <span class="metadata-value">${meta.score}</span>
          </div>
          ${
            meta.artists.length > 0
              ? `
            <div class="tag-list">
              ${meta.artists
                .slice(0, 3)
                .map(
                  (a) =>
                    `<button class="tag artist" data-artist="${escape_html(a)}">${escape_html(a)}</button>`,
                )
                .join("")}
            </div>
          `
              : ""
          }
        </div>
      `
          : ""
      }
    </div>
  `;

  card.addEventListener("click", (e: MouseEvent) => {
    const target = e.target as HTMLElement;

    if (target.closest(".select-btn")) {
      e.stopPropagation();
      toggle_select_with_shift(global_idx, e);
      return;
    }

    if (target.closest(".download-btn")) {
      e.stopPropagation();
      download_media(global_idx);
      return;
    }

    if (target.closest(".tag.artist")) {
      e.stopPropagation();
      const artist = (target as HTMLElement).dataset.artist;
      if (artist) filter_by_tag(artist, "artist");
      return;
    }

    open_modal(global_idx);
  });

  card.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      open_modal(global_idx);
    }
  });

  return card;
}

function setup_video_hover(): void {
  if (state.current_view === "grid" || state.current_view === "compact") {
    document
      .querySelectorAll<HTMLVideoElement>(".media-card video")
      .forEach((video) => {
        const card = video.closest(".media-card");
        card?.addEventListener("mouseenter", () => {
          video.play().catch(() => {});
        });
        card?.addEventListener("mouseleave", () => {
          video.pause();
          video.currentTime = 0;
        });
      });
  }
}

function toggle_select_with_shift(index: number, event?: MouseEvent): void {
  if (event?.shiftKey && state.last_selected_index !== -1) {
    const start = Math.min(state.last_selected_index, index);
    const end = Math.max(state.last_selected_index, index);

    for (let i = start; i <= end; i++) {
      state.selected_items.add(i);
      document.querySelector(`[data-index="${i}"]`)?.classList.add("selected");
    }
    show_toast(`Selected ${end - start + 1} items`, "success", 2000);
  } else {
    toggle_select(index);
  }
  state.last_selected_index = index;
}

function toggle_select(index: number): void {
  if (state.selected_items.has(index)) {
    state.selected_items.delete(index);
  } else {
    state.selected_items.add(index);
  }

  document
    .querySelector(`[data-index="${index}"]`)
    ?.classList.toggle("selected");
  update_bulk_actions();
}

function select_all_visible(): void {
  const start_idx = (state.current_page - 1) * state.items_per_page;
  const end_idx = Math.min(
    start_idx + state.items_per_page,
    state.filtered_media.length,
  );

  for (let i = start_idx; i < end_idx; i++) {
    state.selected_items.add(i);
    document.querySelector(`[data-index="${i}"]`)?.classList.add("selected");
  }

  update_bulk_actions();
  show_toast(`Selected ${end_idx - start_idx} items`, "success", 2000);
}

function clear_sel(): void {
  const count = state.selected_items.size;
  state.selected_items.clear();
  state.last_selected_index = -1;
  document.querySelectorAll(".media-card.selected").forEach((card) => {
    card.classList.remove("selected");
  });
  update_bulk_actions();
  update_stats();
  if (count > 0) {
    show_toast(`Cleared ${count} selections`, "info", 2000);
  }
}

function update_bulk_actions(): void {
  DOM.selected_count.textContent = `${state.selected_items.size} selected`;
  DOM.bulk_actions.hidden = state.selected_items.size === 0;
}

function download_media(index: number): void {
  const item = state.filtered_media[index];
  if (!item) return;

  const link = document.createElement("a");
  link.href = item.path;
  link.download = item.name;
  link.style.display = "none";
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  show_toast(`Downloading ${item.name}`, "success", 2000);
}

function download_selected(): void {
  let count = 0;
  state.selected_items.forEach((index) => {
    download_media(index);
    count++;
  });
  show_toast(`Downloading ${count} items`, "success", 3000);
}

function filter_by_tag(tag: string, type: "artist" | "tag"): void {
  const input = document.getElementById(
    type === "artist" ? "artistFilter" : "tagFilter",
  ) as HTMLInputElement | null;
  if (input) {
    input.value = tag;
  }
  DOM.adv_filters.hidden = false;
  DOM.adv_filter_toggle.setAttribute("aria-expanded", "true");
  apply_adv_filters();
  show_toast(`Filtering by ${type}: ${tag}`, "info", 2000);
}

function apply_adv_filters(): void {
  const get_input_val = (id: string): string => {
    const el = document.getElementById(id) as
      | HTMLInputElement
      | HTMLSelectElement
      | null;
    return el?.value || "";
  };

  state.advanced_filters = {
    rating: get_input_val("ratingFilter"),
    artist: get_input_val("artistFilter"),
    tag: get_input_val("tagFilter"),
    extension: get_input_val("extensionFilter"),
    min_score: get_input_val("minScoreFilter"),
    max_score: get_input_val("maxScoreFilter"),
    min_fav: get_input_val("minFavFilter"),
    post_id: get_input_val("postIdFilter"),
    pool_id: get_input_val("poolIdFilter"),
    min_width: get_input_val("minWidthFilter"),
    min_height: get_input_val("minHeightFilter"),
    aspect_ratio: get_input_val("aspectRatioFilter"),
  };
  state.current_page = 1;
  load_media();
  show_toast("Filters applied", "success", 2000);
}

function clear_all_filters(): void {
  state.current_search = "";
  state.current_filter = "all";
  state.advanced_filters = {};
  state.current_page = 1;
  clear_sel();

  DOM.search_box.value = "";
  DOM.search_clear.hidden = true;

  const filter_ids = [
    "ratingFilter",
    "artistFilter",
    "tagFilter",
    "extensionFilter",
    "minScoreFilter",
    "maxScoreFilter",
    "minFavFilter",
    "postIdFilter",
    "poolIdFilter",
    "minWidthFilter",
    "minHeightFilter",
    "aspectRatioFilter",
  ];
  filter_ids.forEach((id) => {
    const el = document.getElementById(id) as
      | HTMLInputElement
      | HTMLSelectElement
      | null;
    if (el) el.value = "";
  });

  document
    .querySelectorAll<HTMLButtonElement>(".filter-btn[data-filter]")
    .forEach((b) => {
      b.classList.toggle("active", b.dataset.filter === "all");
      b.setAttribute("aria-pressed", String(b.dataset.filter === "all"));
    });

  load_media();
  show_toast("All filters cleared", "success", 2000);
}

function update_stats(): void {
  const stats_info = DOM.stats.querySelector(".stats-info");
  if (!stats_info) return;

  const images = state.filtered_media.filter(
    (m) => m.media_type === "image",
  ).length;
  const videos = state.filtered_media.filter(
    (m) => m.media_type === "video",
  ).length;
  const total_size = state.filtered_media.reduce(
    (sum, item) => sum + item.size,
    0,
  );

  const filter_info: string[] = [];
  if (state.current_search)
    filter_info.push(`search: "${state.current_search}"`);
  if (state.advanced_filters.rating)
    filter_info.push(`rating: ${state.advanced_filters.rating}`);
  if (state.advanced_filters.artist)
    filter_info.push(`artist: ${state.advanced_filters.artist}`);
  if (state.advanced_filters.tag)
    filter_info.push(`tag: ${state.advanced_filters.tag}`);

  const filter_text =
    filter_info.length > 0 ? ` (${filter_info.join(", ")})` : "";

  stats_info.innerHTML = `
    <div class="stat-item"><strong>${state.filtered_media.length}</strong> items</div>
    <div class="stat-item"><strong>${images}</strong> images</div>
    <div class="stat-item"><strong>${videos}</strong> videos</div>
    <div class="stat-item"><strong>${format_fsize(total_size)}</strong> total</div>
    ${filter_text ? `<div class="stat-item" style="color: var(--iris);">${escape_html(filter_text)}</div>` : ""}
    ${state.selected_items.size > 0 ? `<div class="stat-item selection-count">${state.selected_items.size} selected</div>` : ""}
  `;
}

function render_pagination_controls(): void {
  const total_items = state.filtered_media.length;
  const total_pages = Math.ceil(total_items / state.items_per_page);

  if (total_pages <= 1) {
    DOM.pagination.innerHTML = "";
    return;
  }

  const show_pages = 5;
  let start_page = Math.max(1, state.current_page - Math.floor(show_pages / 2));
  let end_page = Math.min(total_pages, start_page + show_pages - 1);

  if (end_page - start_page < show_pages - 1) {
    start_page = Math.max(1, end_page - show_pages + 1);
  }

  let html = `
    <button class="btn page-button" data-action="prev" ${state.current_page === 1 ? "disabled" : ""}>« Prev</button>
  `;

  if (start_page > 1) {
    html += `<button class="btn page-button" data-page="1">1</button>`;
    if (start_page > 2) html += `<span class="page-ellipsis">...</span>`;
  }

  for (let i = start_page; i <= end_page; i++) {
    html += `<button class="btn page-button${i === state.current_page ? " active" : ""}" data-page="${i}" ${i === state.current_page ? 'aria-current="page"' : ""}>${i}</button>`;
  }

  if (end_page < total_pages) {
    if (end_page < total_pages - 1)
      html += `<span class="page-ellipsis">...</span>`;
    html += `<button class="btn page-button" data-page="${total_pages}">${total_pages}</button>`;
  }

  html += `
    <button class="btn page-button" data-action="next" ${state.current_page === total_pages ? "disabled" : ""}>Next »</button>
    <div class="page-jump">
      <label for="pageJumpInput" class="visually-hidden">Jump to page</label>
      <input type="number" id="pageJumpInput" class="input-base" min="1" max="${total_pages}" value="${state.current_page}" inputmode="numeric">
      <button class="btn page-button" data-action="jump">Go</button>
    </div>
  `;

  DOM.pagination.innerHTML = html;
}

function goto_page(page: number): void {
  const totalPages = Math.ceil(
    state.filtered_media.length / state.items_per_page,
  );
  if (page >= 1 && page <= totalPages && page !== state.current_page) {
    state.current_page = page;
    render_gallery();
    render_pagination_controls();
    window.scrollTo({ top: 0, behavior: "smooth" });
  }
}

function open_modal(index: number): void {
  state.current_modal_idx = index;
  show_modal_media();
  DOM.modal.showModal();
  document.body.style.overflow = "hidden";
}

function close_modal(): void {
  DOM.modal.close();
  DOM.modal_info.hidden = true;
  document
    .getElementById("modalMetadata")
    ?.setAttribute("aria-pressed", "false");
  document.body.style.overflow = "";
}

function show_modal_media(): void {
  const item = state.filtered_media[state.current_modal_idx];
  if (!item) return;

  DOM.modal_counter.textContent = `${state.current_modal_idx + 1} / ${state.filtered_media.length}`;

  const isVideo = item.media_type === "video";
  let newMedia: HTMLVideoElement | HTMLImageElement;

  if (isVideo) {
    newMedia = document.createElement("video");
    newMedia.src = item.path;
    newMedia.controls = true;
    newMedia.autoplay = true;
    newMedia.loop = true;
  } else {
    newMedia = document.createElement("img");
    newMedia.src = item.path;
    newMedia.alt = item.name;
  }

  newMedia.className = "modal-media";
  newMedia.id = "modalMedia";

  DOM.modal_media.replaceWith(newMedia);
  DOM.modal_media = newMedia;

  const meta = item.metadata;
  if (meta) {
    DOM.modal_info.innerHTML = `
      <h3>${escape_html(item.name)}</h3>
      <p><strong>ID:</strong> ${meta.id} | <strong>Rating:</strong> ${meta.rating.toUpperCase()} | <strong>Score:</strong> ${meta.score} | <strong>Favorites:</strong> ${meta.fav_count}</p>
      ${meta.artists.length > 0 ? `<p><strong>Artists:</strong> ${escape_html(meta.artists.join(", "))}</p>` : ""}
      ${meta.tags.length > 0 ? `<p><strong>Tags:</strong> ${escape_html(meta.tags.slice(0, 20).join(", "))}${meta.tags.length > 20 ? "..." : ""}</p>` : ""}
      ${meta.character_tags.length > 0 ? `<p><strong>Characters:</strong> ${escape_html(meta.character_tags.join(", "))}</p>` : ""}
      ${meta.species_tags.length > 0 ? `<p><strong>Species:</strong> ${escape_html(meta.species_tags.join(", "))}</p>` : ""}
      <p><strong>Created:</strong> ${new Date(meta.created_at).toLocaleString()}</p>
      <p><strong>Size:</strong> ${format_fsize(item.size)}</p>
    `;
  } else {
    DOM.modal_info.innerHTML = `
      <h3>${escape_html(item.name)}</h3>
      <p><strong>Size:</strong> ${format_fsize(item.size)}</p>
      <p>No metadata available</p>
    `;
  }
}

function nav_modal(direction: 1 | -1): void {
  const length = state.filtered_media.length;
  state.current_modal_idx =
    (state.current_modal_idx + direction + length) % length;
  show_modal_media();
}

function toggle_modal_info(): void {
  const isHidden = DOM.modal_info.hidden;
  DOM.modal_info.hidden = !isHidden;
  document
    .getElementById("modalMetadata")
    ?.setAttribute("aria-pressed", String(isHidden));
}

async function toggle_fullscreen(): Promise<void> {
  const modalContent = document.querySelector(".modal-content");
  if (!modalContent) return;

  try {
    if (!document.fullscreenElement) {
      await modalContent.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  } catch (err) {
    console.error("Fullscreen error:", err);
    show_toast("Fullscreen not supported", "error", 2000);
  }
}

function open_kbhelp(): void {
  DOM.kb_help.showModal();
}

function close_kbhelp(): void {
  DOM.kb_help.close();
}

function setup_evlisteners(): void {
  const debouncedSearch = debounce(() => {
    state.current_search = DOM.search_box.value;
    state.current_page = 1;
    load_media();
  }, CONFIG.SEARCH_DEBOUNCE_MS);

  DOM.search_box.addEventListener("input", (e: Event) => {
    const target = e.target as HTMLInputElement;
    DOM.search_clear.hidden = target.value.length === 0;
    debouncedSearch();
  });

  DOM.search_clear.addEventListener("click", () => {
    DOM.search_box.value = "";
    DOM.search_clear.hidden = true;
    state.current_search = "";
    state.current_page = 1;
    load_media();
  });

  document
    .querySelectorAll<HTMLButtonElement>(".filter-btn[data-filter]")
    .forEach((btn) => {
      btn.addEventListener("click", () => {
        document
          .querySelectorAll<HTMLButtonElement>(".filter-btn[data-filter]")
          .forEach((b) => {
            b.classList.remove("active");
            b.setAttribute("aria-pressed", "false");
          });
        btn.classList.add("active");
        btn.setAttribute("aria-pressed", "true");
        state.current_filter = btn.dataset.filter as MediaFilter;
        state.current_page = 1;
        load_media();
      });
    });

  document.querySelectorAll<HTMLButtonElement>(".view-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll<HTMLButtonElement>(".view-btn").forEach((b) => {
        b.classList.remove("active");
        b.setAttribute("aria-pressed", "false");
      });
      btn.classList.add("active");
      btn.setAttribute("aria-pressed", "true");
      state.current_view = btn.dataset.view as ViewMode;
      render_gallery();
      show_toast(`View changed to ${state.current_view}`, "info", 1500);
    });
  });

  DOM.sort_by.addEventListener("change", (e: Event) => {
    const target = e.target as HTMLSelectElement;
    state.current_sort = target.value as SortOption;
    apply_sorting();
    render_gallery();
    render_pagination_controls();
  });

  DOM.sort_order.addEventListener("click", () => {
    state.sort_order = state.sort_order === "asc" ? "desc" : "asc";
    DOM.sort_order.textContent = state.sort_order === "asc" ? "↑" : "↓";
    DOM.sort_order.setAttribute(
      "aria-label",
      `Sort ${state.sort_order === "asc" ? "ascending" : "descending"}, click to toggle`,
    );
    DOM.sort_order.dataset.order = state.sort_order;
    apply_sorting();
    render_gallery();
    render_pagination_controls();
  });

  DOM.items_per_page.addEventListener("change", (e: Event) => {
    const target = e.target as HTMLSelectElement;
    state.items_per_page = parseInt(target.value, 10);
    state.current_page = 1;
    render_gallery();
    render_pagination_controls();
    show_toast(`Showing ${state.items_per_page} items per page`, "info", 2000);
  });

  DOM.adv_filter_toggle.addEventListener("click", () => {
    const isHidden = DOM.adv_filters.hidden;
    DOM.adv_filters.hidden = !isHidden;
    DOM.adv_filter_toggle.setAttribute("aria-expanded", String(isHidden));
  });

  document
    .getElementById("applyFilters")
    ?.addEventListener("click", apply_adv_filters);
  document
    .getElementById("clearFilters")
    ?.addEventListener("click", clear_all_filters);

  document
    .getElementById("downloadSelected")
    ?.addEventListener("click", download_selected);
  document
    .getElementById("clearSelection")
    ?.addEventListener("click", clear_sel);

  DOM.pagination.addEventListener("click", (e: MouseEvent) => {
    const btn = (e.target as HTMLElement).closest("button");
    if (!btn) return;

    const page = btn.dataset.page;
    const action = btn.dataset.action;

    if (page) {
      goto_page(parseInt(page, 10));
    } else if (action === "prev") {
      goto_page(state.current_page - 1);
    } else if (action === "next") {
      goto_page(state.current_page + 1);
    } else if (action === "jump") {
      const input = document.getElementById(
        "pageJumpInput",
      ) as HTMLInputElement | null;
      if (input) goto_page(parseInt(input.value, 10));
    }
  });

  DOM.pagination.addEventListener("keydown", (e: KeyboardEvent) => {
    const target = e.target as HTMLElement;
    if (e.key === "Enter" && target.id === "pageJumpInput") {
      goto_page(parseInt((target as HTMLInputElement).value, 10));
    }
  });

  window.addEventListener(
    "scroll",
    () => {
      DOM.scroll_top.hidden = window.pageYOffset <= CONFIG.SCROLL_THRESHOLD;
    },
    { passive: true },
  );

  DOM.scroll_top.addEventListener("click", () => {
    window.scrollTo({ top: 0, behavior: "smooth" });
  });

  document.getElementById("modalClose")?.addEventListener("click", close_modal);
  document
    .getElementById("modalNext")
    ?.addEventListener("click", () => nav_modal(1));
  document
    .getElementById("modalPrev")
    ?.addEventListener("click", () => nav_modal(-1));
  document
    .getElementById("modalDownload")
    ?.addEventListener("click", () => download_media(state.current_modal_idx));
  document
    .getElementById("modalFullscreen")
    ?.addEventListener("click", toggle_fullscreen);
  document
    .getElementById("modalMetadata")
    ?.addEventListener("click", toggle_modal_info);

  DOM.modal.addEventListener("click", (e: MouseEvent) => {
    if (e.target === DOM.modal) close_modal();
  });

  DOM.modal.addEventListener("cancel", (e: Event) => {
    e.preventDefault();
    close_modal();
  });

  document
    .getElementById("keyboardHintBtn")
    ?.addEventListener("click", open_kbhelp);
  document
    .getElementById("closeKeyboardHelp")
    ?.addEventListener("click", close_kbhelp);

  DOM.kb_help.addEventListener("click", (e: MouseEvent) => {
    if (e.target === DOM.kb_help) close_kbhelp();
  });

  document.addEventListener("fullscreenchange", () => {
    const btn = document.getElementById("modalFullscreen");
    if (btn) {
      btn.title = document.fullscreenElement
        ? "Exit Fullscreen"
        : "Toggle fullscreen";
    }
  });

  document.addEventListener("keydown", handle_key_down);
}

function handle_key_down(e: KeyboardEvent): void {
  if (DOM.modal.open) {
    switch (e.key) {
      case "Escape":
        close_modal();
        return;
      case "ArrowRight":
      case "d":
      case "D":
        nav_modal(1);
        return;
      case "ArrowLeft":
      case "A":
      case "a":
        nav_modal(-1);
        return;
      case "d":
      case "D":
        download_media(state.current_modal_idx);
        return;
      case "i":
      case "I":
        toggle_modal_info();
        return;
      case "f":
      case "F":
        toggle_fullscreen();
        return;
    }
    return;
  }

  if (DOM.kb_help.open) {
    if (e.key === "Escape") close_kbhelp();
    return;
  }

  const target = e.target as HTMLElement;
  if (target.tagName === "INPUT" || target.tagName === "SELECT") {
    return;
  }

  switch (e.key) {
    case "n":
    case "N":
      goto_page(state.current_page + 1);
      break;
    case "p":
    case "P":
      goto_page(state.current_page - 1);
      break;
    case "?":
      open_kbhelp();
      break;
    case "/":
      e.preventDefault();
      DOM.search_box.focus();
      break;
    case "Escape":
      if (state.selected_items.size > 0) {
        clear_sel();
      }
      break;
    case "a":
    case "A":
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        select_all_visible();
      }
      break;
  }
}

function init(): void {
  state.items_per_page = parseInt(DOM.items_per_page.value, 10);
  setup_evlisteners();
  load_media();
  show_toast("Press ? for keyboard shortcuts", "info", 4000);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
