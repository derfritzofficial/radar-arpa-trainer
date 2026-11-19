//! Bettet unter Windows das Programm-Icon und die Versionsinfos in die .exe ein.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Radar ARPA Trainer");
        res.set("FileDescription", "Radar ARPA Trainer");
        if let Err(e) = res.compile() {
            // ohne Ressourcen-Compiler trotzdem bauen – nur ohne Datei-Icon
            println!("cargo:warning=Windows-Icon konnte nicht eingebettet werden: {e}");
        }
    }
}
