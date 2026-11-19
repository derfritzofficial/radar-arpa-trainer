//! Übungsfragen zu den Kapiteln – frei erfunden, vollständig lösbar und so
//! gewählt, dass sich die Antwort im jeweiligen Kapitel am Modell ausprobieren
//! lässt. Jede Frage gehört zu dem Kapitel, das in `chapters` zuerst steht;
//! weitere Einträge werden als Querverweise angezeigt.

use std::collections::HashSet;

use eframe::egui::{self, RichText};

use super::{pal, Chapter};
use crate::i18n::t;

pub struct Q {
    pub de: &'static str,
    pub en: &'static str,
    pub ans_de: &'static str,
    pub ans_en: &'static str,
    pub chapters: &'static [Chapter],
}

use Chapter as C;

pub const QUESTIONS: &[Q] = &[
    // ------------------------------------------------------------- 1 Welle
    Q { chapters: &[C::Wave],
        de: "Ein Radar sendet auf 9,375 GHz. Wie groß ist die Wellenlänge, und in welchem Band arbeitet es?",
        en: "A radar transmits at 9.375 GHz. What is the wavelength, and in which band does it operate?",
        ans_de: "λ = c / f = 3 · 10⁸ m/s ÷ 9,375 · 10⁹ Hz ≈ 0,032 m = 3,2 cm ⇒ X-Band (9,2–9,5 GHz). Im Modell: Frequenz auf 9,38 GHz stellen.",
        ans_en: "λ = c / f = 3 · 10⁸ m/s ÷ 9.375 · 10⁹ Hz ≈ 0.032 m = 3.2 cm ⇒ X-band (9.2–9.5 GHz). In the model: set the frequency to 9.38 GHz." },
    Q { chapters: &[C::Wave],
        de: "Ein S-Band-Radar arbeitet mit λ = 9,8 cm. Welche Frequenz hat es?",
        en: "An S-band radar works with λ = 9.8 cm. What is its frequency?",
        ans_de: "f = c / λ = 3 · 10⁸ m/s ÷ 0,098 m ≈ 3,06 · 10⁹ Hz = 3,06 GHz – also im S-Band (2,9–3,1 GHz).",
        ans_en: "f = c / λ = 3 · 10⁸ m/s ÷ 0.098 m ≈ 3.06 · 10⁹ Hz = 3.06 GHz – i.e. in the S-band (2.9–3.1 GHz)." },
    Q { chapters: &[C::Wave, C::RangeRes],
        de: "Wie viele Schwingungen enthält ein Puls von 0,08 µs bei 9,41 GHz, und wie lang ist er im Raum?",
        en: "How many oscillations does a 0.08 µs pulse at 9.41 GHz contain, and how long is it in space?",
        ans_de: "N = f · τ = 9,41 · 10⁹ Hz · 0,08 · 10⁻⁶ s ≈ 750 Schwingungen. Länge im Raum: c · τ = 300 m/µs · 0,08 µs = 24 m. Die halbe Pulslänge (12 m) ist zugleich die radiale Auflösung.",
        ans_en: "N = f · τ = 9.41 · 10⁹ Hz · 0.08 · 10⁻⁶ s ≈ 750 oscillations. Length in space: c · τ = 300 m/µs · 0.08 µs = 24 m. Half the pulse length (12 m) is also the range resolution." },
    Q { chapters: &[C::Wave, C::Signal],
        de: "Warum sind Schiffsradare in der Regel horizontal polarisiert?",
        en: "Why are marine radars usually horizontally polarised?",
        ans_de: "Eine horizontal polarisierte Welle wird von der Wasseroberfläche schwächer zurückgestreut als eine vertikal polarisierte. Der Seegangsclutter ist geringer, kleine Ziele im Nahbereich bleiben besser sichtbar.",
        ans_en: "A horizontally polarised wave is backscattered less by the sea surface than a vertically polarised one. Sea clutter is weaker, and small targets at short range stay more visible." },
    // ------------------------------------------------------------- 2 X/S-Band
    Q { chapters: &[C::Bands],
        de: "Die Rückstreuung von Regentropfen wächst etwa mit 1/λ⁴. Um welchen Faktor ist das Regenecho im X-Band (3,2 cm) stärker als im S-Band (10 cm)?",
        en: "Backscatter from raindrops grows roughly with 1/λ⁴. By what factor is the rain echo stronger in X-band (3.2 cm) than in S-band (10 cm)?",
        ans_de: "(10 / 3,2)⁴ = 3,125⁴ ≈ 95 – also rund 100-mal bzw. 20 dB stärker. Im Modell: Regenstärke hochdrehen – links (X) füllt sich der Schauer, rechts (S) bleibt er fast leer.",
        ans_en: "(10 / 3.2)⁴ = 3.125⁴ ≈ 95 – roughly 100 times or 20 dB stronger. In the model: turn up the rain – on the left (X) the shower fills up, on the right (S) it stays almost empty." },
    Q { chapters: &[C::Bands, C::Range],
        de: "Im Regenschauer voraus fährt eine Segelyacht. Im X-Band ist sie nicht zu sehen, im S-Band schon. Nennen Sie zwei Gründe.",
        en: "A sailing yacht is inside a rain shower ahead. It is invisible on X-band but visible on S-band. Give two reasons.",
        ans_de: "1) Der Regen streut im S-Band viel weniger zurück – das Yachtecho hebt sich vom Regenclutter ab. 2) Der Regen dämpft die S-Band-Welle auf Hin- und Rückweg kaum; im X-Band kommt im und hinter dem Schauer deutlich weniger Energie an.",
        ans_en: "1) Rain scatters much less back in S-band – the yacht's echo stands out from the rain clutter. 2) Rain hardly attenuates the S-band wave on the way out and back; in X-band much less energy reaches targets in and behind the shower." },
    Q { chapters: &[C::Bands, C::Antenna],
        de: "Ein X-Band-Radar mit 4-ft-Antenne soll durch ein S-Band-Radar mit gleicher Keulenbreite ergänzt werden. Wie lang muss die S-Band-Antenne sein (Rechnung mit λ = 3 cm bzw. 10 cm)?",
        en: "An X-band radar with a 4 ft antenna is to be complemented by an S-band radar with the same beamwidth. How long must the S-band antenna be (calculate with λ = 3 cm and 10 cm)?",
        ans_de: "Gleiche Keule ⇔ gleiches λ / D: D_S = D_X · λ_S / λ_X = 4 ft · 10 / 3 ≈ 13,3 ft (≈ 4,1 m). Im Rechner oben: S-Band-Antenne auf 13,5 ft stellen ⇒ D_X ≈ 4 ft.",
        ans_en: "Same beam ⇔ same λ / D: D_S = D_X · λ_S / λ_X = 4 ft · 10 / 3 ≈ 13.3 ft (≈ 4.1 m). In the calculator above: set the S-band antenna to 13.5 ft ⇒ D_X ≈ 4 ft." },
    Q { chapters: &[C::Bands],
        de: "Eine Rettungsinsel mit SART treibt in der Nähe. Welches Radar nutzen Sie, und woran erkennen Sie das Signal?",
        en: "A life raft with a SART is drifting nearby. Which radar do you use, and how do you recognise the signal?",
        ans_de: "Das X-Band-Radar – nur 9,2–9,5 GHz löst den SART aus. Er antwortet mit einer Linie aus 12 Punkten, die von seiner Position radial nach außen läuft; kommt man näher, werden daraus Bögen und schließlich Kreise.",
        ans_en: "The X-band radar – only 9.2–9.5 GHz triggers the SART. It answers with a line of 12 dots running radially outwards from its position; as you get closer they turn into arcs and finally circles." },
    // ------------------------------------------------------------- 3 Antenne
    Q { chapters: &[C::Antenna],
        de: "Schätzen Sie die Halbwertsbreite einer 1,8 m langen X-Band-Antenne (λ = 3,2 cm).",
        en: "Estimate the half-power beamwidth of a 1.8 m X-band antenna (λ = 3.2 cm).",
        ans_de: "θ ≈ 70 · λ / D = 70 · 0,032 m / 1,8 m ≈ 1,24°. Im Modell: D = 1,80 m und „X 9,41 GHz“.",
        ans_en: "θ ≈ 70 · λ / D = 70 · 0.032 m / 1.8 m ≈ 1.24°. In the model: D = 1.80 m and “X 9.41 GHz”." },
    Q { chapters: &[C::Antenna],
        de: "Ab welcher Entfernung beginnt das Fernfeld einer 2,4 m langen S-Band-Antenne (λ = 10 cm)? Was bedeutet das für sehr nahe Ziele?",
        en: "From what distance does the far field of a 2.4 m S-band antenna (λ = 10 cm) begin? What does this mean for very close targets?",
        ans_de: "R ≈ 2 · D² / λ = 2 · (2,4 m)² / 0,1 m ≈ 115 m. Näher ist die Keule noch nicht ausgebildet: Dort ist sie etwa so breit wie die Antenne selbst, und die Faustformel für θ gilt nicht.",
        ans_en: "R ≈ 2 · D² / λ = 2 · (2.4 m)² / 0.1 m ≈ 115 m. Closer in, the beam has not formed yet: there it is about as wide as the antenna itself, and the rule of thumb for θ does not apply." },
    Q { chapters: &[C::Antenna, C::FalseEchoes],
        de: "Was bewirkt ein Amplitudentaper (die Strahler zur Mitte hin stärker angesteuert)? Nennen Sie einen Vor- und einen Nachteil.",
        en: "What does an amplitude taper (elements driven more strongly towards the centre) do? Give one advantage and one drawback.",
        ans_de: "Vorteil: Die Nebenkeulen sinken deutlich (Kosinus-Taper ≈ −23 dB statt −13 dB) – weniger Nebenkeulenechos. Nachteil: Die Hauptkeule wird etwas breiter (≈ 70 · λ/D statt ≈ 51 · λ/D). Im Modell zwischen „gleichmäßig“ und „Kosinus-Taper“ umschalten.",
        ans_en: "Advantage: the side lobes drop markedly (cosine taper ≈ −23 dB instead of −13 dB) – fewer side-lobe echoes. Drawback: the main lobe gets slightly wider (≈ 70 · λ/D instead of ≈ 51 · λ/D). In the model switch between “uniform” and “cosine taper”." },
    Q { chapters: &[C::Antenna],
        de: "Ein Phased-Array-Radar soll seine Keule um 15° schwenken. Die Strahler stehen λ/2 auseinander. Welche Phasenverschiebung zwischen benachbarten Strahlern ist nötig?",
        en: "A phased-array radar is to steer its beam by 15°. The elements are λ/2 apart. Which phase shift between neighbouring elements is needed?",
        ans_de: "Δφ = 360° · d · sin θ / λ = 360° · ½ · sin 15° ≈ 46,6°. Jeder Strahler wird gegenüber seinem Nachbarn um diesen Winkel verzögert – die Wellenfront kippt, die Keule schwenkt ohne mechanische Drehung.",
        ans_en: "Δφ = 360° · d · sin θ / λ = 360° · ½ · sin 15° ≈ 46.6°. Each element is delayed by this angle relative to its neighbour – the wavefront tilts and the beam swings without mechanical rotation." },
    // ------------------------------------------------------------- 4 Radiale Auflösung
    Q { chapters: &[C::RangeRes],
        de: "Welche radiale Auflösung hat ein Radar mit 0,25 µs Pulslänge?",
        en: "What is the range resolution of a radar with a 0.25 µs pulse?",
        ans_de: "ΔR = c · τ / 2 = 150 m/µs · 0,25 µs = 37,5 m.",
        ans_en: "ΔR = c · τ / 2 = 150 m/µs · 0.25 µs = 37.5 m." },
    Q { chapters: &[C::RangeRes],
        de: "Zwei Tonnen liegen auf gleicher Peilung 60 m hintereinander. Bei welchen Pulslängen erscheinen sie getrennt: 0,05 µs, 0,3 µs, 0,8 µs?",
        en: "Two buoys lie 60 m apart on the same bearing. At which pulse lengths are they shown separately: 0.05 µs, 0.3 µs, 0.8 µs?",
        ans_de: "ΔR = 7,5 m, 45 m bzw. 120 m. Getrennt bei 0,05 µs und 0,3 µs (60 m > ΔR); bei 0,8 µs verschmelzen sie zu einem langen Echo. Im Modell: Abstand der Ziele 60 m und die Pulslänge variieren.",
        ans_en: "ΔR = 7.5 m, 45 m and 120 m. Separate at 0.05 µs and 0.3 µs (60 m > ΔR); at 0.8 µs they merge into one long echo. In the model: target separation 60 m, then vary the pulse length." },
    Q { chapters: &[C::RangeRes, C::FalseEchoes],
        de: "Ein Radar arbeitet mit einer Pulsfolgefrequenz von 1200 Hz. Bis zu welcher Entfernung misst es eindeutig?",
        en: "A radar works with a pulse repetition frequency of 1200 Hz. Up to what range does it measure unambiguously?",
        ans_de: "R = c / (2 · PRF) = 3 · 10⁸ m/s ÷ 2400 Hz = 125 km ≈ 67,5 sm. Echos weiter entfernter Ziele kommen erst nach dem nächsten Puls zurück (Überreichweiten).",
        ans_en: "R = c / (2 · PRF) = 3 · 10⁸ m/s ÷ 2400 Hz = 125 km ≈ 67.5 nm. Echoes from more distant targets only return after the next pulse (second-trace echoes)." },
    Q { chapters: &[C::RangeRes],
        de: "Ein Solid-State-Radar sendet einen 30-µs-Chirp mit 20 MHz Bandbreite. Welche radiale Auflösung erreicht es nach der Kompression, und wie groß wäre der Blindbereich ohne zusätzlichen Kurzpuls?",
        en: "A solid-state radar transmits a 30 µs chirp with 20 MHz bandwidth. What range resolution does it achieve after compression, and how large would the blind zone be without an additional short pulse?",
        ans_de: "Nach der Kompression: ΔR = c / (2 · B) = 150 m ÷ 20 = 7,5 m. Blindbereich: c · τ / 2 = 150 m/µs · 30 µs = 4500 m ≈ 2,4 sm – während des langen Pulses kann nicht empfangen werden. Deshalb sendet man für den Nahbereich zusätzlich einen kurzen Puls.",
        ans_en: "After compression: ΔR = c / (2 · B) = 150 m ÷ 20 = 7.5 m. Blind zone: c · τ / 2 = 150 m/µs · 30 µs = 4500 m ≈ 2.4 nm – nothing can be received while the long pulse is being sent. That is why a short pulse is added for short range." },
    // ------------------------------------------------------------- 5 Azimutale Auflösung
    Q { chapters: &[C::BearingRes],
        de: "Zwei Tonnen liegen 300 m auseinander, beide 6 sm entfernt. Die Keulenbreite beträgt 1,2°. Werden sie getrennt dargestellt? Ab welcher Entfernung verschmelzen sie?",
        en: "Two buoys are 300 m apart, both 6 nm away. The beamwidth is 1.2°. Are they shown separately? From what range do they merge?",
        ans_de: "Winkelabstand: 300 m / (6 · 1852 m) = 0,027 rad ≈ 1,55° > 1,2° ⇒ getrennt. Sie verschmelzen, sobald R · θ > 300 m: R > 300 m / 0,0209 ≈ 14 300 m ≈ 7,7 sm.",
        ans_en: "Angular separation: 300 m / (6 · 1852 m) = 0.027 rad ≈ 1.55° > 1.2° ⇒ separate. They merge once R · θ > 300 m: R > 300 m / 0.0209 ≈ 14,300 m ≈ 7.7 nm." },
    Q { chapters: &[C::BearingRes],
        de: "Wie breit erscheint eine einzelne Tonne in 8 sm bei 1,8° Keulenbreite?",
        en: "How wide does a single buoy appear at 8 nm with a 1.8° beamwidth?",
        ans_de: "ΔA ≈ R · θ = 8 · 1852 m · 0,0314 ≈ 465 m – rund ¼ sm breit, obwohl die Tonne nur wenige Meter misst.",
        ans_en: "ΔA ≈ R · θ = 8 · 1852 m · 0.0314 ≈ 465 m – about ¼ nm wide, although the buoy measures only a few metres." },
    Q { chapters: &[C::BearingRes, C::Bands],
        de: "Eine 200 m breite Hafeneinfahrt liegt 5 sm voraus. Welche Keulenbreite darf das Radar höchstens haben, damit die Einfahrt als Lücke erscheint? Schafft das eine 4-ft- bzw. eine 8-ft-X-Band-Antenne?",
        en: "A 200 m wide harbour entrance lies 5 nm ahead. What is the maximum beamwidth for the entrance to show as a gap? Does a 4 ft or an 8 ft X-band antenna manage it?",
        ans_de: "θ < 200 m / (5 · 1852 m) = 0,0216 rad ≈ 1,24°. 4 ft (≈ 1,8°): nein – die Molenköpfe verschmelzen, die Einfahrt wirkt geschlossen. 8 ft (≈ 0,9°): ja.",
        ans_en: "θ < 200 m / (5 · 1852 m) = 0.0216 rad ≈ 1.24°. 4 ft (≈ 1.8°): no – the pier heads merge and the entrance looks closed. 8 ft (≈ 0.9°): yes." },
    // ------------------------------------------------------------- 6 Reichweite
    Q { chapters: &[C::Range],
        de: "Ihre Antenne ist 20 m hoch. Wie weit reicht der Radarhorizont (Faktor 2,2)?",
        en: "Your antenna is 20 m high. How far is the radar horizon (factor 2.2)?",
        ans_de: "d = 2,2 · √20 ≈ 9,8 sm.",
        ans_en: "d = 2.2 · √20 ≈ 9.8 nm." },
    Q { chapters: &[C::Range],
        de: "Eine 80 m hohe Steilküste, Antennenhöhe 25 m: Aus welcher Entfernung kann die Küste theoretisch erfasst werden?",
        en: "A cliff 80 m high, antenna height 25 m: from what range can the coast theoretically be detected?",
        ans_de: "d = 2,2 · (√25 + √80) = 2,2 · (5 + 8,94) ≈ 30,7 sm. Im Modell: Knopf „Beispiel: Steilküste 80 m, Antenne 25 m“.",
        ans_en: "d = 2.2 · (√25 + √80) = 2.2 · (5 + 8.94) ≈ 30.7 nm. In the model: button “Example: cliff 80 m, antenna 25 m”." },
    Q { chapters: &[C::Range],
        de: "Um wie viel Prozent wächst die Detektionsreichweite, wenn die Sendeleistung verdreifacht wird?",
        en: "By what percentage does the detection range grow if the transmitter power is tripled?",
        ans_de: "Die Reichweite wächst nur mit der vierten Wurzel der Leistung: ⁴√3 ≈ 1,32 ⇒ etwa +32 %. Im Modell die Sendeleistung z. B. von 10 auf 30 kW stellen und die Detektionsgrenze vergleichen.",
        ans_en: "Range grows only with the fourth root of power: ⁴√3 ≈ 1.32 ⇒ about +32 %. In the model set the power e.g. from 10 to 30 kW and compare the detection limit." },
    Q { chapters: &[C::Range],
        de: "Hinter einer flachen Küste steigt landeinwärts ein Gebirge an. Warum kann das Radar die Küste zu weit entfernt anzeigen?",
        en: "Behind a low coast, mountains rise inland. Why can the radar make the coast appear too far away?",
        ans_de: "Die flache Küstenlinie liegt noch unter dem Radarhorizont, die Berge dahinter ragen schon darüber. Das erste Landecho stammt also von den Bergen – wer es für die Küste hält, schätzt den Abstand zum Land zu groß.",
        ans_en: "The low coastline is still below the radar horizon while the mountains behind already rise above it. The first land echo therefore comes from the mountains – mistaking it for the coast overestimates the distance to land." },
    // ------------------------------------------------------------- 7 Falschechos
    Q { chapters: &[C::FalseEchoes],
        de: "Ein großes Schiff passiert Sie querab in 0,4 sm. Wo können Mehrfachechos erscheinen?",
        en: "A large ship passes you abeam at 0.4 nm. Where can multiple echoes appear?",
        ans_de: "Auf derselben Peilung in 0,8 sm und 1,2 sm (2- und 3-fache Entfernung), nach außen schwächer. Sie verschwinden, sobald das Schiff nicht mehr querab steht. Im Modell: Abstand großes Schiff 0,4 sm.",
        ans_en: "On the same bearing at 0.8 nm and 1.2 nm (2× and 3× the range), weaker outwards. They vanish as soon as the ship is no longer abeam. In the model: range of large ship 0.4 nm." },
    Q { chapters: &[C::FalseEchoes],
        de: "Ein Ziel in 1,5 sm wird über Ihren Schornstein (Seitenpeilung 180°) indirekt angestrahlt. Wo erscheint das indirekte Echo?",
        en: "A target at 1.5 nm is illuminated indirectly via your funnel (relative bearing 180°). Where does the indirect echo appear?",
        ans_de: "In Richtung des Schornsteins (Seitenpeilung 180°, also im Schattensektor) und in etwa der richtigen Entfernung von 1,5 sm – der Umweg über den Schornstein ist kurz.",
        ans_en: "In the direction of the funnel (relative bearing 180°, i.e. in the shadow sector) and at roughly the correct range of 1.5 nm – the detour via the funnel is short." },
    Q { chapters: &[C::FalseEchoes, C::RangeRes],
        de: "PRF 1500 Hz, eine hohe Steilküste liegt in 60 sm. Wo kann sie als Überreichweitenecho erscheinen, und wie entlarven Sie es?",
        en: "PRF 1500 Hz, a high cliff lies at 60 nm. Where can it appear as a second-trace echo, and how do you expose it?",
        ans_de: "Eindeutige Reichweite c / (2 · PRF) = 3 · 10⁸ m/s ÷ 3000 Hz = 100 km ≈ 54,0 sm. Das Echo erscheint bei 60 − 54,0 ≈ 6,0 sm. Bei einem PRF-Wechsel (anderer Bereich, Jitter) springt es – echte Echos bleiben stehen. Im Modell: PRF 1500 Hz, fernes Ziel 60 sm.",
        ans_en: "Unambiguous range c / (2 · PRF) = 3 · 10⁸ m/s ÷ 3000 Hz = 100 km ≈ 54.0 nm. The echo appears at 60 − 54.0 ≈ 6.0 nm. When the PRF changes (other range scale, jitter) it jumps – real echoes stay put. In the model: PRF 1500 Hz, distant target 60 nm." },
    Q { chapters: &[C::FalseEchoes, C::Antenna],
        de: "Woran erkennen Sie Nebenkeulenechos, und was hilft dagegen?",
        en: "How do you recognise side-lobe echoes, and what helps against them?",
        ans_de: "Bögen in gleicher Entfernung beiderseits eines starken, nahen Echos (oft symmetrisch). Gain bzw. STC etwas zurücknehmen – die schwachen Nebenkeulenechos verschwinden zuerst.",
        ans_en: "Arcs at the same range on both sides of a strong, close echo (often symmetrical). Reduce gain or STC slightly – the weak side-lobe echoes disappear first." },
    // ------------------------------------------------------------- 8 Signalverarbeitung
    Q { chapters: &[C::Signal],
        de: "Ein Motorboot in 0,5 sm verschwindet im Seegangsclutter. Welchen Regler nutzen Sie, und was passiert, wenn Sie ihn zu weit aufdrehen?",
        en: "A motor boat at 0.5 nm disappears in sea clutter. Which control do you use, and what happens if you turn it up too far?",
        ans_de: "SEA (STC): Die Verstärkung wird im Nahbereich abgesenkt, der Clutter verschwindet, das stärkere Bootsecho bleibt. Zu viel STC löscht auch das Boot – der Nahbereich wird leergefegt. Im Modell: STC langsam erhöhen und am A-Scope verfolgen.",
        ans_en: "SEA (STC): gain is reduced at short range, the clutter vanishes and the stronger boat echo remains. Too much STC also wipes out the boat – the short range is swept empty. In the model: raise STC slowly and watch the A-scope." },
    Q { chapters: &[C::Signal],
        de: "Ein A/D-Wandler tastet mit 40 MHz ab. Welcher Entfernungsabstand liegt zwischen zwei Abtastwerten? Wie viele Stufen hat ein 8-Bit-Wandler?",
        en: "An A/D converter samples at 40 MHz. Which range interval lies between two samples? How many levels does an 8-bit converter have?",
        ans_de: "Δt = 1 / 40 MHz = 25 ns ⇒ ΔR = c · Δt / 2 = 150 m/µs · 0,025 µs = 3,75 m. 8 bit ⇒ 2⁸ = 256 Stufen.",
        ans_en: "Δt = 1 / 40 MHz = 25 ns ⇒ ΔR = c · Δt / 2 = 150 m/µs · 0.025 µs = 3.75 m. 8 bit ⇒ 2⁸ = 256 levels." },
    Q { chapters: &[C::Signal],
        de: "Warum braucht ein Solid-State-Radar keinen Tune-Regler?",
        en: "Why does a solid-state radar need no tune control?",
        ans_de: "Es gibt kein Magnetron, dessen Frequenz mit der Erwärmung driftet. Sende- und Empfangssignal werden aus demselben stabilen Oszillator abgeleitet (kohärent) – der Empfänger ist immer richtig abgestimmt.",
        ans_en: "There is no magnetron whose frequency drifts as it warms up. Transmit and receive signals are derived from the same stable oscillator (coherent) – the receiver is always correctly tuned." },
    Q { chapters: &[C::Signal, C::Bands],
        de: "Eine Yacht steckt in einem Regenschauer. Welcher Regler hilft, und warum erscheint die Yacht danach kürzer?",
        en: "A yacht is inside a rain shower. Which control helps, and why does the yacht then look shorter?",
        ans_de: "RAIN (FTC): Das Videosignal wird differenziert – von ausgedehnten Echos wie Regen bleiben nur die Vorderkanten, die Yacht hebt sich ab. Auch ihr Echo wird auf die Vorderflanke verkürzt und etwas schwächer.",
        ans_en: "RAIN (FTC): the video signal is differentiated – only the leading edges of extended echoes such as rain remain, and the yacht stands out. Its own echo is also cut down to the leading edge and becomes somewhat weaker." },
    // ------------------------------------------------------------- 9 Darstellung
    Q { chapters: &[C::Display],
        de: "North Up, RM, eigener Kurs 090°, 12 kn. Eine feste Tonne liegt 3 sm voraus. Wie sieht ihr relativer Trail nach 6 min aus – und ihr wahrer?",
        en: "North up, RM, own course 090°, 12 kn. A fixed buoy lies 3 nm ahead. What does its relative trail look like after 6 min – and its true trail?",
        ans_de: "Relativ bewegt sich die Tonne mit 12 kn nach 270°, also auf uns zu: Sie steht jetzt in 1,8 sm, der Trail zieht sich 1,2 sm hinter ihr nach Osten. Wahrer Trail: keiner – die Tonne steht. Im Modell: Sollkurs 90° und zwischen relativen und wahren Trails umschalten.",
        ans_en: "Relatively the buoy moves at 12 kn towards 270°, i.e. towards us: it is now at 1.8 nm, and the trail stretches 1.2 nm behind it to the east. True trail: none – the buoy is stationary. In the model: set course 90° and switch between relative and true trails." },
    Q { chapters: &[C::Display],
        de: "Warum verschmiert das Bild beim Gieren in Head Up, in North Up aber nicht?",
        en: "Why does the picture smear when yawing in head up, but not in north up?",
        ans_de: "In Head Up hängt das Bild am Vorausstrich: Jede Gierbewegung dreht alle Echos mit, zwischen zwei Umläufen landen sie an verschiedenen Stellen – das Bild verschmiert. North Up ist über den Kreisel stabilisiert, die Echos bleiben stehen. Im Modell: „Gieren ±4°“ einschalten.",
        ans_en: "In head up the picture is tied to the heading line: every yaw rotates all echoes, so between two sweeps they are painted in different places – the picture smears. North up is gyro-stabilised and the echoes stay still. In the model: switch on “yawing ±4°”." },
    Q { chapters: &[C::Display],
        de: "In True Motion erreicht das Eigenschiff den Rand des Schirms. Was passiert, und warum springt es nicht in die Mitte?",
        en: "In true motion own ship reaches the edge of the screen. What happens, and why does it not jump to the centre?",
        ans_de: "Es folgt ein Rücksprung (TM-Reset): Das Eigenschiff wird wieder in die Nähe der Mitte versetzt, aber entgegen der Fahrtrichtung verschoben (hier 3 sm hinter die Mitte) – so bleibt voraus möglichst viel Fläche sichtbar.",
        ans_en: "A TM reset follows: own ship is moved back near the centre, but offset against the direction of travel (here 3 nm behind the centre) – this keeps as much area as possible visible ahead." },
    Q { chapters: &[C::Display],
        de: "Sie fahren Course Up und ändern den Kurs um 40° nach Steuerbord. Was sehen Sie, und was ist danach zu tun?",
        en: "You are in course up and alter course by 40° to starboard. What do you see, and what do you do afterwards?",
        ans_de: "Das Bild bleibt stabilisiert stehen, nur der Vorausstrich dreht 40° nach rechts. Damit „voraus“ wieder oben ist, CU-Reset drücken – das Bild dreht sich einmalig um 40°.",
        ans_en: "The picture stays stabilised, only the heading line turns 40° to the right. To bring “ahead” back to the top, press CU reset – the picture rotates once by 40°." },
    // ------------------------------------------------------------- 10 Stabilisierung
    Q { chapters: &[C::Stabilisation],
        de: "Kurs 000°, 10 kn durchs Wasser, Strom 2 kn nach 090°. Welchen wahren Vektor zeigt eine feste Tonne bei See- und welchen bei Grundstabilisierung?",
        en: "Course 000°, 10 kn through the water, current 2 kn setting 090°. Which true vector does a fixed buoy show with sea and with ground stabilisation?",
        ans_de: "Seestabilisiert: 2 kn nach 270° – genau entgegen dem Strom, denn das Wasser zieht unter der Tonne weg. Grundstabilisiert: kein Vektor (0 kn), die Tonne steht über Grund.",
        ans_en: "Sea-stabilised: 2 kn towards 270° – exactly against the current, because the water moves away beneath the buoy. Ground-stabilised: no vector (0 kn), the buoy is stationary over ground." },
    Q { chapters: &[C::Stabilisation],
        de: "Kurs 000°, 10 kn durchs Wasser, Strom 2 kn nach 090°: Welchen Kurs und welche Fahrt über Grund macht das Eigenschiff?",
        en: "Course 000°, 10 kn through the water, current 2 kn setting 090°: which course and speed over ground does own ship make?",
        ans_de: "Vektoraddition: (0 | 10) + (2 | 0) kn ⇒ KüG = arctan(2/10) ≈ 011°, FüG = √(10² + 2²) ≈ 10,2 kn. Im Modell: HDG 0°, STW 10 kn, Strom 2 kn nach 90°.",
        ans_en: "Vector addition: (0 | 10) + (2 | 0) kn ⇒ COG = arctan(2/10) ≈ 011°, SOG = √(10² + 2²) ≈ 10.2 kn. In the model: HDG 0°, STW 10 kn, current 2 kn towards 90°." },
    Q { chapters: &[C::Stabilisation, C::Plotting],
        de: "Warum eignet sich für die Beurteilung des Aspekts nach KVR die Seestabilisierung besser?",
        en: "Why is sea stabilisation better suited for judging the aspect under the COLREGs?",
        ans_de: "Seestabilisierte wahre Vektoren zeigen die Bewegung durchs Wasser – ihre Richtung entspricht ungefähr der Vorausrichtung des anderen Schiffs, also seinem Aspekt. Grundstabilisierte Vektoren enthalten die Stromversetzung und können bei Querstrom einen falschen Aspekt vortäuschen.",
        ans_en: "Sea-stabilised true vectors show the motion through the water – their direction roughly matches the other ship's heading, i.e. its aspect. Ground-stabilised vectors include the drift due to current and can suggest a wrong aspect in a cross current." },
    // ------------------------------------------------------------- 11 Relativvektor
    Q { chapters: &[C::RelVector],
        de: "Eigenschiff 000°/12 kn, Fremdschiff 270°/12 kn. Welchen Relativkurs und welche Relativfahrt hat das Fremdschiff?",
        en: "Own ship 000°/12 kn, other ship 270°/12 kn. Which relative course and speed does the other ship have?",
        ans_de: "Vbr = Vb − Va = (−12 | 0) − (0 | 12) = (−12 | −12) kn ⇒ KBr = 225°, vBr = 12 · √2 ≈ 17,0 kn.",
        ans_en: "Vbr = Vb − Va = (−12 | 0) − (0 | 12) = (−12 | −12) kn ⇒ relative course 225°, relative speed 12 · √2 ≈ 17.0 kn." },
    Q { chapters: &[C::RelVector],
        de: "Ein Schiff kommt genau entgegen (180°, 10 kn), Sie fahren 000°/10 kn. Welcher Relativvektor ergibt sich – und welcher, wenn Sie auf 060° drehen?",
        en: "A ship approaches exactly head-on (180°, 10 kn); you steer 000°/10 kn. What is the relative vector – and what is it after you turn to 060°?",
        ans_de: "Vorher: Vbr = (0 | −10) − (0 | 10) = (0 | −20) ⇒ 180°, 20 kn (Kollisionskurs). Nach der Drehung: Va = (8,66 | 5,0) ⇒ Vbr = (−8,66 | −15,0) ⇒ 210°, 17,3 kn. Der Relativvektor schwenkt um 30° – das Schiff passiert Sie an Backbord.",
        ans_en: "Before: Vbr = (0 | −10) − (0 | 10) = (0 | −20) ⇒ 180°, 20 kn (collision course). After the turn: Va = (8.66 | 5.0) ⇒ Vbr = (−8.66 | −15.0) ⇒ 210°, 17.3 kn. The relative vector swings by 30° – the ship passes you to port." },
    Q { chapters: &[C::RelVector, C::Display],
        de: "Sie halbieren Ihre Fahrt, der Kurs bleibt gleich. Warum ändern sich trotzdem die Relativvektoren aller Ziele – auch der einer festen Tonne?",
        en: "You halve your speed and keep your course. Why do the relative vectors of all targets change – even that of a fixed buoy?",
        ans_de: "Vbr = Vb − Va hängt vom eigenen Vektor ab: Ändert sich Va, ändert sich jeder Relativvektor. Für die Tonne gilt Vbr = −Va – ihr Relativvektor zeigt weiter genau nach achtern, ist aber nur noch halb so lang.",
        ans_en: "Vbr = Vb − Va depends on own vector: if Va changes, every relative vector changes. For the buoy Vbr = −Va – its relative vector still points straight astern but is only half as long." },
    // ------------------------------------------------------------- 12 Plotten
    Q { chapters: &[C::Plotting],
        de: "Vorlage „Übung“: Eigenschiff 030°/13 kn. Um 10:00 steht ein Ziel in 8,0 sm auf 350°, um 10:06 in 6,8 sm auf 351°. Bestimmen Sie Relativbewegung, CPA/TCPA sowie Kurs und Fahrt des Ziels. Welche Lage nach KVR liegt vor?",
        en: "Template “exercise”: own ship 030°/13 kn. At 10:00 a target is at 8.0 nm bearing 350°, at 10:06 at 6.8 nm bearing 351°. Determine the relative motion, CPA/TCPA and the target's course and speed. Which COLREG situation is it?",
        ans_de: "KBr ≈ 164°, vBr ≈ 12,1 kn; CPA ≈ 0,8 sm um ≈ 10:40 (TCPA ≈ 34 min ab 10:06); das Ziel kreuzt ≈ 1,1 sm vor dem Bug. KB ≈ 092°, vB ≈ 9,8 kn. Kreuzende Kurse, Ziel an Backbord: In Sicht wäre es ausweichpflichtig und wir Kurshalter (Regeln 15 und 17); bei verminderter Sicht muss jedes Schiff nach Regel 19 handeln.",
        ans_en: "Relative course ≈ 164°, relative speed ≈ 12.1 kn; CPA ≈ 0.8 nm at ≈ 10:40 (TCPA ≈ 34 min from 10:06); the target crosses ≈ 1.1 nm ahead. Target course ≈ 092°, speed ≈ 9.8 kn. Crossing situation, target on the port side: in sight it would be the give-way vessel and we the stand-on vessel (rules 15 and 17); in restricted visibility every vessel must act under rule 19." },
    Q { chapters: &[C::Plotting],
        de: "Gleiche Lage bei verminderter Sicht: Welche Kursänderung um 10:12 ergibt einen CPA von 2 sm, und warum nach Steuerbord?",
        en: "Same situation in restricted visibility: which course alteration at 10:12 gives a CPA of 2 nm, and why to starboard?",
        ans_de: "Um 10:12 (Distanz ≈ 5,6 sm) auf Kurs ≈ 061° gehen, also ≈ 31° nach Steuerbord ⇒ CPA 2,0 sm um ≈ 10:58; den alten Kurs kann man um ≈ 11:02 wieder aufnehmen. Nach Regel 19 d) i) ist bei einem Fahrzeug vorlicher als querab eine Kursänderung nach Backbord zu vermeiden (außer beim Überholen).",
        ans_en: "At 10:12 (range ≈ 5.6 nm) come round to ≈ 061°, i.e. ≈ 31° to starboard ⇒ CPA 2.0 nm at ≈ 10:58; the old course can be resumed at ≈ 11:02. Under rule 19 (d)(i) an alteration of course to port for a vessel forward of the beam is to be avoided (other than for a vessel being overtaken)." },
    Q { chapters: &[C::Plotting],
        de: "Ändern Sie in der Vorlage „Übung“ nur die zweite Peilung um 1° auf 352°. Wie stark ändert sich der CPA – und was folgt daraus?",
        en: "In the template “exercise” change only the second bearing by 1° to 352°. How much does the CPA change – and what follows from this?",
        ans_de: "Der CPA springt von ≈ 0,8 sm auf ≈ 1,5 sm. Der Relativweg in 6 min (≈ 1,2 sm) wird bis zum CPA mehr als fünffach verlängert – kleine Mess- und Ablesefehler wirken stark. Deshalb genau messen, länger beobachten bzw. mehrere Peilungen ausgleichen und mit Sicherheitsreserve planen.",
        ans_en: "The CPA jumps from ≈ 0.8 nm to ≈ 1.5 nm. The relative track over 6 min (≈ 1.2 nm) is extended more than five times up to the CPA – small measuring and reading errors have a large effect. Hence measure carefully, observe longer or average several bearings, and plan with a safety margin." },
    Q { chapters: &[C::Plotting, C::SpeedTime],
        de: "Warum beobachtet man beim Plotten gern im Abstand von 6 Minuten?",
        en: "Why are 6-minute intervals popular for plotting?",
        ans_de: "6 min sind eine Zehntelstunde: Der Relativweg in sm mal 10 ist direkt die Relativfahrt in kn (z. B. 1,2 sm ⇒ 12 kn). Das macht die Rechnung im Kopf möglich.",
        ans_en: "6 min are a tenth of an hour: the relative track in nm times 10 is directly the relative speed in kn (e.g. 1.2 nm ⇒ 12 kn). This allows mental arithmetic." },
    // ------------------------------------------------------------- 13 Weg – Zeit – Fahrt
    Q { chapters: &[C::SpeedTime],
        de: "Ein Ziel legt relativ in 6 min 1,7 sm zurück. Wie groß ist die Relativfahrt?",
        en: "A target covers 1.7 nm relatively in 6 min. What is the relative speed?",
        ans_de: "6-Minuten-Regel: 1,7 sm · 10 = 17 kn.",
        ans_en: "6-minute rule: 1.7 nm · 10 = 17 kn." },
    Q { chapters: &[C::SpeedTime, C::Plotting],
        de: "Bis zum CPA sind es auf der Relativbahn noch 4,2 sm, die Relativfahrt beträgt 14 kn. Wie groß ist die TCPA?",
        en: "Along the relative track 4.2 nm remain to the CPA; the relative speed is 14 kn. What is the TCPA?",
        ans_de: "t = 60 · 4,2 / 14 = 18 min. Rechenschieber: 14 unter den Pfeil ▼ stellen; über 4,2 auf der Schieberskala steht 18 auf der MINUTES-Skala.",
        ans_en: "t = 60 · 4.2 / 14 = 18 min. Slide rule: set 14 under the ▼ arrow; above 4.2 on the slide scale you read 18 on the MINUTES scale." },
    Q { chapters: &[C::SpeedTime],
        de: "Wie weit kommen Sie mit 16 kn in 2 h 45 min? Lösen Sie die Aufgabe mit dem Rechenschieber.",
        en: "How far do you get at 16 kn in 2 h 45 min? Solve it with the slide rule.",
        ans_de: "16 kn unter den Pfeil ▼ bei 60 MINUTES stellen. Auf der HOURS-Skala 2¾ aufsuchen und darüber auf der unteren Schieberskala 44 ablesen ⇒ 44 sm (16 · 2,75 = 44).",
        ans_en: "Set 16 kn under the ▼ arrow at 60 MINUTES. Find 2¾ on the HOURS scale and read 44 above it on the lower slide scale ⇒ 44 nm (16 · 2.75 = 44)." },
    Q { chapters: &[C::SpeedTime],
        de: "Zirkelmethode: Bestimmen Sie den Weg bei 9 kn in 40 min. Beschreiben Sie die Schritte.",
        en: "Dividers method: find the distance at 9 kn in 40 min. Describe the steps.",
        ans_de: "1) Den Zirkel auf der logarithmischen Skala zwischen 60 und 9 öffnen. 2) Die Öffnung beibehalten und die Spitze, die auf 60 stand, auf 40 setzen. 3) Die andere Spitze zeigt 6 ⇒ 6,0 sm (9 · 40 / 60 = 6).",
        ans_en: "1) Open the dividers on the logarithmic scale between 60 and 9. 2) Keep the spread and move the point that was on 60 to 40. 3) The other point shows 6 ⇒ 6.0 nm (9 · 40 / 60 = 6)." },
    Q { chapters: &[C::SpeedTime],
        de: "Wie lange braucht ein Schiff mit 7,5 kn für 2,5 sm?",
        en: "How long does a ship at 7.5 kn take for 2.5 nm?",
        ans_de: "t = 60 · 2,5 / 7,5 = 20 min.",
        ans_en: "t = 60 · 2.5 / 7.5 = 20 min." },
];

fn question_card(
    ui: &mut egui::Ui,
    id: &str,
    q: &Q,
    revealed: &mut HashSet<&'static str>,
    nav: &mut Option<Chapter>,
    here: Chapter,
) {
    let p = pal(ui);
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{id}  ")).strong().color(p.accent));
            ui.label(RichText::new(t(q.de, q.en)).strong());
        });
        let open = revealed.contains(q.de) || super::demo_flag("answers");
        ui.horizontal_wrapped(|ui| {
            if ui
                .small_button(if open {
                    t("Lösung ausblenden", "Hide answer")
                } else {
                    t("Lösung zeigen", "Show answer")
                })
                .clicked()
            {
                if open {
                    revealed.remove(q.de);
                } else {
                    revealed.insert(q.de);
                }
            }
            for c in q.chapters.iter().filter(|c| **c != here) {
                if ui.link(format!("→ {} {}", c.number(), c.title())).clicked() {
                    *nav = Some(*c);
                }
            }
        });
        if open {
            ui.label(RichText::new(t(q.ans_de, q.ans_en)).color(p.ink));
        }
    });
}

/// Übungsfragen eines Kapitels (am Ende des Kapitels).
pub fn chapter_ui(
    ui: &mut egui::Ui,
    ch: Chapter,
    revealed: &mut HashSet<&'static str>,
    nav: &mut Option<Chapter>,
) {
    let qs: Vec<&Q> = QUESTIONS
        .iter()
        .filter(|q| q.chapters.first() == Some(&ch))
        .collect();
    if qs.is_empty() {
        return;
    }
    super::sub(ui, t("🎓 Übungsfragen", "🎓 Practice questions"));
    for (i, q) in qs.iter().enumerate() {
        question_card(ui, &format!("{}.{}", ch.number(), i + 1), q, revealed, nav, ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_chapter_has_questions_and_texts_are_complete() {
        let mut seen = HashSet::new();
        for q in QUESTIONS {
            assert!(seen.insert(q.de), "doppelte Frage: {}", q.de);
            assert!(!q.chapters.is_empty(), "ohne Kapitel: {}", q.de);
            for s in [q.de, q.en, q.ans_de, q.ans_en] {
                assert!(s.len() > 10, "Text fehlt bei: {}", q.de);
            }
        }
        for ch in Chapter::ALL.iter().filter(|c| **c != Chapter::Intro) {
            let n = QUESTIONS.iter().filter(|q| q.chapters[0] == *ch).count();
            assert!(n >= 3, "Kapitel {:?} hat nur {n} Übungsfragen", ch);
        }
    }
}
