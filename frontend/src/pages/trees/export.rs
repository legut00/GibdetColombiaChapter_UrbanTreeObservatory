//! CSV downloads. The backend builds the files (`/biodiversity/inventory/export/`):
//! all matching trees through a plain link (the server streams it as an attachment),
//! the selected ones through a POST whose response is saved from a Blob.

use wasm_bindgen::JsCast;

use crate::api::console_error;

fn click_link(href: &str, file_name: Option<&str>) {
    let anchor = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.create_element("a").ok())
        .and_then(|e| e.dyn_into::<web_sys::HtmlAnchorElement>().ok());
    match anchor {
        Some(a) => {
            a.set_href(href);
            if let Some(name) = file_name {
                a.set_download(name);
            }
            a.click();
        }
        None => console_error("CSV export failed:", "could not create the download link"),
    }
}

/// Every tree matching the filters (`query` = the list's query string, without `page`).
pub fn download_filtered(export_url: &str, query: &str) {
    let sep = if query.is_empty() { "" } else { "?" };
    click_link(&format!("{export_url}{sep}{query}"), None);
}

/// The selected trees. Returns an error message for the UI.
pub async fn download_selected(export_url: &str, ids: &[i64]) -> Result<(), String> {
    let body = serde_json::json!({ "ids": ids }).to_string();
    let resp = gloo_net::http::Request::post(export_url)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("export failed: {} {}", resp.status(), resp.status_text()));
    }
    let csv = resp.text().await.map_err(|e| e.to_string())?;
    // Text decoding drops the server's BOM; Excel needs it to read the file as UTF-8.
    let csv = if csv.starts_with('\u{FEFF}') { csv } else { format!("\u{FEFF}{csv}") };

    let parts = js_sys::Array::of1(&csv.into());
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/csv;charset=utf-8");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)
        .map_err(|_| "could not build the file".to_string())?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "could not build the file".to_string())?;
    let date: String = js_sys::Date::new_0().to_iso_string().into();
    click_link(&url, Some(&format!("tree-inventory-selected-{}.csv", &date[..10])));
    // Revoked later: some browsers read the Blob after `click()` returns.
    leptos::prelude::set_timeout(
        move || {
            let _ = web_sys::Url::revoke_object_url(&url);
        },
        std::time::Duration::from_secs(30),
    );
    Ok(())
}
