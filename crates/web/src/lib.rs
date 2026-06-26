use askama::Template;
use axum::response::Html;
use datastar::{consts::ElementPatchMode, patch_elements::PatchElements};
use magnetar_db::Monitor;

fn render<T: Template>(t: &T) -> anyhow::Result<Html<String>> {
    Ok(Html(t.render()?))
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomePage {
    count: i64,
}

#[derive(Template)]
#[template(path = "fragments/count.html")]
struct CountFragment {
    count: i64,
}

#[derive(Template)]
#[template(path = "pages/monitors.html")]
struct MonitorsPage {}

#[derive(Template)]
#[template(path = "fragments/monitor_list.html")]
struct MonitorListFragment {
    monitors: Vec<Monitor>,
}

pub fn home_page(count: i64) -> anyhow::Result<Html<String>> {
    render(&HomePage { count })
}

pub fn count_fragment(count: i64) -> anyhow::Result<Html<String>> {
    render(&CountFragment { count })
}

pub fn monitors_page() -> anyhow::Result<Html<String>> {
    render(&MonitorsPage {})
}

pub fn monitor_list_patch(monitors: Vec<Monitor>) -> anyhow::Result<PatchElements> {
    let html = render(&MonitorListFragment { monitors })?;

    Ok(PatchElements::new(html.0)
        .selector("#monitor-list")
        .mode(ElementPatchMode::Inner))
}

#[cfg(test)]
mod tests {
    use super::*;
    use magnetar_db::Monitor;
    use time::OffsetDateTime;
    use uuid::Uuid;

    fn fixture_monitor() -> Monitor {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

        Monitor {
            id: Uuid::nil(),
            name: "Example monitor".to_string(),
            url: "https://example.com".to_string(),
            interval_seconds: 60,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn monitor_list_patch_renders_monitor_fragment() {
        let rendered = render(&MonitorListFragment {
            monitors: vec![fixture_monitor()],
        })
        .unwrap();

        assert!(rendered.0.contains("Example monitor"));
        assert!(rendered.0.contains("https://example"));
    }
}
