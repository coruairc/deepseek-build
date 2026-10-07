use std::cmp::{Ordering, Reverse};

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use super::PartialReason;
use super::envelope::SessionKind;
use super::row::UnifiedRow;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct CompositeCursor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<BoundaryKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conv_page_token: Option<String>,
    #[serde(default)]
    pub conv_page_drained: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct BoundaryKey {
    pub updated_at: String,
    pub kind: SessionKind,
    pub session_id: String,
}

impl CompositeCursor {
    pub(super) fn decode(raw: Option<&str>) -> Self {
        raw.filter(|s| !s.is_empty())
            .and_then(|s| {
                base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(s)
                    .ok()
            })
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub(super) fn encode(&self) -> String {
        let json = serde_json::to_vec(self).unwrap_or_default();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
    }
}

pub(super) struct Paginated {
    pub candidates: Vec<UnifiedRow>,
    pub emit_count: usize,
    pub next_cursor: Option<CompositeCursor>,
    pub partial: Option<PartialReason>,
}

pub(super) fn merge_and_paginate(
    local: Vec<UnifiedRow>,
    cursor: &CompositeCursor,
    limit: usize,
) -> Paginated {
    let mut keyed: Vec<(SortKey, UnifiedRow)> = local
        .into_iter()
        .map(|row| (row_sort_key(&row), row))
        .collect();

    if let Some(boundary) = &cursor.boundary {
        let bkey = boundary_sort_key(boundary);
        keyed.retain(|(k, _)| k.cmp(&bkey) == Ordering::Greater);
    }

    keyed.sort_by(|(a, _), (b, _)| a.cmp(b));

    let emit_count = keyed.len().min(limit);
    let new_boundary = emit_count
        .checked_sub(1)
        .and_then(|i| keyed.get(i))
        .map(|(_, row)| boundary_of(row));

    let tail = keyed.get(emit_count..).unwrap_or(&[]);
    let local_has_more = tail.iter().any(|(_, r)| r.kind == SessionKind::Build);

    let next_cursor = local_has_more.then(|| CompositeCursor {
        boundary: new_boundary.or_else(|| cursor.boundary.clone()),
        conv_page_token: cursor.conv_page_token.clone(),
        conv_page_drained: cursor.conv_page_drained,
    });

    let candidates: Vec<UnifiedRow> = keyed.into_iter().map(|(_, row)| row).collect();

    Paginated {
        candidates,
        emit_count,
        next_cursor,
        partial: None,
    }
}

type SortKey = (
    Reverse<Option<chrono::DateTime<chrono::FixedOffset>>>,
    SessionKind,
    String,
);

fn row_sort_key(row: &UnifiedRow) -> SortKey {
    (
        Reverse(row.sort_timestamp()),
        row.kind,
        row.legacy.session_id.clone(),
    )
}

fn boundary_sort_key(boundary: &BoundaryKey) -> SortKey {
    (
        Reverse(parse_ts(&boundary.updated_at)),
        boundary.kind,
        boundary.session_id.clone(),
    )
}

fn boundary_of(row: &UnifiedRow) -> BoundaryKey {
    BoundaryKey {
        updated_at: row.updated_at.clone().unwrap_or_default(),
        kind: row.kind,
        session_id: row.legacy.session_id.clone(),
    }
}

fn parse_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(s).ok()
}

pub(super) fn timestamp_desc(
    a: Option<chrono::DateTime<chrono::FixedOffset>>,
    b: Option<chrono::DateTime<chrono::FixedOffset>>,
) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(super) fn cmp_total_order(a: &UnifiedRow, b: &UnifiedRow) -> Ordering {
    timestamp_desc(a.sort_timestamp(), b.sort_timestamp())
        .then_with(|| a.kind.cmp(&b.kind))
        .then_with(|| a.legacy.session_id.cmp(&b.legacy.session_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trips() {
        let cur = CompositeCursor {
            boundary: Some(BoundaryKey {
                updated_at: "2026-06-01T00:00:00Z".into(),
                kind: SessionKind::Chat,
                session_id: "conv_1".into(),
            }),
            conv_page_token: Some("p3".into()),
            conv_page_drained: true,
        };
        let decoded = CompositeCursor::decode(Some(&cur.encode()));
        assert_eq!(decoded.conv_page_token.as_deref(), Some("p3"));
        assert!(decoded.conv_page_drained);
        let b = decoded.boundary.unwrap();
        assert_eq!(b.session_id, "conv_1");
        assert_eq!(b.kind, SessionKind::Chat);
    }

    #[test]
    fn malformed_cursor_decodes_to_fresh_first_page() {
        for bad in [Some("not base64 !!!"), Some(""), None] {
            let c = CompositeCursor::decode(bad);
            assert!(c.boundary.is_none());
            assert!(c.conv_page_token.is_none());
            assert!(!c.conv_page_drained);
        }
    }
}
