use crate::index::query::{ParsedQuery, TypeFilter};
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;

pub fn meta_matches(
    store: &ArchiveStore,
    query: &ParsedQuery,
    ext_ids: &[u16],
    ex_ext_ids: &[u16],
    idx: usize,
) -> bool {
    let ext_id = store.file_ext.get(idx).copied().unwrap_or(0);
    if (!ext_ids.is_empty() && !ext_ids.contains(&ext_id)) || ex_ext_ids.contains(&ext_id) {
        return false;
    }
    if !query.type_filters.is_empty() {
        let kind = store
            .file_kind
            .get(idx)
            .copied()
            .unwrap_or(MediaKind::Other);
        let ok = query.type_filters.iter().any(|&tf| match tf {
            TypeFilter::Image => kind == MediaKind::Image,
            TypeFilter::Video => kind == MediaKind::Video,
            TypeFilter::Raw => store.is_raw_ext_id(ext_id),
            TypeFilter::Other => kind == MediaKind::Other,
        });
        if !ok {
            return false;
        }
    }
    let size = store.file_size.get(idx).copied().unwrap_or(0);
    if query.min_size.is_some_and(|m| size < m) || query.max_size.is_some_and(|m| size > m) {
        return false;
    }
    let year = store.file_year.get(idx).copied().unwrap_or(0);
    if query.min_year.is_some_and(|y| year == 0 || year < y)
        || query.max_year.is_some_and(|y| year == 0 || year > y)
    {
        return false;
    }
    query.cam_filters.is_empty() || {
        let cam_id = store.file_cam.get(idx).copied().unwrap_or(0);
        let cam = store.cam_name(cam_id).to_ascii_lowercase();
        query.cam_filters.iter().any(|c| cam.contains(c))
    }
}
