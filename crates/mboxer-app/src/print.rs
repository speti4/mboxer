//! Printing on macOS through a second webview window (task 020).
//!
//! WKWebView ignores `print()` on an iframe (tauri-apps/tauri #13451), which is
//! the whole print path of task 011. On a Mac the webview therefore hands the
//! finished print document (cid images already inlined as `data:` URLs) to the
//! `print_message` command: the document is kept here as a print job, a window
//! labelled `print-<id>` loads it from the `mboxer-print:` scheme, and once the
//! page has loaded the window runs `Webview::print()`, the native print panel
//! with *Save as PDF*. wry runs that panel as a sheet on the webview's own
//! window and reports no end, so the window is a visible preview the user
//! closes; it is dropped with the main window. Windows keeps the iframe path.
//!
//! The jobs, the page response with its CSP and the input checks are plain
//! code with unit tests; only [`open_window`] and the hooks need a Mac.

use std::collections::HashMap;

use tauri::http::{Response, StatusCode, header};

/// The URI scheme the print windows load their page from.
pub const PRINT_SCHEME: &str = "mboxer-print";

/// Every print window's label starts with this, followed by the job id.
pub const LABEL_PREFIX: &str = "print-";

/// The largest print document accepted: the body with its inlined images.
pub const MAX_PRINT_HTML: usize = 128 * 1024 * 1024;

/// The longest window title taken from a subject, in characters.
const MAX_TITLE_CHARS: usize = 120;

/// The window label of print job `id`.
pub fn label(id: u64) -> String {
    format!("{LABEL_PREFIX}{id}")
}

/// The print job a window label names, if it is a print window's.
pub fn job_of_label(label: &str) -> Option<u64> {
    label.strip_prefix(LABEL_PREFIX)?.parse().ok()
}

/// The page's Content-Security-Policy: no script, no connection, no remote
/// style, images only from `data:` — the reader's stance (task 007), with the
/// remote images the user allowed for this message (`allow_remote`) and
/// nothing else. The document's own CSP meta (copied from the reader frame)
/// applies on top of it.
pub fn page_csp(allow_remote: bool) -> String {
    let remote = if allow_remote { " https: http:" } else { "" };
    format!("default-src 'none'; img-src data:{remote}; style-src 'unsafe-inline'")
}

/// The print window's title: the subject on one line, shortened, or "Print".
pub fn window_title(subject: &str) -> String {
    let one_line: String = subject
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let words = one_line.split_whitespace().collect::<Vec<_>>().join(" ");
    if words.is_empty() {
        return "Print".to_owned();
    }
    let mut title: String = words.chars().take(MAX_TITLE_CHARS).collect();
    if words.chars().count() > MAX_TITLE_CHARS {
        title.push('…');
    }
    format!("Print: {title}")
}

/// The command's input check: a non-empty document within [`MAX_PRINT_HTML`].
pub fn check_html(html: &str) -> Result<(), String> {
    if html.trim().is_empty() {
        return Err("the print document is empty".to_owned());
    }
    if html.len() > MAX_PRINT_HTML {
        return Err(format!(
            "the print document is too large ({} MB, at most {} MB)",
            html.len() / (1024 * 1024),
            MAX_PRINT_HTML / (1024 * 1024)
        ));
    }
    Ok(())
}

/// One document waiting in, or shown by, a print window.
#[derive(Debug)]
struct PrintJob {
    html: String,
    allow_remote: bool,
    /// The print panel has been asked for; a reload does not ask again.
    printed: bool,
}

/// The print jobs of the open print windows, by job id. A job lives as long
/// as its window: the page may be loaded again (a reload) while it is open.
#[derive(Debug, Default)]
pub struct PrintJobs {
    jobs: HashMap<u64, PrintJob>,
    next_id: u64,
}

impl PrintJobs {
    /// Keeps `html` as a new job and returns its id; ids are never reused.
    pub fn add(&mut self, html: String, allow_remote: bool) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.jobs.insert(
            id,
            PrintJob {
                html,
                allow_remote,
                printed: false,
            },
        );
        id
    }

    /// The `mboxer-print:` response for a request from the webview `label`:
    /// that window's own document with the page CSP, else 404. Another
    /// webview (the main one included) gets nothing from the scheme.
    pub fn respond(&self, label: &str) -> Response<Vec<u8>> {
        let job = job_of_label(label).and_then(|id| self.jobs.get(&id));
        let Some(job) = job else {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(b"no print document".to_vec())
                .expect("a static response is valid");
        };
        Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CONTENT_SECURITY_POLICY, page_csp(job.allow_remote))
            .header(header::CACHE_CONTROL, "no-store")
            .body(job.html.clone().into_bytes())
            .expect("a header value from page_csp is valid")
    }

    /// True once per job, when the window `label` has loaded its page and
    /// should open the print panel; false for a reload or an unknown label.
    pub fn take_print(&mut self, label: &str) -> bool {
        let Some(job) = job_of_label(label).and_then(|id| self.jobs.get_mut(&id)) else {
            return false;
        };
        !std::mem::replace(&mut job.printed, true)
    }

    /// Forgets the job of the window `label` (it has been closed).
    pub fn remove(&mut self, label: &str) {
        if let Some(id) = job_of_label(label) {
            self.jobs.remove(&id);
        }
    }

    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

/// Opens the print window of a new job: a page-sized window over the main
/// one that loads the document and, once it has loaded, opens the native
/// print panel on itself. Links in the document go nowhere and no window
/// opens from it.
#[cfg(target_os = "macos")]
pub fn open_window(
    app: &tauri::AppHandle,
    html: String,
    subject: &str,
    allow_remote: bool,
) -> Result<(), String> {
    use tauri::webview::{NewWindowResponse, PageLoadEvent};
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    use crate::state::AppState;

    let id = app.state::<AppState>().print_jobs().add(html, allow_remote);
    let url = format!("{PRINT_SCHEME}://localhost/")
        .parse()
        .map_err(|err| format!("bad print URL: {err}"))?;
    let built = WebviewWindowBuilder::new(app, label(id), WebviewUrl::CustomProtocol(url))
        .title(window_title(subject))
        .inner_size(820.0, 1000.0)
        .min_inner_size(400.0, 300.0)
        .center()
        .focused(true)
        .on_navigation(|url| url.scheme() == PRINT_SCHEME)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_page_load(|window, payload| {
            if payload.event() != PageLoadEvent::Finished || payload.url().scheme() != PRINT_SCHEME
            {
                return;
            }
            let print = window
                .state::<AppState>()
                .print_jobs()
                .take_print(window.label());
            if print {
                // From another thread, so the call reaches the main thread
                // as an event after this navigation callback has returned
                // (from the main thread it would run inline). The panel is a
                // sheet on this window; print() returns while it is open.
                std::thread::spawn(move || {
                    if let Err(err) = window.print() {
                        eprintln!("print failed: {err}");
                    }
                });
            }
        })
        .build();
    if let Err(err) = built {
        app.state::<AppState>().print_jobs().remove(&label(id));
        return Err(format!("could not open the print window: {err}"));
    }
    Ok(())
}

/// The `mboxer-print:` scheme handler: serves each print window its document.
#[cfg(target_os = "macos")]
pub fn serve(
    ctx: tauri::UriSchemeContext<'_, tauri::Wry>,
    _request: tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    use tauri::Manager;
    ctx.app_handle()
        .state::<crate::state::AppState>()
        .print_jobs()
        .respond(ctx.webview_label())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_name_their_job() {
        assert_eq!(label(7), "print-7");
        assert_eq!(job_of_label("print-7"), Some(7));
        assert_eq!(job_of_label("main"), None);
        assert_eq!(job_of_label("print-"), None);
        assert_eq!(job_of_label("print-x"), None);
    }

    #[test]
    fn the_page_csp_blocks_remote_content_unless_allowed() {
        let blocked = page_csp(false);
        assert_eq!(
            blocked,
            "default-src 'none'; img-src data:; style-src 'unsafe-inline'"
        );
        assert!(!blocked.contains("http"));
        assert!(!blocked.contains("script"));
        let allowed = page_csp(true);
        assert!(allowed.contains("img-src data: https: http:;"));
        assert!(allowed.starts_with("default-src 'none';"));
    }

    #[test]
    fn titles_are_one_short_line() {
        assert_eq!(
            window_title("  Invoice\r\n\tMarch  "),
            "Print: Invoice March"
        );
        assert_eq!(window_title(" \n "), "Print");
        let long = window_title(&"é".repeat(300));
        assert_eq!(long.chars().count(), "Print: ".len() + MAX_TITLE_CHARS + 1);
        assert!(long.ends_with('…'));
    }

    #[test]
    fn the_document_must_be_there_and_bounded() {
        assert!(check_html("<!doctype html><p>x</p>").is_ok());
        assert!(check_html("").is_err());
        assert!(check_html("  \n").is_err());
        assert!(check_html(&"x".repeat(MAX_PRINT_HTML)).is_ok());
        assert!(check_html(&"x".repeat(MAX_PRINT_HTML + 1)).is_err());
    }

    #[test]
    fn a_window_gets_its_own_document_only() {
        let mut jobs = PrintJobs::default();
        let first = jobs.add("<p>one</p>".to_owned(), false);
        let second = jobs.add("<p>two</p>".to_owned(), true);
        assert_ne!(first, second);

        let response = jobs.respond(&label(first));
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body(), b"<p>one</p>");
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            page_csp(false).as_str()
        );
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8"
        );
        let response = jobs.respond(&label(second));
        assert_eq!(response.body(), b"<p>two</p>");
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            page_csp(true).as_str()
        );

        for other in ["main", "print-99", "print-"] {
            let response = jobs.respond(other);
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{other}");
            assert!(
                !response
                    .headers()
                    .contains_key(header::CONTENT_SECURITY_POLICY)
            );
        }
    }

    #[test]
    fn a_job_prints_once_and_goes_with_its_window() {
        let mut jobs = PrintJobs::default();
        let id = jobs.add("<p>x</p>".to_owned(), false);
        assert!(jobs.take_print(&label(id)));
        assert!(
            !jobs.take_print(&label(id)),
            "a reload does not print again"
        );
        assert!(!jobs.take_print("main"));
        assert_eq!(jobs.len(), 1);
        jobs.remove(&label(id));
        assert!(jobs.is_empty());
        assert_eq!(jobs.respond(&label(id)).status(), StatusCode::NOT_FOUND);
        let next = jobs.add("<p>y</p>".to_owned(), false);
        assert!(next > id, "ids are not reused");
    }
}
