use ratatui::Terminal;
use ratatui::backend::TestBackend;

use myharness::brand::{leaks_vendor_chrome, remap_tool, strip_chrome};
use myharness::tui::App;

#[test]
fn remaps_concise_namespace() {
    assert_eq!(remap_tool("GrokBuildConcise:bash"), "bash");
}

#[test]
fn chrome_frame_has_our_wordmark_not_vendor() {
    let mut app = App::default();
    app.push_tool("GrokBuildConcise:bash", "uname -sm");
    let backend = TestBackend::new(72, 16);
    let mut term = Terminal::new(backend).unwrap();
    term.draw(|f| app.draw(f)).unwrap();
    let buf = term.backend().buffer();
    let mut dumped = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            dumped.push_str(buf[(x, y)].symbol());
        }
        dumped.push('\n');
    }
    assert!(dumped.contains("myharness"), "{dumped}");
    assert!(dumped.contains("MiniMax-M3"), "{dumped}");
    assert!(dumped.contains("[tool]  bash"), "{dumped}");
    assert!(!leaks_vendor_chrome(&dumped), "{dumped}");
    assert!(!dumped.to_ascii_lowercase().contains("grok build"));
}

#[test]
fn strip_drops_think_and_pager_copy() {
    let raw = "<think>secret</think>\nGrok Build TUI\nok\n";
    let cleaned = strip_chrome(raw);
    assert_eq!(cleaned, "ok");
}
