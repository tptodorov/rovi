//! `docs/reference/protocol.md` shows `transports::DIFFERENCES` (add-rovi-protocol task 2.7).

use std::path::PathBuf;

const START: &str = "<!-- generated: rovi-protocol transports::table -->\n";
const END: &str = "<!-- end generated -->";

#[test]
fn the_reference_shows_the_crates_transport_differences() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference/protocol.md");
    let doc = std::fs::read_to_string(&path).unwrap();
    let (head, rest) = doc.split_once(START).expect("start marker");
    let (_, tail) = rest.split_once(END).expect("end marker");
    let mut table = String::new();
    rovi_protocol::transports::table(&mut table).unwrap();
    let generated = format!("{head}{START}{table}{END}{tail}");
    if std::env::var_os("ROVI_BLESS_DOCS").is_some() {
        std::fs::write(&path, &generated).unwrap();
        return;
    }
    assert!(
        doc == generated,
        "docs/reference/protocol.md is stale: rerun with ROVI_BLESS_DOCS=1"
    );
}
