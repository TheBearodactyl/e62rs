# e62rs

An extremely configurable client for browsing [e621](https://e621.net) and [e926](https://e926.net)

---

## Configuration

Configuration is loaded in this order:

1. Local `e62rs` file (e.g. `e62rs.toml`, `e62rs.yaml`, or `e62rs.json`) – optional.
2. Global `e62rs` file (e.g. `~/.config/e62rs.toml`, `~/AppData/Roaming/e62rs.yaml`, or `~/Library/Application Support/e926.json`) – optional.
3. Environment variables prefixed with `E62RS_`.
4. Defaults defined in code.

---

## Top-Level Configuration (`Cfg`)

| Key             | Type                | Default                                                                | Description                                                           |
| --------------- | ------------------- | ---------------------------------------------------------------------- | --------------------------------------------------------------------- |
| `download_dir`  | `String`            | `"downloads"`                                                          | Directory where posts are saved.                                      |
| `output_format` | `String`            | `"$artists[3]/$rating/$tags[3] - $id - $date $time - $score.$ext"`     | Filename template (see [Filename Formatting](#-filename-formatting)). |
| `post_count`    | `u64`               | `320`                                                                  | Number of posts returned per search.                                  |
| `base_url`      | `String`            | `"https://e621.net"`                                                   | API base URL.                                                         |
| `display`       | `ImageDisplay`      | see [Image Display](#image-display-configuration-imagedisplay)         | Image preview/display settings.                                       |
| `tags`          | `String`            | `"data/tags.csv"`                                                      | Path to `tags.csv` for tag autocompletion.                            |
| `pools`         | `String`            | `"data/pools.csv"`                                                     | Path to `pools.csv` for pool autocompletion.                          |
| `http`          | `HttpConfig`        | see [HTTP Config](#http-configuration-httpconfig)                      | HTTP client options.                                                  |
| `cache`         | `CacheConfig`       | see [Cache Config](#cache-configuration-cacheconfig)                   | Caching options.                                                      |
| `performance`   | `PerformanceConfig` | see [Performance Config](#performance-configuration-performanceconfig) | Performance tuning.                                                   |
| `ui`            | `UiConfig`          | see [UI Config](#ui-configuration-uiconfig)                            | UI/console options.                                                   |
| `search`        | `SearchCfg`         | see [Search Config](#search-configuration-searchcfg)                   | Search tuning.                                                        |
| `completion`    | `CompletionCfg`     | see [Completion Config](#completion-configuration-completioncfg)       | Autocompletion settings.                                              |
| `blacklist`     | `Vec<String>`       | `["young", "rape", "feral", "bestiality"]`                             | Blacklisted tags to always filter out.                                |

---

## HTTP Configuration (`HttpConfig`)

| Key                      | Type     | Default | Description                          |
| ------------------------ | -------- | ------- | ------------------------------------ |
| `pool_max_idle_per_host` | `usize`  | `32`    | Max idle connections per host.       |
| `pool_idle_timeout_secs` | `u64`    | `90`    | Idle timeout for pooled connections. |
| `timeout_secs`           | `u64`    | `30`    | Request timeout.                     |
| `connect_timeout_secs`   | `u64`    | `10`    | Connection establishment timeout.    |
| `max_connections`        | `usize`  | `15`    | Max concurrent connections.          |
| `http2_prior_knowledge`  | `bool`   | `true`  | Enable HTTP/2 prior knowledge.       |
| `tcp_keepalive`          | `bool`   | `true`  | Enable TCP keep-alive.               |
| `user_agent`             | `String` | _none_  | Custom User-Agent string.            |

---

## Cache Configuration (`CacheConfig`)

| Key           | Type     | Default    | Description                   |
| ------------- | -------- | ---------- | ----------------------------- |
| `enabled`     | `bool`   | `true`     | Enable/disable caching.       |
| `cache_dir`   | `String` | `".cache"` | Cache directory.              |
| `ttl_secs`    | `u64`    | `3600`     | Cache time-to-live (seconds). |
| `max_size_mb` | `u64`    | `500`      | Max cache size in MB.         |

---

## Performance Configuration (`PerformanceConfig`)

| Key                    | Type    | Default | Description                         |
| ---------------------- | ------- | ------- | ----------------------------------- |
| `concurrent_downloads` | `usize` | `15`    | Number of concurrent downloads.     |
| `prefetch_enabled`     | `bool`  | `true`  | Enable prefetching posts.           |
| `prefetch_batch_size`  | `usize` | `10`    | Number of posts per prefetch batch. |
| `preload_images`       | `bool`  | `false` | Enable image preloading.            |
| `max_preload_size_mb`  | `u64`   | `100`   | Max preload size (MB).              |

---

## UI Configuration (`UiConfig`)

| Key                     | Type    | Default | Description                           |
| ----------------------- | ------- | ------- | ------------------------------------- |
| `progress_refresh_rate` | `u64`   | `20`    | Progress bar refresh rate (Hz).       |
| `detailed_progress`     | `bool`  | `true`  | Show detailed progress.               |
| `auto_clear_progress`   | `bool`  | `true`  | Clear progress bars after completion. |
| `pagination_size`       | `usize` | `20`    | Posts per page in listings.           |
| `colored_output`        | `bool`  | `true`  | Enable colored console output.        |

---

## Image Display Configuration (`ImageDisplay`)

| Key               | Type     | Default      | Description                                                              |
| ----------------- | -------- | ------------ | ------------------------------------------------------------------------ |
| `width`           | `u64`    | `800`        | Max display width.                                                       |
| `height`          | `u64`    | `600`        | Max display height.                                                      |
| `image_when_info` | `bool`   | `true`       | Show image in post info.                                                 |
| `sixel_quality`   | `u8`     | `100`        | Quality for sixel conversion (1–100).                                    |
| `resize_method`   | `String` | `"lanczos3"` | Resize algorithm (`nearest`, `linear`, `cubic`, `gaussian`, `lanczos3`). |

---

## Search Configuration (`SearchCfg`)

| Key                        | Type   | Default | Description                                                  |
| -------------------------- | ------ | ------- | ------------------------------------------------------------ |
| `min_posts_on_tag`         | `u64`  | `2`     | Minimum number of posts for a tag to appear in suggestions.  |
| `min_posts_on_pool`        | `u64`  | `2`     | Minimum number of posts for a pool to appear in suggestions. |
| `show_inactive_pools`      | `bool` | `true`  | Show inactive pools.                                         |
| `sort_pools_by_post_count` | `bool` | `false` | Sort pools by number of posts.                               |
| `sort_tags_by_post_count`  | `bool` | `true`  | Sort tags by number of posts.                                |
| `min_post_score`           | `i64`  | `0`     | Minimum score a post must have to appear in results.         |
| `max_post_score`           | `i64`  | `∞`     | Maximum score a post can have to appear in results.          |
| `reverse_tags_order`       | `bool` | `false` | Reverse alphabetic order of tag sorting.                     |

---

## Completion Configuration (`CompletionCfg`)

| Key                        | Type  | Default | Description                                         |
| -------------------------- | ----- | ------- | --------------------------------------------------- |
| `tag_similarity_threshold` | `f64` | `0.8`   | Threshold for fuzzy tag autocompletion (0–1 range). |

---

## Filename Formatting

The `output_format` setting controls how filenames are generated when saving posts.

Forward slashes denote subfolders.

### Available placeholders:

- `$id` → post ID
- `$rating` → rating (e.g. `"safe"`, `"questionable"`, `"explicit"`)
- `$rating_first` → first char of rating
- `$score` → post score
- `$fav_count` → number of favorites
- `$comment_count` → number of comments
- `$md5` → MD5 hash of file
- `$ext` → file extension
- `$width` / `$height` / `$size` → file dimensions and size
- `$artist` → first listed artist (or `"unknown"`)
- `$uploader` / `$uploader_id` → uploader info

### Date/time placeholders:

- From post creation date: `$year`, `$month`, `$day`, `$hour`, `$minute`, `$second`, `$date`, `$time`, `$datetime`
- From current time: `$now_year`, `$now_month`, `$now_day`, `$now_hour`, `$now_minute`, `$now_second`, `$now_date`, `$now_time`, `$now_datetime`

### Tag placeholders:

- `$tags[N]` → first `N` general tags joined by commas.
- `$artists[N]` → first `N` artist tags joined by commas.

### Example:

```toml
output_format = "$id-$artists[1]-$score.$ext"
```

Might produce:

```
123456-artistname-42.png
```
