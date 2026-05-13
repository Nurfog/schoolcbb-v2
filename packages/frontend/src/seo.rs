/// Updates the document title and meta tags per-page.
///
/// Sets the `<title>` element, `<meta name="description">`, and the corresponding
/// Open Graph (`og:title`, `og:description`) and Twitter Card (`twitter:title`,
/// `twitter:description`) meta tags. Skips any meta tag that isn't present in the DOM.
pub fn use_page_title(title: &str) {
    let title = format!("{} | SchoolCBB", title);
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document.set_title(&title);

    let desc = format!("SchoolCBB - {}", title);

    if let Some(meta) = document.query_selector("meta[name='description']").ok().flatten() {
        let _ = meta.set_attribute("content", &desc);
    }

    if let Some(meta) = document.query_selector("meta[property='og:title']").ok().flatten() {
        let _ = meta.set_attribute("content", &title);
    }

    if let Some(meta) = document.query_selector("meta[property='og:description']").ok().flatten() {
        let _ = meta.set_attribute("content", &desc);
    }

    if let Some(meta) = document.query_selector("meta[name='twitter:title']").ok().flatten() {
        let _ = meta.set_attribute("content", &title);
    }

    if let Some(meta) = document.query_selector("meta[name='twitter:description']").ok().flatten() {
        let _ = meta.set_attribute("content", &desc);
    }
}
