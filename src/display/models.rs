use {
    crate::{fmt_value, impl_display, models::*},
    owo_colors::OwoColorize,
};

impl_display!(E6PostsResponse, "E6PostsResponse", cyan, posts: |posts: &Vec<E6Post>| {
    let mut result = String::new();
    for post in posts {
        result.push_str(&format!("\n  {}", post));
    }
    result
});

impl_display!(E6PoolsResponse, "E6PoolsResponse", cyan, pools: |pools: &Vec<E6Pool>| {
    let mut result = String::new();
    for pool in pools {
        result.push_str(&format!("\n  {}", pool));
    }
    result
});

impl_display!(E6PoolResponse, "E6PoolResponse", cyan, pool: |p: &E6Pool| format!("{}", p));

impl_display!(
    E6Pool,
    "E6Pool",
    green,
    id: fmt_value!(),
    name: fmt_value!(),
    created_at: fmt_value!(),
    updated_at: fmt_value!(),
    creator_id: fmt_value!(),
    creator_name: fmt_value!(),
    description: fmt_value!(),
    is_active: fmt_value!(),
    category: fmt_value!(),
    post_ids: fmt_value!(debug),
    post_count: fmt_value!()
);

impl_display!(
    PoolEntry,
    "PoolEntry",
    green,
    id: fmt_value!(),
    name: fmt_value!(),
    description: fmt_value!(),
    category: fmt_value!()
);

impl_display!(E6PostResponse, "E6PostResponse", cyan, post: |p: &E6Post| format!("{}", p));

impl_display!(
    TagEntry,
    "TagEntry",
    green,
    id: fmt_value!(),
    name: fmt_value!(),
    category: fmt_value!(),
    post_count: fmt_value!()
);

impl_display!(
    E6Post,
    "E6Post",
    green,
    id: fmt_value!(),
    created_at: fmt_value!(),
    updated_at: fmt_value!(),
    file: fmt_value!(),
    preview: fmt_value!(),
    sample: fmt_value!(),
    score: fmt_value!(),
    tags: fmt_value!(debug),
    locked_tags: fmt_value!(debug),
    change_seq: fmt_value!(),
    flags: fmt_value!(debug),
    rating: fmt_value!(),
    fav_count: fmt_value!(),
    sources: fmt_value!(debug),
    pools: fmt_value!(debug),
    relationships: fmt_value!(),
    approver_id: fmt_value!(debug),
    uploader_id: fmt_value!(),
    uploader_name: fmt_value!(),
    description: fmt_value!(),
    comment_count: fmt_value!(),
    is_favorited: fmt_value!(),
    has_notes: fmt_value!(),
    duration: fmt_value!(debug)
);

impl_display!(
    File,
    "File",
    blue,
    width: fmt_value!(),
    height: fmt_value!(),
    ext: fmt_value!(),
    size: fmt_value!(),
    md5: fmt_value!(),
    url: fmt_value!(debug)
);

impl_display!(
    Preview,
    "Preview",
    blue,
    width: fmt_value!(),
    height: fmt_value!(),
    url: fmt_value!(debug),
    alt: fmt_value!(debug)
);

impl_display!(
    Sample,
    "Sample",
    blue,
    has: fmt_value!(),
    width: fmt_value!(),
    height: fmt_value!(),
    url: fmt_value!(debug),
    alt: fmt_value!(debug),
    alternates: fmt_value!()
);

impl_display!(
    Alternates,
    "Alternates",
    blue,
    has: fmt_value!(),
    original: fmt_value!(debug),
    variants: fmt_value!(debug),
    samples: fmt_value!(debug)
);

impl_display!(
    Original,
    "Original",
    blue,
    fps: fmt_value!(),
    codec: fmt_value!(),
    size: fmt_value!(),
    width: fmt_value!(),
    height: fmt_value!(),
    url: fmt_value!(debug)
);

impl_display!(Variants, "Variants", blue, mp4: fmt_value!(debug));

impl_display!(
    Mp4,
    "Mp4",
    blue,
    codec: fmt_value!(),
    fps: fmt_value!(),
    size: fmt_value!(),
    width: fmt_value!(),
    height: fmt_value!(),
    url: fmt_value!(debug)
);

impl_display!(
    Quality,
    "Quality",
    blue,
    fps: fmt_value!(),
    size: fmt_value!(),
    codec: fmt_value!(),
    width: fmt_value!(),
    height: fmt_value!(),
    url: fmt_value!(debug)
);

impl_display!(
    Score,
    "Score",
    blue,
    up: fmt_value!(),
    down: fmt_value!(),
    total: fmt_value!()
);

impl_display!(
    Tags,
    "Tags",
    blue,
    general: fmt_value!(debug),
    artist: fmt_value!(debug),
    contributor: fmt_value!(debug),
    copyright: fmt_value!(debug),
    character: fmt_value!(debug),
    species: fmt_value!(debug),
    invalid: fmt_value!(debug),
    meta: fmt_value!(debug),
    lore: fmt_value!(debug)
);

impl_display!(
    Flags,
    "Flags",
    blue,
    pending: fmt_value!(),
    flagged: fmt_value!(),
    note_locked: fmt_value!(),
    status_locked: fmt_value!(),
    rating_locked: fmt_value!(),
    deleted: fmt_value!()
);

impl_display!(
    Relationships,
    "Relationships",
    blue,
    parent_id: fmt_value!(debug),
    has_children: fmt_value!(),
    has_active_children: fmt_value!(),
    children: fmt_value!(debug)
);
