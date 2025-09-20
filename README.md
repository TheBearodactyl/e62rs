# CONFIG OPTIONS:

`download_dir`: The directory to download posts to
`output_format`: The output format for downloaded files
`post_count`: The amount of posts to show in a search
`base_url`: The base URL of the API (defaults to https://e621.net)
`display`: Post viewing settings
`tags`: The path to `tags.csv` that's used for tag searching/autocompletion
`http`: HTTP client configuration
`cache`: Cache configuration
`performance`: Performance settings
`ui`: UI settings

`pool_max_idle_per_host`: Connection pool size per host
`pool_idle_timeout_secs`: Connection pool idle timeout in seconds
`timeout_secs`: Request timeout in seconds
`connect_timeout_secs`: Connection timeout in seconds
`max_connections`: Max concurrent connections
`http2_prior_knowledge`: Enable HTTP/2
`tcp_keepalive`: Enable keep-alive
`user_agent`: User agent string

`enabled`: Enable response caching
`cache_dir`: Cache directory
`ttl_secs`: Cache TTL in seconds
`max_size_mb`: Max cache size in MB

`concurrent_downloads`: Number of concurrent downloads
`prefetch_enabled`: Prefetch next batch of posts
`prefetch_batch_size`: Prefetch batch size
`preload_images`: Enable image preloading
`max_preload_size_mb`: Max image preload size in MB

`progress_refresh_rate`: Progress bar refresh rate (Hz)
`detailed_progress`: Show detailed progress info
`auto_clear_progress`: Auto-clear completed progress bars
`pagination_size`: Pagination size for post listings
`colored_output`: Enable colored output

`width`: The max width of displayed images
`height`: The max height of displayed images
`image_when_info`: Whether to display the image when showing post info
`sixel_quality`: Image quality for sixel conversion (1-100)
`resize_method`: Resize method (nearest, linear, cubic, gaussian, lanczos3)
