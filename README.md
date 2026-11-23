# Radar / ARPA Trainer

Trainingssimulator für die Bedienung eines Schiffsradars, die Anwendung von ARPA
und grundlegende Radarfunktionen. Geschrieben in Rust (egui/eframe) – läuft nativ
auf **Windows**, **macOS** und Linux.

## Funktionen

**Radar**
- Rotierende Antenne (24 U/min) mit simuliertem Rohvideo, Nachleuchten, Strahl
- **X-Band / S-Band** umschaltbar: S-Band mit breiterer Keule, deutlich weniger See- und
  Regenclutter und kaum Regendämpfung, dafür schwächere Echos kleiner Ziele
- Bereiche 0,25 – 24 NM, Ringe, Pulslänge (Auto/SP/MP/LP)
- GAIN, SEA (STC), RAIN (FTC) mit realistischer Wirkung: Rauschen, Seegangsclutter
  (luvseitig stärker), Regenschauer mit Dämpfung, Nebenkeulen, Fremdradar-Interferenz (IR)
- Land mit Radarschatten (Ziele hinter Inseln unsichtbar), Radarhorizont je Zieltyp
- North-Up / Head-Up / Course-Up, Relativ- und True-Motion (mit TM-Reset), Off-Center
- Echospuren (Trails) wahr/relativ, 0:30 – 30:00 min in 30-s-Schritten
- Farbschemata: Blau (hell, Standard), Gelb, Grün, Orange; Helligkeit von Echos und Hintergrund getrennt einstellbar
- Bedienoberfläche Dunkel / Grau / Hell, im Programm wählbar (unabhängig vom System-Hell/Dunkel-Modus)
- 2 × EBL, 2 × VRM, Parallel-Index-Linie, Cursor-Peilung/-Distanz

**ARPA**
- Erfassung per Mausklick auf ein Echo, Erfassungsphase (~1 min), Trend bis 3 min
- Verfolgung bis 32 NM unabhängig vom eingestellten Bereich
- Plot-Extraktion mit Messrauschen, Regressionsfilter mit Manövererkennung
  (sichtbare ARPA-Verzögerung nach Kursänderungen der Ziele)
- Eigenes Menü „Vektoren & Trails“: wahre/relative Vektoren, Vektorlänge 0:30 – 60:00 min
  in 30-s-Schritten, eigener Vektor ein/aus, vergangene Positionen
- CPA/TCPA, BCR/BCT, KüG/FüG, Grund- bzw. Seestabilisierung
- Alarme: CPA/TCPA (gefährliches Ziel), verlorenes Ziel, Guard Zone mit Auto-Erfassung
- Guard Zone als Ring und/oder **Polygon** (per Mausklick gezeichnet, schiffsfest oder
  nordausgerichtet), jederzeit ausschalt- und löschbar
- Relativbahn mit CPA-Punkt für das gewählte Ziel
- **Trial-Manöver**: Probekurs/-fahrt mit Verzögerung, Trial-CPA je Ziel, Übernahme per Knopf

**Simulation / Ausbilder**
- Eigenschiff: Kurs direkt auf dem Schirm (Peilskala klicken/ziehen, Vektor-Endpunkt ziehen),
  Drehrate und Beschleunigung einstellbar; diese Autopilot-Bedienung lässt sich ausblenden (Taste A)
- Ziele hinzufügen über Dropdown (Typ, Peilung, Distanz, Kurs manuell / Kollisionskurs / CPA vorgeben)
  oder per Rechtsklick direkt auf dem Schirm; Kurs/Fahrt der Ziele jederzeit änderbar
- 10 Szenarien (Kreuzer, Begegnung, Überholen, Mehrschiff, Küste & Abschattung,
  Seegang & Regen, Fischereigebiet, Zielmanöver, Strom/Stabilisierung)
- Umwelt: Seegang 0–6, Windrichtung, Strom (Set/Drift), Regenschauer
- Zeitraffer 1× – 60×, Pause
- Ausbilderansicht: wahre Positionen/Vektoren zum Vergleich mit ARPA
- Bildschirmfoto (Knopf „Foto“ oder F12) als PNG in den Benutzerordner

## Startauswahl

Beim Start fragt das Programm nach der Arbeitsumgebung: **Simulation** (Radar mit ARPA)
oder **Plottumgebung** (klassisches Radarplotten) oder **Arbeitsbuch**. Das Haus-Symbol 🏠 oben
links führt nach einer Rückfrage zurück zur Auswahl – die laufende Sitzung wird dabei beendet.
Sprache und Oberfläche liegen im Zahnrad-Menü ⚙, in der Simulation zusätzlich Ausbilderansicht
(wahre Ziele), Zeitraffer und Hilfe;
💾 speichert die Einstellungen als Standard, ↺ setzt den Standard zurück.
Ziele setzt man über ➕ in der Kopfleiste oder per Rechtsklick „Ziel hier einsetzen …“: Typ,
Position (Peilung/Distanz oder Breite/Länge), Kurs (vorgegeben, Kollisionskurs oder mit
CPA-Vorgabe) und Fahrt, mit Vorschau von CPA/TCPA. Die Eigenschiffsposition wird im Bereich
„Eigenschiff“ gesetzt; Zieldaten und Cursor zeigen dazu Breite und Länge. Die Sprache (Deutsch/Englisch) folgt der Systemeinstellung und ist
im Programm umschaltbar.

## Arbeitsbuch

Der dritte Modus macht die wichtigsten Radarphänomene interaktiv erlebbar – in 13 Kapiteln
(plus Einführung) mit Reglern, Erklärung, Merke-Kasten, Querverweisen und Übungsfragen:

| Kapitel | Inhalt |
|---|---|
| 1 Welle & Frequenzbänder | f, λ, X/S-Band, Polarisation |
| 2 X- und S-Band | zwei Radare live nebeneinander (Regen, Seegang, Gain/STC/FTC), Antennenlänge für gleiche Keule |
| 3 Antenne & Radarkeule | Richtdiagramm kartesisch/polar, 70·λ/D, Nebenkeulen, Nah-/Fernfeld, Beam Steering |
| 4 Radiale Auflösung | ΔR = c·τ/2 am A-Scope, Bereich/Puls/PRF, Pulskompression mit Nebenzipfeln |
| 5 Azimutale Auflösung | Keulendurchlauf, Echoverbreiterung, ΔA = R·θ |
| 6 Reichweite | Radarhorizont 2,2·(√h₁+√h₂), Radargleichung (1/R⁴), PRF-Reichweite |
| 7 Mehrfach- und Falschechos | Mehrfach-, indirekte, Nebenkeulen- und Überreichweiten-Echos |
| 8 Signalverarbeitung | A-Scope-Kette Rohsignal → STC → FTC → Gain/Schwelle → A/D-Abtastung, Tune nach Kaltstart |
| 9 Darstellung | A-Scope/PPI, Head/North/Course Up, RM/TM, relative/wahre Trails, Kompassausfall |
| 10 Stabilisierung | wahre Vektoren über Grund vs. durchs Wasser, Strom an festen Objekten |
| 11 Relativvektor | Verhalten bei eigener Kursänderung, Vb = Va + Vbr, stehende Peilung |
| 12 Manuelles Plotten | 11 Schritte mit Rechnung und Lupe, Vorlagen „Beispiel“ und „Übung“, Übergabe an die Plottumgebung |
| 13 Weg–Zeit–Fahrt | eine Aufgabe, drei Hilfsmittel: Nomogramm (Lineal), Zirkelmethode auf der logarithmischen Skala, Rechenschieber nach Art des Portland Speed-Time-Distance Calculators (Sekunden/Minuten/Stunden); Zufallsaufgaben mit verdecktem Ergebnis; Rechenscheibe als Variante |

Am Ende jedes Kapitels stehen frei erfundene Übungsfragen mit Lösungsweg (Deutsch/Englisch);
die Antworten lassen sich am Modell des Kapitels nachvollziehen.

## Standard-Templates

- **Simulation** startet mit Szenario „Freies Fahren (leer)“, X-Band, 12 NM. 💾 (Als Standard)
  (zweite Zeile oben) speichert Radar-, ARPA-, Vektor-, Trail-, Darstellungs- und
  Eigenschiff-Einstellungen; das Szenario gehört nicht dazu, der Start erfolgt immer „leer“.
- **Plottumgebung** startet mit nur Ziel A und leeren Eingaben. 💾 (Als Standard) speichert die
  aktuellen Einstellungen inkl. Ziele; „Neu (Standard)“ lädt sie wieder.
- ↺ (Standard zurücksetzen) stellt jeweils die Werkseinstellung wieder her (die aktuelle Eingabe
  bleibt bis zum nächsten Start bzw. „Neu“ erhalten).
- Gespeichert wird als JSON unter `~/Library/Application Support/radar-arpa-trainer/` (macOS),
  `%APPDATA%\radar-arpa-trainer\` (Windows) bzw. `~/.config/radar-arpa-trainer/` (Linux).

## Plottumgebung

- Eigenschiff (Kurs, Fahrt, CPA-Grenze), Sicht: **in Sicht** (Regeln 11–18) oder **keine Sicht** (Regel 19)
- Ziele A, B, C … mit je zwei Beobachtungen (Zeit, Distanz, Peilung rw/rv);
  Beobachtungen auch per Rechtsklick auf das Plottblatt
- Ergebnisse: Δt, KBr, vBr, KB, vB, CPA, TCPA, tCPA, PCPA, SPCPA, BCR, BCT, tBC, Aspekt
- Darstellung: Relativbewegung mit CPA, Vektordreieck (W–O–A)
- Manöverplanung: Manöverpunkt nach Zeit oder Distanz; Kurs- oder Fahrtänderung für einen
  gewünschten CPA oder für einen vorgegebenen Kurs/Fahrt; neue Relativbewegung ab Manöverpunkt
  und parallel verschoben durch A (neuer Eigenvektor O′), Wiederaufnahme von Kurs/Fahrt (R),
  Auswirkung auf alle übrigen Ziele
- **Begründung**: Lagebeurteilung nach KVR (Regel 7, 8, 13–17 bzw. 19 inkl. 19(d)(i)/(ii)),
  Bewertung des geplanten Manövers mit ✔/⚠/✘
- **Cockcroft-Diagramm** als Overlay (vor dem Manöver) mit markiertem Sektor des gewählten Ziels,
  3-sm- und 4–6-sm-Ringen

## Bedienung Simulation (Kurzfassung)

| Aktion | Maus / Taste |
|---|---|
| Ziel erfassen / auswählen | Linksklick auf Echo |
| Sollkurs | Klick oder Ziehen auf der Peilskala, ← / → (Shift = 10°) |
| Sollkurs + Sollfahrt | grünen Vektor-Endpunkt ziehen, ↑ / ↓ für Fahrt |
| EBL / VRM verstellen | Linie bzw. Kreis ziehen |
| Trial-Manöver | Taste T, orangen Vektor ziehen |
| Autopilot-Anzeige ein/aus | Taste A |
| Guard-Zone-Polygon | „Polygon zeichnen“, Eckpunkte per Linksklick, Enter = fertig, Esc = abbrechen |
| Kontextmenü | Rechtsklick (Ziel platzieren, EBL/VRM, Off-Center, Guard Zone …) |
| Bereich | Mausrad, + / − |
| Pause / Zeitraffer | Leertaste, Tasten 1–7 |
| Track löschen / abwählen | Entf / Esc |
| Hilfe | H oder F1 |

## Bauen und starten

Voraussetzung: Rust-Toolchain (https://rustup.rs).

```bash
cargo run --release
```

Das fertige Programm liegt danach unter `target/release/radar-arpa-trainer`
(Windows: `target\release\radar-arpa-trainer.exe`) und kann ohne weitere Dateien
weitergegeben werden.

### Ohne Terminal starten

- **macOS:** `./scripts/build-macos-app.sh` erzeugt `dist/Radar ARPA Trainer.app` mit eigenem Icon.
  Per Doppelklick starten – es öffnet sich kein Terminal. Mit `--install` wird die App zusätzlich
  nach `/Applications` kopiert. (Eine aus GitHub geladene App ist nur ad-hoc signiert; beim ersten
  Start Rechtsklick → „Öffnen“.)
- **Windows:** Die Release-`.exe` (`cargo build --release`) startet ohne Konsolenfenster und trägt
  das Icon direkt in der Datei.
  
### Windows
1. Rust installieren (rustup-init.exe, Standardoptionen inkl. „Visual Studio Build Tools“).
2. Im Projektordner: `cargo build --release`
3. `target\release\radar-arpa-trainer.exe` starten (kein Konsolenfenster im Release-Build).

### macOS
1. Rust installieren (`curl https://sh.rustup.rs -sSf | sh`), ggf. Xcode Command Line Tools.
2. `cargo build --release`
3. `./target/release/radar-arpa-trainer`

Optional als `.app`-Bundle: `cargo install cargo-bundle && cargo bundle --release`.

### Linux (Debian/Ubuntu, Fedora)
1. Rust installieren (`curl https://sh.rustup.rs -sSf | sh`) und Bauabhängigkeiten:
   - Debian/Ubuntu: `sudo apt install build-essential libxkbcommon-dev libgl1-mesa-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`
   - Fedora: `sudo dnf install gcc libxkbcommon-devel mesa-libGL-devel wayland-devel libxcb-devel`
2. `cargo build --release` und `./target/release/radar-arpa-trainer`
3. Pakete: `cargo install cargo-deb cargo-generate-rpm`, dann `cargo deb` (`.deb`) bzw.
   `cargo build --release && cargo generate-rpm` (`.rpm`) – mit Startmenü-Eintrag und Icon.

### Automatische Builds
`.github/workflows/build.yml` baut bei jedem Push auf GitHub die Windows-`.exe` (mit Icon),
das macOS-App-Paket (`Radar-ARPA-Trainer-macOS.zip`) und die Linux-Pakete (`.deb` für
Debian/Ubuntu, `.rpm` für Fedora) und stellt sie unter Actions → Build → Artifacts bereit.
Installation unter Linux: `sudo apt install ./radar-arpa-trainer_*.deb` bzw.
`sudo dnf install ./radar-arpa-trainer-*.rpm`.

## Tests

```bash
cargo test --release
```

Ein Headless-Test lässt das Mehrschiff-Szenario 4 min laufen und prüft, dass
alle erfassten Ziele verfolgt werden und die ARPA-Schätzung zur Wahrheit passt.

## Projektstruktur

| Datei | Inhalt |
|---|---|
| `src/sim.rs` | Eigenschiff-/Zieldynamik, Umwelt, Szenarien |
| `src/radar.rs` | Radarvideo (Echos, Clutter, STC/FTC/Gain, Land), PPI-Rendering |
| `src/arpa.rs` | Tracking, CPA/TCPA, Alarme, Guard Zone, Trial |
| `src/app.rs` | Oberfläche, Mausbedienung, Anzeigen |
| `src/util.rs` | Vektor-/Winkelmathematik, Zufall, Rauschen |

Hinweis: Der Simulator dient der Ausbildung und ersetzt kein zugelassenes
Radar-/ARPA-Gerät oder eine vorgeschriebene Schulung.
