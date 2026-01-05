var __awaiter =
  (this && this.__awaiter) ||
  function (thisArg, _arguments, P, generator) {
    function adopt(value) {
      return value instanceof P
        ? value
        : new P(function (resolve) {
            resolve(value);
          });
    }
    return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) {
        try {
          step(generator.next(value));
        } catch (e) {
          reject(e);
        }
      }
      function rejected(value) {
        try {
          step(generator["throw"](value));
        } catch (e) {
          reject(e);
        }
      }
      function step(result) {
        result.done
          ? resolve(result.value)
          : adopt(result.value).then(fulfilled, rejected);
      }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
  };
var __generator =
  (this && this.__generator) ||
  function (thisArg, body) {
    var _ = {
        label: 0,
        sent: function () {
          if (t[0] & 1) throw t[1];
          return t[1];
        },
        trys: [],
        ops: [],
      },
      f,
      y,
      t,
      g = Object.create(
        (typeof Iterator === "function" ? Iterator : Object).prototype,
      );
    return (
      (g.next = verb(0)),
      (g["throw"] = verb(1)),
      (g["return"] = verb(2)),
      typeof Symbol === "function" &&
        (g[Symbol.iterator] = function () {
          return this;
        }),
      g
    );
    function verb(n) {
      return function (v) {
        return step([n, v]);
      };
    }
    function step(op) {
      if (f) throw new TypeError("Generator is already executing.");
      while ((g && ((g = 0), op[0] && (_ = 0)), _))
        try {
          if (
            ((f = 1),
            y &&
              (t =
                op[0] & 2
                  ? y["return"]
                  : op[0]
                    ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                    : y.next) &&
              !(t = t.call(y, op[1])).done)
          )
            return t;
          if (((y = 0), t)) op = [op[0] & 2, t.value];
          switch (op[0]) {
            case 0:
            case 1:
              t = op;
              break;
            case 4:
              _.label++;
              return { value: op[1], done: false };
            case 5:
              _.label++;
              y = op[1];
              op = [0];
              continue;
            case 7:
              op = _.ops.pop();
              _.trys.pop();
              continue;
            default:
              if (
                !((t = _.trys), (t = t.length > 0 && t[t.length - 1])) &&
                (op[0] === 6 || op[0] === 2)
              ) {
                _ = 0;
                continue;
              }
              if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                _.label = op[1];
                break;
              }
              if (op[0] === 6 && _.label < t[1]) {
                _.label = t[1];
                t = op;
                break;
              }
              if (t && _.label < t[2]) {
                _.label = t[2];
                _.ops.push(op);
                break;
              }
              if (t[2]) _.ops.pop();
              _.trys.pop();
              continue;
          }
          op = body.call(thisArg, _);
        } catch (e) {
          op = [6, e];
          y = 0;
        } finally {
          f = t = 0;
        }
      if (op[0] & 5) throw op[1];
      return { value: op[0] ? op[1] : void 0, done: true };
    }
  };
var CONFIG = Object.freeze({
  SEARCH_DEBOUNCE_MS: 300,
  TOAST_DURATION_MS: 1000,
  SCROLL_THRESHOLD: 300,
  LAZY_LOAD_MARGIN: "100px",
  DEFAULT_ITEMS_PER_PAGE: 20,
});
var DOM = {
  gallery: document.getElementById("gallery"),
  modal: document.getElementById("modal"),
  modal_media: document.getElementById("modalMedia"),
  modal_info: document.getElementById("modalInfo"),
  modal_counter: document.getElementById("modalCounter"),
  search_box: document.getElementById("searchBox"),
  search_clear: document.getElementById("searchClear"),
  pagination: document.getElementById("pagination"),
  stats: document.getElementById("stats"),
  bulk_actions: document.getElementById("bulkActions"),
  selected_count: document.getElementById("selectedCount"),
  scroll_top: document.getElementById("scrollTop"),
  adv_filters: document.getElementById("advancedFilters"),
  adv_filter_toggle: document.getElementById("advancedFilterToggle"),
  toast_container: document.getElementById("toastContainer"),
  kb_help: document.getElementById("keyboardHelp"),
  items_per_page: document.getElementById("itemsPerPage"),
  sort_by: document.getElementById("sortBy"),
  sort_order: document.getElementById("sortOrder"),
};
var state = {
  filtered_media: [],
  current_filter: "all",
  current_search: "",
  current_modal_idx: -1,
  current_view: "grid",
  current_sort: "name",
  sort_order: "asc",
  advanced_filters: {},
  selected_items: new Set(),
  last_selected_index: -1,
  current_page: 1,
  items_per_page: CONFIG.DEFAULT_ITEMS_PER_PAGE,
  is_loading: false,
  image_observer: null,
};
function debounce(fn, delay) {
  var timeout_id;
  return function () {
    var args = [];
    for (var _i = 0; _i < arguments.length; _i++) {
      args[_i] = arguments[_i];
    }
    clearTimeout(timeout_id);
    timeout_id = setTimeout(function () {
      return fn.apply(void 0, args);
    }, delay);
  };
}
function format_fsize(bytes) {
  if (bytes === 0) return "0 B";
  var k = 1024;
  var sizes = ["B", "KB", "MB", "GB"];
  var i = Math.floor(Math.log(bytes) / Math.log(k));
  return ""
    .concat(Math.round((bytes / Math.pow(k, i)) * 100) / 100, " ")
    .concat(sizes[i]);
}
function escape_html(str) {
  var div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}
function show_toast(message, type, duration) {
  if (type === void 0) {
    type = "info";
  }
  if (duration === void 0) {
    duration = CONFIG.TOAST_DURATION_MS;
  }
  var toast = document.createElement("div");
  toast.className = "toast ".concat(type);
  toast.setAttribute("role", "alert");
  var icons = {
    success: "✓",
    error: "✗",
    info: "ℹ",
  };
  toast.innerHTML = '\n    <span class="toast-icon" aria-hidden="true">'
    .concat(icons[type], '</span>\n    <div class="toast-content">')
    .concat(
      escape_html(message),
      '</div>\n    <button class="toast-close" aria-label="Dismiss notification">\u00D7</button>\n  ',
    );
  var close_btn = toast.querySelector(".toast-close");
  close_btn === null || close_btn === void 0
    ? void 0
    : close_btn.addEventListener("click", function () {
        return remove_toast(toast);
      });
  DOM.toast_container.appendChild(toast);
  var timeoutId = setTimeout(function () {
    return remove_toast(toast);
  }, duration);
  toast.dataset.timeoutId = String(timeoutId);
}
function remove_toast(toast) {
  if (!toast.parentNode) return;
  clearTimeout(Number(toast.dataset.timeoutId));
  toast.style.animation = "slideInRight 0.3s ease reverse";
  toast.addEventListener(
    "animationend",
    function () {
      return toast.remove();
    },
    { once: true },
  );
}
function setup_lazy_loading() {
  if (state.image_observer) {
    state.image_observer.disconnect();
  }
  state.image_observer = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (entry) {
        var _a;
        if (entry.isIntersecting) {
          var img_1 = entry.target;
          if (img_1.dataset.src) {
            img_1.src = img_1.dataset.src;
            img_1.removeAttribute("data-src");
            img_1.classList.remove("loading");
            img_1.addEventListener(
              "load",
              function () {
                return img_1.classList.add("loaded");
              },
              {
                once: true,
              },
            );
            img_1.addEventListener(
              "error",
              function () {
                img_1.classList.remove("loading");
                img_1.alt = "Failed to load image";
              },
              { once: true },
            );
          }
          (_a = state.image_observer) === null || _a === void 0
            ? void 0
            : _a.unobserve(img_1);
        }
      });
    },
    { rootMargin: CONFIG.LAZY_LOAD_MARGIN },
  );
  document.querySelectorAll("img[data-src]").forEach(function (img) {
    var _a;
    (_a = state.image_observer) === null || _a === void 0
      ? void 0
      : _a.observe(img);
  });
}
function load_media() {
  return __awaiter(this, void 0, void 0, function () {
    var params,
      filterMappings,
      _i,
      _a,
      _b,
      key,
      param,
      filterKey,
      response,
      _c,
      error_1,
      error_msg;
    var _d;
    return __generator(this, function (_e) {
      switch (_e.label) {
        case 0:
          if (state.is_loading) return [2 /*return*/];
          state.is_loading = true;
          _e.label = 1;
        case 1:
          _e.trys.push([1, 4, 5, 6]);
          params = new URLSearchParams();
          if (state.current_search)
            params.append("search", state.current_search);
          if (state.current_filter !== "all") {
            params.append("media_type", state.current_filter);
          }
          filterMappings = {
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
          for (
            _i = 0, _a = Object.entries(filterMappings);
            _i < _a.length;
            _i++
          ) {
            ((_b = _a[_i]), (key = _b[0]), (param = _b[1]));
            filterKey = key;
            if (state.advanced_filters[filterKey]) {
              params.append(param, state.advanced_filters[filterKey]);
            }
          }
          return [4 /*yield*/, fetch("/api/media?".concat(params))];
        case 2:
          response = _e.sent();
          if (!response.ok) {
            throw new Error("Server error: ".concat(response.status));
          }
          _c = state;
          return [4 /*yield*/, response.json()];
        case 3:
          _c.filtered_media = _e.sent();
          apply_sorting();
          render_gallery();
          update_stats();
          render_pagination_controls();
          return [3 /*break*/, 6];
        case 4:
          error_1 = _e.sent();
          console.error("Error loading media:", error_1);
          error_msg =
            error_1 instanceof Error ? error_1.message : "Unknown error";
          DOM.gallery.innerHTML =
            '\n      <div class="empty">\n        <h2>Error loading downloads</h2>\n        <p>'.concat(
              escape_html(error_msg),
              '</p>\n        <button class="btn" id="retryLoad">Retry</button>\n      </div>\n    ',
            );
          (_d = document.getElementById("retryLoad")) === null || _d === void 0
            ? void 0
            : _d.addEventListener("click", load_media);
          show_toast("Failed to load media", "error");
          return [3 /*break*/, 6];
        case 5:
          state.is_loading = false;
          return [7 /*endfinally*/];
        case 6:
          return [2 /*return*/];
      }
    });
  });
}
function apply_sorting() {
  var _a;
  if (state.current_sort === "random") {
    for (var i = state.filtered_media.length - 1; i > 0; i--) {
      var j = Math.floor(Math.random() * (i + 1));
      ((_a = [state.filtered_media[j], state.filtered_media[i]]),
        (state.filtered_media[i] = _a[0]),
        (state.filtered_media[j] = _a[1]));
    }
    return;
  }
  var sort_funcs = {
    name: function (a, b) {
      return a.name.localeCompare(b.name);
    },
    size: function (a, b) {
      return a.size - b.size;
    },
    date: function (a, b) {
      var _a, _b;
      return (
        ((_a = a.metadata) === null || _a === void 0
          ? void 0
          : _a.created_at) || ""
      ).localeCompare(
        ((_b = b.metadata) === null || _b === void 0
          ? void 0
          : _b.created_at) || "",
      );
    },
    score: function (a, b) {
      var _a, _b;
      return (
        (((_a = a.metadata) === null || _a === void 0 ? void 0 : _a.score) ||
          0) -
        (((_b = b.metadata) === null || _b === void 0 ? void 0 : _b.score) || 0)
      );
    },
    fav_count: function (a, b) {
      var _a, _b;
      return (
        (((_a = a.metadata) === null || _a === void 0
          ? void 0
          : _a.fav_count) || 0) -
        (((_b = b.metadata) === null || _b === void 0
          ? void 0
          : _b.fav_count) || 0)
      );
    },
  };
  var sort_func = sort_funcs[state.current_sort];
  if (sort_func) {
    state.filtered_media.sort(function (a, b) {
      var result = sort_func(a, b);
      return state.sort_order === "asc" ? result : -result;
    });
  }
}
function render_gallery() {
  DOM.gallery.className = "gallery ".concat(state.current_view, "-view");
  if (state.filtered_media.length === 0) {
    DOM.gallery.innerHTML =
      '\n      <div class="empty">\n        <h2>No downloads found</h2>\n        <p>Try adjusting your filters or search query.</p>\n      </div>\n    ';
    return;
  }
  var start_idx = (state.current_page - 1) * state.items_per_page;
  var end_idx = Math.min(
    start_idx + state.items_per_page,
    state.filtered_media.length,
  );
  var page_media = state.filtered_media.slice(start_idx, end_idx);
  var fragment = document.createDocumentFragment();
  page_media.forEach(function (item, index) {
    var global_idx = start_idx + index;
    var is_selected = state.selected_items.has(global_idx);
    var card = create_media_card(item, global_idx, is_selected);
    fragment.appendChild(card);
  });
  DOM.gallery.innerHTML = "";
  DOM.gallery.appendChild(fragment);
  setup_lazy_loading();
  setup_video_hover();
}
function create_media_card(item, global_idx, is_selected) {
  var card = document.createElement("div");
  card.className = "media-card".concat(is_selected ? " selected" : "");
  card.dataset.index = String(global_idx);
  card.setAttribute("role", "gridcell");
  card.setAttribute("tabindex", "0");
  var meta = item.metadata;
  var ratingClass = meta ? "rating-".concat(meta.rating) : "";
  card.innerHTML = '\n    <div class="media-preview-container">\n      '
    .concat(
      item.media_type === "video"
        ? '<video class="media-preview" src="'.concat(
            escape_html(item.path),
            '" preload="metadata" loop muted></video>',
          )
        : '<img class="media-preview loading" data-src="'
            .concat(escape_html(item.path), '" alt="')
            .concat(escape_html(item.name), '">'),
      '\n      <div class="media-overlay">\n        <div class="quick-actions">\n          <button class="quick-action select-btn" title="Select" aria-label="Select item">\u2611</button>\n          <button class="quick-action download-btn" title="Download" aria-label="Download item">\u2B07</button>\n        </div>\n        <span class="media-type ',
    )
    .concat(item.media_type, '">')
    .concat(
      item.media_type,
      '</span>\n      </div>\n    </div>\n    <div class="media-info">\n      <div class="media-name">',
    )
    .concat(
      escape_html(item.name),
      '</div>\n      <div class="media-meta">\n        <span>',
    )
    .concat(format_fsize(item.size), "</span>\n      </div>\n      ")
    .concat(
      meta
        ? '\n        <div class="media-metadata">\n          <div class="metadata-row">\n            <span class="metadata-label">ID:</span>\n            <span class="metadata-value">'
            .concat(
              meta.id,
              '</span>\n          </div>\n          <div class="metadata-row">\n            <span class="metadata-label">Artist:</span>\n            <span class="metadata-value">',
            )
            .concat(
              escape_html(meta.artists.join(", ") || "Unknown"),
              '</span>\n          </div>\n          <div class="metadata-row">\n            <span class="metadata-label">Rating:</span>\n            <span class="rating-badge ',
            )
            .concat(ratingClass, '">')
            .concat(
              meta.rating.toUpperCase(),
              '</span>\n          </div>\n          <div class="metadata-row">\n            <span class="metadata-label">Score:</span>\n            <span class="metadata-value">',
            )
            .concat(meta.score, "</span>\n          </div>\n          ")
            .concat(
              meta.artists.length > 0
                ? '\n            <div class="tag-list">\n              '.concat(
                    meta.artists
                      .slice(0, 3)
                      .map(function (a) {
                        return '<button class="tag artist" data-artist="'
                          .concat(escape_html(a), '">')
                          .concat(escape_html(a), "</button>");
                      })
                      .join(""),
                    "\n            </div>\n          ",
                  )
                : "",
              "\n        </div>\n      ",
            )
        : "",
      "\n    </div>\n  ",
    );
  card.addEventListener("click", function (e) {
    var target = e.target;
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
      var artist = target.dataset.artist;
      if (artist) filter_by_tag(artist, "artist");
      return;
    }
    open_modal(global_idx);
  });
  card.addEventListener("keydown", function (e) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      open_modal(global_idx);
    }
  });
  return card;
}
function setup_video_hover() {
  if (state.current_view === "grid" || state.current_view === "compact") {
    document.querySelectorAll(".media-card video").forEach(function (video) {
      var card = video.closest(".media-card");
      card === null || card === void 0
        ? void 0
        : card.addEventListener("mouseenter", function () {
            video.play().catch(function () {});
          });
      card === null || card === void 0
        ? void 0
        : card.addEventListener("mouseleave", function () {
            video.pause();
            video.currentTime = 0;
          });
    });
  }
}
function toggle_select_with_shift(index, event) {
  var _a;
  if (
    (event === null || event === void 0 ? void 0 : event.shiftKey) &&
    state.last_selected_index !== -1
  ) {
    var start = Math.min(state.last_selected_index, index);
    var end = Math.max(state.last_selected_index, index);
    for (var i = start; i <= end; i++) {
      state.selected_items.add(i);
      (_a = document.querySelector('[data-index="'.concat(i, '"]'))) === null ||
      _a === void 0
        ? void 0
        : _a.classList.add("selected");
    }
    show_toast("Selected ".concat(end - start + 1, " items"), "success", 2000);
  } else {
    toggle_select(index);
  }
  state.last_selected_index = index;
}
function toggle_select(index) {
  var _a;
  if (state.selected_items.has(index)) {
    state.selected_items.delete(index);
  } else {
    state.selected_items.add(index);
  }
  (_a = document.querySelector('[data-index="'.concat(index, '"]'))) === null ||
  _a === void 0
    ? void 0
    : _a.classList.toggle("selected");
  update_bulk_actions();
}
function select_all_visible() {
  var _a;
  var start_idx = (state.current_page - 1) * state.items_per_page;
  var end_idx = Math.min(
    start_idx + state.items_per_page,
    state.filtered_media.length,
  );
  for (var i = start_idx; i < end_idx; i++) {
    state.selected_items.add(i);
    (_a = document.querySelector('[data-index="'.concat(i, '"]'))) === null ||
    _a === void 0
      ? void 0
      : _a.classList.add("selected");
  }
  update_bulk_actions();
  show_toast(
    "Selected ".concat(end_idx - start_idx, " items"),
    "success",
    2000,
  );
}
function clear_sel() {
  var count = state.selected_items.size;
  state.selected_items.clear();
  state.last_selected_index = -1;
  document.querySelectorAll(".media-card.selected").forEach(function (card) {
    card.classList.remove("selected");
  });
  update_bulk_actions();
  update_stats();
  if (count > 0) {
    show_toast("Cleared ".concat(count, " selections"), "info", 2000);
  }
}
function update_bulk_actions() {
  DOM.selected_count.textContent = "".concat(
    state.selected_items.size,
    " selected",
  );
  DOM.bulk_actions.hidden = state.selected_items.size === 0;
}
function download_media(index) {
  var item = state.filtered_media[index];
  if (!item) return;
  var link = document.createElement("a");
  link.href = item.path;
  link.download = item.name;
  link.style.display = "none";
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  show_toast("Downloading ".concat(item.name), "success", 2000);
}
function download_selected() {
  var count = 0;
  state.selected_items.forEach(function (index) {
    download_media(index);
    count++;
  });
  show_toast("Downloading ".concat(count, " items"), "success", 3000);
}
function filter_by_tag(tag, type) {
  var input = document.getElementById(
    type === "artist" ? "artistFilter" : "tagFilter",
  );
  if (input) {
    input.value = tag;
  }
  DOM.adv_filters.hidden = false;
  DOM.adv_filter_toggle.setAttribute("aria-expanded", "true");
  apply_adv_filters();
  show_toast("Filtering by ".concat(type, ": ").concat(tag), "info", 2000);
}
function apply_adv_filters() {
  var get_input_val = function (id) {
    var el = document.getElementById(id);
    return (el === null || el === void 0 ? void 0 : el.value) || "";
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
function clear_all_filters() {
  state.current_search = "";
  state.current_filter = "all";
  state.advanced_filters = {};
  state.current_page = 1;
  clear_sel();
  DOM.search_box.value = "";
  DOM.search_clear.hidden = true;
  var filter_ids = [
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
  filter_ids.forEach(function (id) {
    var el = document.getElementById(id);
    if (el) el.value = "";
  });
  document.querySelectorAll(".filter-btn[data-filter]").forEach(function (b) {
    b.classList.toggle("active", b.dataset.filter === "all");
    b.setAttribute("aria-pressed", String(b.dataset.filter === "all"));
  });
  load_media();
  show_toast("All filters cleared", "success", 2000);
}
function update_stats() {
  var stats_info = DOM.stats.querySelector(".stats-info");
  if (!stats_info) return;
  var images = state.filtered_media.filter(function (m) {
    return m.media_type === "image";
  }).length;
  var videos = state.filtered_media.filter(function (m) {
    return m.media_type === "video";
  }).length;
  var total_size = state.filtered_media.reduce(function (sum, item) {
    return sum + item.size;
  }, 0);
  var filter_info = [];
  if (state.current_search)
    filter_info.push('search: "'.concat(state.current_search, '"'));
  if (state.advanced_filters.rating)
    filter_info.push("rating: ".concat(state.advanced_filters.rating));
  if (state.advanced_filters.artist)
    filter_info.push("artist: ".concat(state.advanced_filters.artist));
  if (state.advanced_filters.tag)
    filter_info.push("tag: ".concat(state.advanced_filters.tag));
  var filter_text =
    filter_info.length > 0 ? " (".concat(filter_info.join(", "), ")") : "";
  stats_info.innerHTML = '\n    <div class="stat-item"><strong>'
    .concat(
      state.filtered_media.length,
      '</strong> items</div>\n    <div class="stat-item"><strong>',
    )
    .concat(
      images,
      '</strong> images</div>\n    <div class="stat-item"><strong>',
    )
    .concat(
      videos,
      '</strong> videos</div>\n    <div class="stat-item"><strong>',
    )
    .concat(format_fsize(total_size), "</strong> total</div>\n    ")
    .concat(
      filter_text
        ? '<div class="stat-item" style="color: var(--iris);">'.concat(
            escape_html(filter_text),
            "</div>",
          )
        : "",
      "\n    ",
    )
    .concat(
      state.selected_items.size > 0
        ? '<div class="stat-item selection-count">'.concat(
            state.selected_items.size,
            " selected</div>",
          )
        : "",
      "\n  ",
    );
}
function render_pagination_controls() {
  var total_items = state.filtered_media.length;
  var total_pages = Math.ceil(total_items / state.items_per_page);
  if (total_pages <= 1) {
    DOM.pagination.innerHTML = "";
    return;
  }
  var show_pages = 5;
  var start_page = Math.max(1, state.current_page - Math.floor(show_pages / 2));
  var end_page = Math.min(total_pages, start_page + show_pages - 1);
  if (end_page - start_page < show_pages - 1) {
    start_page = Math.max(1, end_page - show_pages + 1);
  }
  var html = '\n    <button class="btn page-button" data-action="prev" '.concat(
    state.current_page === 1 ? "disabled" : "",
    ">\u00AB Prev</button>\n  ",
  );
  if (start_page > 1) {
    html += '<button class="btn page-button" data-page="1">1</button>';
    if (start_page > 2) html += '<span class="page-ellipsis">...</span>';
  }
  for (var i = start_page; i <= end_page; i++) {
    html += '<button class="btn page-button'
      .concat(i === state.current_page ? " active" : "", '" data-page="')
      .concat(i, '" ')
      .concat(i === state.current_page ? 'aria-current="page"' : "", ">")
      .concat(i, "</button>");
  }
  if (end_page < total_pages) {
    if (end_page < total_pages - 1)
      html += '<span class="page-ellipsis">...</span>';
    html += '<button class="btn page-button" data-page="'
      .concat(total_pages, '">')
      .concat(total_pages, "</button>");
  }
  html += '\n    <button class="btn page-button" data-action="next" '
    .concat(
      state.current_page === total_pages ? "disabled" : "",
      '>Next \u00BB</button>\n    <div class="page-jump">\n      <label for="pageJumpInput" class="visually-hidden">Jump to page</label>\n      <input type="number" id="pageJumpInput" class="input-base" min="1" max="',
    )
    .concat(total_pages, '" value="')
    .concat(
      state.current_page,
      '" inputmode="numeric">\n      <button class="btn page-button" data-action="jump">Go</button>\n    </div>\n  ',
    );
  DOM.pagination.innerHTML = html;
}
function goto_page(page) {
  var totalPages = Math.ceil(
    state.filtered_media.length / state.items_per_page,
  );
  if (page >= 1 && page <= totalPages && page !== state.current_page) {
    state.current_page = page;
    render_gallery();
    render_pagination_controls();
    window.scrollTo({ top: 0, behavior: "smooth" });
  }
}
function open_modal(index) {
  state.current_modal_idx = index;
  show_modal_media();
  DOM.modal.showModal();
  document.body.style.overflow = "hidden";
}
function close_modal() {
  var _a;
  DOM.modal.close();
  DOM.modal_info.hidden = true;
  (_a = document.getElementById("modalMetadata")) === null || _a === void 0
    ? void 0
    : _a.setAttribute("aria-pressed", "false");
  document.body.style.overflow = "";
}
function show_modal_media() {
  var item = state.filtered_media[state.current_modal_idx];
  if (!item) return;
  DOM.modal_counter.textContent = ""
    .concat(state.current_modal_idx + 1, " / ")
    .concat(state.filtered_media.length);
  var isVideo = item.media_type === "video";
  var newMedia;
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
  var meta = item.metadata;
  if (meta) {
    DOM.modal_info.innerHTML = "\n      <h3>"
      .concat(escape_html(item.name), "</h3>\n      <p><strong>ID:</strong> ")
      .concat(meta.id, " | <strong>Rating:</strong> ")
      .concat(meta.rating.toUpperCase(), " | <strong>Score:</strong> ")
      .concat(meta.score, " | <strong>Favorites:</strong> ")
      .concat(meta.fav_count, "</p>\n      ")
      .concat(
        meta.artists.length > 0
          ? "<p><strong>Artists:</strong> ".concat(
              escape_html(meta.artists.join(", ")),
              "</p>",
            )
          : "",
        "\n      ",
      )
      .concat(
        meta.tags.length > 0
          ? "<p><strong>Tags:</strong> "
              .concat(escape_html(meta.tags.slice(0, 20).join(", ")))
              .concat(meta.tags.length > 20 ? "..." : "", "</p>")
          : "",
        "\n      ",
      )
      .concat(
        meta.character_tags.length > 0
          ? "<p><strong>Characters:</strong> ".concat(
              escape_html(meta.character_tags.join(", ")),
              "</p>",
            )
          : "",
        "\n      ",
      )
      .concat(
        meta.species_tags.length > 0
          ? "<p><strong>Species:</strong> ".concat(
              escape_html(meta.species_tags.join(", ")),
              "</p>",
            )
          : "",
        "\n      <p><strong>Created:</strong> ",
      )
      .concat(
        new Date(meta.created_at).toLocaleString(),
        "</p>\n      <p><strong>Size:</strong> ",
      )
      .concat(format_fsize(item.size), "</p>\n    ");
  } else {
    DOM.modal_info.innerHTML = "\n      <h3>"
      .concat(escape_html(item.name), "</h3>\n      <p><strong>Size:</strong> ")
      .concat(
        format_fsize(item.size),
        "</p>\n      <p>No metadata available</p>\n    ",
      );
  }
}
function nav_modal(direction) {
  var length = state.filtered_media.length;
  state.current_modal_idx =
    (state.current_modal_idx + direction + length) % length;
  show_modal_media();
}
function toggle_modal_info() {
  var _a;
  var isHidden = DOM.modal_info.hidden;
  DOM.modal_info.hidden = !isHidden;
  (_a = document.getElementById("modalMetadata")) === null || _a === void 0
    ? void 0
    : _a.setAttribute("aria-pressed", String(isHidden));
}
function toggle_fullscreen() {
  return __awaiter(this, void 0, void 0, function () {
    var modalContent, err_1;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          modalContent = document.querySelector(".modal-content");
          if (!modalContent) return [2 /*return*/];
          _a.label = 1;
        case 1:
          _a.trys.push([1, 6, , 7]);
          if (!!document.fullscreenElement) return [3 /*break*/, 3];
          return [4 /*yield*/, modalContent.requestFullscreen()];
        case 2:
          _a.sent();
          return [3 /*break*/, 5];
        case 3:
          return [4 /*yield*/, document.exitFullscreen()];
        case 4:
          _a.sent();
          _a.label = 5;
        case 5:
          return [3 /*break*/, 7];
        case 6:
          err_1 = _a.sent();
          console.error("Fullscreen error:", err_1);
          show_toast("Fullscreen not supported", "error", 2000);
          return [3 /*break*/, 7];
        case 7:
          return [2 /*return*/];
      }
    });
  });
}
function open_kbhelp() {
  DOM.kb_help.showModal();
}
function close_kbhelp() {
  DOM.kb_help.close();
}
function setup_evlisteners() {
  var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k, _l, _m;
  var debouncedSearch = debounce(function () {
    state.current_search = DOM.search_box.value;
    state.current_page = 1;
    load_media();
  }, CONFIG.SEARCH_DEBOUNCE_MS);
  DOM.search_box.addEventListener("input", function (e) {
    var target = e.target;
    DOM.search_clear.hidden = target.value.length === 0;
    debouncedSearch();
  });
  DOM.search_clear.addEventListener("click", function () {
    DOM.search_box.value = "";
    DOM.search_clear.hidden = true;
    state.current_search = "";
    state.current_page = 1;
    load_media();
  });
  document.querySelectorAll(".filter-btn[data-filter]").forEach(function (btn) {
    btn.addEventListener("click", function () {
      document
        .querySelectorAll(".filter-btn[data-filter]")
        .forEach(function (b) {
          b.classList.remove("active");
          b.setAttribute("aria-pressed", "false");
        });
      btn.classList.add("active");
      btn.setAttribute("aria-pressed", "true");
      state.current_filter = btn.dataset.filter;
      state.current_page = 1;
      load_media();
    });
  });
  document.querySelectorAll(".view-btn").forEach(function (btn) {
    btn.addEventListener("click", function () {
      document.querySelectorAll(".view-btn").forEach(function (b) {
        b.classList.remove("active");
        b.setAttribute("aria-pressed", "false");
      });
      btn.classList.add("active");
      btn.setAttribute("aria-pressed", "true");
      state.current_view = btn.dataset.view;
      render_gallery();
      show_toast("View changed to ".concat(state.current_view), "info", 1500);
    });
  });
  DOM.sort_by.addEventListener("change", function (e) {
    var target = e.target;
    state.current_sort = target.value;
    apply_sorting();
    render_gallery();
    render_pagination_controls();
  });
  DOM.sort_order.addEventListener("click", function () {
    state.sort_order = state.sort_order === "asc" ? "desc" : "asc";
    DOM.sort_order.textContent = state.sort_order === "asc" ? "↑" : "↓";
    DOM.sort_order.setAttribute(
      "aria-label",
      "Sort ".concat(
        state.sort_order === "asc" ? "ascending" : "descending",
        ", click to toggle",
      ),
    );
    DOM.sort_order.dataset.order = state.sort_order;
    apply_sorting();
    render_gallery();
    render_pagination_controls();
  });
  DOM.items_per_page.addEventListener("change", function (e) {
    var target = e.target;
    state.items_per_page = parseInt(target.value, 10);
    state.current_page = 1;
    render_gallery();
    render_pagination_controls();
    show_toast(
      "Showing ".concat(state.items_per_page, " items per page"),
      "info",
      2000,
    );
  });
  DOM.adv_filter_toggle.addEventListener("click", function () {
    var isHidden = DOM.adv_filters.hidden;
    DOM.adv_filters.hidden = !isHidden;
    DOM.adv_filter_toggle.setAttribute("aria-expanded", String(isHidden));
  });
  (_a = document.getElementById("applyFilters")) === null || _a === void 0
    ? void 0
    : _a.addEventListener("click", apply_adv_filters);
  (_b = document.getElementById("clearFilters")) === null || _b === void 0
    ? void 0
    : _b.addEventListener("click", clear_all_filters);
  (_c = document.getElementById("downloadSelected")) === null || _c === void 0
    ? void 0
    : _c.addEventListener("click", download_selected);
  (_d = document.getElementById("clearSelection")) === null || _d === void 0
    ? void 0
    : _d.addEventListener("click", clear_sel);
  DOM.pagination.addEventListener("click", function (e) {
    var btn = e.target.closest("button");
    if (!btn) return;
    var page = btn.dataset.page;
    var action = btn.dataset.action;
    if (page) {
      goto_page(parseInt(page, 10));
    } else if (action === "prev") {
      goto_page(state.current_page - 1);
    } else if (action === "next") {
      goto_page(state.current_page + 1);
    } else if (action === "jump") {
      var input = document.getElementById("pageJumpInput");
      if (input) goto_page(parseInt(input.value, 10));
    }
  });
  DOM.pagination.addEventListener("keydown", function (e) {
    var target = e.target;
    if (e.key === "Enter" && target.id === "pageJumpInput") {
      goto_page(parseInt(target.value, 10));
    }
  });
  window.addEventListener(
    "scroll",
    function () {
      DOM.scroll_top.hidden = window.pageYOffset <= CONFIG.SCROLL_THRESHOLD;
    },
    { passive: true },
  );
  DOM.scroll_top.addEventListener("click", function () {
    window.scrollTo({ top: 0, behavior: "smooth" });
  });
  (_e = document.getElementById("modalClose")) === null || _e === void 0
    ? void 0
    : _e.addEventListener("click", close_modal);
  (_f = document.getElementById("modalNext")) === null || _f === void 0
    ? void 0
    : _f.addEventListener("click", function () {
        return nav_modal(1);
      });
  (_g = document.getElementById("modalPrev")) === null || _g === void 0
    ? void 0
    : _g.addEventListener("click", function () {
        return nav_modal(-1);
      });
  (_h = document.getElementById("modalDownload")) === null || _h === void 0
    ? void 0
    : _h.addEventListener("click", function () {
        return download_media(state.current_modal_idx);
      });
  (_j = document.getElementById("modalFullscreen")) === null || _j === void 0
    ? void 0
    : _j.addEventListener("click", toggle_fullscreen);
  (_k = document.getElementById("modalMetadata")) === null || _k === void 0
    ? void 0
    : _k.addEventListener("click", toggle_modal_info);
  DOM.modal.addEventListener("click", function (e) {
    if (e.target === DOM.modal) close_modal();
  });
  DOM.modal.addEventListener("cancel", function (e) {
    e.preventDefault();
    close_modal();
  });
  (_l = document.getElementById("keyboardHintBtn")) === null || _l === void 0
    ? void 0
    : _l.addEventListener("click", open_kbhelp);
  (_m = document.getElementById("closeKeyboardHelp")) === null || _m === void 0
    ? void 0
    : _m.addEventListener("click", close_kbhelp);
  DOM.kb_help.addEventListener("click", function (e) {
    if (e.target === DOM.kb_help) close_kbhelp();
  });
  document.addEventListener("fullscreenchange", function () {
    var btn = document.getElementById("modalFullscreen");
    if (btn) {
      btn.title = document.fullscreenElement
        ? "Exit Fullscreen"
        : "Toggle fullscreen";
    }
  });
  document.addEventListener("keydown", handle_key_down);
}
function handle_key_down(e) {
  if (DOM.modal.open) {
    switch (e.key) {
      case "Escape":
        close_modal();
        return;
      case "ArrowRight":
        nav_modal(1);
        return;
      case "ArrowLeft":
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
  var target = e.target;
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
function init() {
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
