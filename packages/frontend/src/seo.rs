pub fn use_page_title(title: &str) {
    let title = format!("{} | SchoolCBB", title);
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document.set_title(&title);

    if let Some(meta) = document.query_selector("meta[name='description']").ok().flatten() {
        let _ = meta.set_attribute("content", &format!("SchoolCBB - {}", title));
    }
}
