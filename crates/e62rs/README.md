# e62rs

An extremely configurable client for browsing [e621](https://e621.net) and [e926](https://e926.net)

---

## 📦 Configuration

Configuration is loaded in this order:

1. `e62rs` file (e.g. `e62rs.toml`, `e62rs.yaml`, or `e62rs.json`) – optional.
2. Environment variables prefixed with `E62RS_`.
3. Defaults defined in code.

---

## Top-Level Configuration (`Cfg`)

| Key             | Type                | Default              | Description                                                           |
| --------------- | ------------------- | -------------------- | --------------------------------------------------------------------- |
| `download_dir`  | `String`            | `"downloads"`        | Directory where posts are saved.                                      |
| `output_format` | `String`            | `"$id.$ext"`         | Filename template (see [Filename Formatting](#-filename-formatting)). |
| `post_count`    | `u64`               | `32`                 | Number of posts returned per search.                                  |
| `base_url`      | `String`            | `"https://e621.net"` | API base URL.                                                         |
| `display`       | `ImageDisplay`      | see below            | Image preview/display settings.                                       |
| `tags`          | `String`            | `"data/tags.csv"`    | Path to `tags.csv` for tag autocompletion.                            |
| `http`          | `HttpConfig`        | see below            | HTTP client options.                                                  |
| `cache`         | `CacheConfig`       | see below            | Caching options.                                                      |
| `performance`   | `PerformanceConfig` | see below            | Performance tuning.                                                   |
| `ui`            | `UiConfig`          | see below            | UI/console options.                                                   |

---

## HTTP Configuration (`HttpConfig`)

| Key                      | Type     | Default | Description                          |
| ------------------------ | -------- | ------- | ------------------------------------ |
| `pool_max_idle_per_host` | `usize`  | `32`    | Max idle connections per host.       |
| `pool_idle_timeout_secs` | `u64`    | `90`    | Idle timeout for pooled connections. |
| `timeout_secs`           | `u64`    | `30`    | Request timeout.                     |
| `connect_timeout_secs`   | `u64`    | `10`    | Connection establishment timeout.    |
| `max_connections`        | `usize`  | `2`     | Max concurrent connections.          |
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
| `concurrent_downloads` | `usize` | `2`     | Number of concurrent downloads.     |
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
| `image_when_info` | `bool`   | `false`      | Show image in post info.                                                 |
| `sixel_quality`   | `u8`     | `75`         | Quality for sixel conversion (1–100).                                    |
| `resize_method`   | `String` | `"lanczos3"` | Resize algorithm (`nearest`, `linear`, `cubic`, `gaussian`, `lanczos3`). |

---

## Filename Formatting

The `output_format` setting controls how filenames are generated when saving posts.

Forward slashes denote subfolders

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

### Example:

```toml
output_format = "$id-$artist-$score.$ext"
```

Might produce:

```
123456-artistname-42.png
```
