//! Kapitel 0: Einführung und Inhaltsübersicht.

use eframe::egui::{self, RichText};

use super::{pal, sub, text, Chapter};
use crate::i18n::t;

pub fn ui(ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
    text(
        ui,
        t(
            "Dieses Arbeitsbuch macht die wichtigsten Radarphänomene erlebbar. Jedes Kapitel hat eine interaktive Darstellung mit Reglern, eine kurze Erklärung, einen Merke-Kasten, Querverweise zu verwandten Kapiteln und Übungsfragen mit Lösungen – deren Antworten Sie direkt am Modell ausprobieren können.",
            "This workbook lets you experience the most important radar phenomena. Every chapter has an interactive view with controls, a short explanation, key points, links to related chapters and practice questions with answers – which you can try out directly on the model.",
        ),
    );
    text(
        ui,
        t(
            "Tipp: Die Kapitel bauen locker aufeinander auf, lassen sich aber auch einzeln öffnen. „◀ Zurück“ links unten führt zum vorher besuchten Kapitel zurück.",
            "Tip: The chapters build on each other loosely but can be opened individually. “◀ Back” at the bottom left returns to the previously visited chapter.",
        ),
    );
    sub(ui, t("Inhalt", "Contents"));
    let groups: [(&str, &[Chapter]); 4] = [
        (
            t("Physik der Radarwelle", "Physics of the radar wave"),
            &[Chapter::Wave, Chapter::Bands, Chapter::Antenna],
        ),
        (
            t("Auflösung und Reichweite", "Resolution and range"),
            &[Chapter::RangeRes, Chapter::BearingRes, Chapter::Range],
        ),
        (
            t("Bild und Bildfehler", "Picture and picture errors"),
            &[Chapter::FalseEchoes, Chapter::Signal, Chapter::Display],
        ),
        (
            t(
                "Bewegung, Vektoren und Plotten",
                "Motion, vectors and plotting",
            ),
            &[
                Chapter::Stabilisation,
                Chapter::RelVector,
                Chapter::Plotting,
                Chapter::SpeedTime,
            ],
        ),
    ];
    let accent = pal(ui).accent;
    for (g, chs) in groups {
        ui.add_space(6.0);
        ui.label(RichText::new(g).strong().color(accent));
        for c in chs {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .link(RichText::new(format!("{:>2}  {}", c.number(), c.title())).strong())
                    .clicked()
                {
                    *nav = Some(*c);
                }
                ui.label(RichText::new(format!("– {}", c.subtitle())).italics());
            });
        }
    }
    sub(
        ui,
        t(
            "Zusammenspiel mit den anderen Modi",
            "Working with the other modes",
        ),
    );
    text(
        ui,
        t(
            "Die Simulation zeigt alle Effekte im Zusammenspiel (Szenarien „Seegang & Regen“, „Küste“, „Zielmanöver“, „Strom“). Die Plottumgebung rechnet beliebige Begegnungen durch; aus dem Kapitel „Manuelles Plotten“ lässt sich eine Aufgabe direkt dorthin übernehmen. Gewechselt wird oben über „⟲ Modus“.",
            "The simulation shows all effects together (scenarios “sea & rain”, “coast”, “target manoeuvre”, “current”). The plotting environment works out any encounter; from the chapter “manual plotting” a task can be opened there directly. Switch via “⟲ Mode” at the top.",
        ),
    );
}
