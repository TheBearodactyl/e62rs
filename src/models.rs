use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PostsResponse {
    #[serde(default)]
    pub posts: Vec<E6Post>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6PostResponse {
    #[serde(default)]
    pub post: E6Post,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct E6Post {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub file: File,
    #[serde(default)]
    pub preview: Preview,
    #[serde(default)]
    pub sample: Sample,
    #[serde(default)]
    pub score: Score,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub locked_tags: Vec<String>,
    #[serde(default)]
    pub change_seq: i64,
    #[serde(default)]
    pub flags: Flags,
    #[serde(default)]
    pub rating: String,
    #[serde(default)]
    pub fav_count: i64,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub pools: Vec<i64>,
    #[serde(default)]
    pub relationships: Relationships,
    #[serde(default)]
    pub approver_id: Option<i64>,
    #[serde(default)]
    pub uploader_id: i64,
    #[serde(default)]
    pub uploader_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub comment_count: i64,
    #[serde(default)]
    pub is_favorited: bool,
    #[serde(default)]
    pub has_notes: bool,
    #[serde(default)]
    pub duration: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct File {
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub ext: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub md5: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct Preview {
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    #[serde(default)]
    pub has: bool,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub alternates: Alternates,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternates {
    #[serde(default)]
    pub has: bool,
    #[serde(default)]
    pub original: Option<Original>,
    #[serde(default)]
    pub variants: Option<Variants>,
    #[serde(default)]
    pub samples: Option<Samples>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Original {
    #[serde(default)]
    pub fps: f64,
    #[serde(default)]
    pub codec: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variants {
    #[serde(default)]
    pub mp4: Option<Mp4>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mp4 {
    #[serde(default)]
    pub codec: String,
    #[serde(default)]
    pub fps: f64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Samples(pub HashMap<String, Quality>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quality {
    #[serde(default)]
    pub fps: f64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub codec: String,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Score {
    #[serde(default)]
    pub up: i64,
    #[serde(default)]
    pub down: i64,
    #[serde(default)]
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tags {
    #[serde(default)]
    pub general: Vec<String>,
    #[serde(default)]
    pub artist: Vec<String>,
    #[serde(default)]
    pub contributor: Vec<String>,
    #[serde(default)]
    pub copyright: Vec<String>,
    #[serde(default)]
    pub character: Vec<String>,
    #[serde(default)]
    pub species: Vec<String>,
    #[serde(default)]
    pub invalid: Vec<String>,
    #[serde(default)]
    pub meta: Vec<String>,
    #[serde(default)]
    pub lore: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flags {
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub flagged: bool,
    #[serde(default)]
    pub note_locked: bool,
    #[serde(default)]
    pub status_locked: bool,
    #[serde(default)]
    pub rating_locked: bool,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationships {
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub has_children: bool,
    #[serde(default)]
    pub has_active_children: bool,
    #[serde(default)]
    pub children: Option<Vec<i64>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagEntry {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: i64,
    #[serde(default)]
    pub post_count: i64,
}

macro_rules! impl_display {
    ($type:ty, $name:expr, $color:ident, $($field:ident: $format:expr),*) => {
        impl Display for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                writeln!(f, "{} {{", $name.$color())?;
                $(
                    writeln!(f, "  {}: {}", stringify!($field).yellow(), $format(&self.$field))?;
                )*
                writeln!(f, "}}")
            }
        }
    };
}

macro_rules! fmt_value {
    () => {
        |v| format!("{}", v)
    };
    (debug) => {
        |v| format!("{:?}", v)
    };
}

impl_display!(E6PostsResponse, "E6PostsResponse", cyan, posts: |posts: &Vec<E6Post>| {
    let mut result = String::new();
    for post in posts {
        result.push_str(&format!("\n  {}", post));
    }
    result
});

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
