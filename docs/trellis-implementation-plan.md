# Trellis – Implementációs terv

**Dátum:** 2026-02-13
**Alapdokumentumok:** trellis-spec.md (v0.3), trellis-project-structure.md

---

## Elvek

- Minden **mérföldkő (M)** végén a projekt **lefordul és futtatható**
- A mérföldkövek egymásra épülnek – a sorrend kötött
- Minden mérföldkő végén van egy **smoke test**: a CLI-vel futtatható bemenet → kimenet
- A checkbox-ok ( `[ ]` ) jelölik az egyes feladatok állapotát

---

## M1 – Projekt váz és minimális pipeline (skeleton)

> **Cél:** A monorepo felépül, a 4 crate lefordul, a CLI "hello world" szinten fut.
> **Smoke test:** `cargo run -p trellis-cli -- render test.mmd -o test.svg` → üres/placeholder SVG

* [x] Cargo workspace inicializálás (`Cargo.toml` a gyökérben, `crates/` alkönyvtárak)
* [x] `trellis-parser` crate váz (`lib.rs`, `ast.rs` – üres `Graph`, `Node`, `Edge`, `Subgraph` struktúrák)
* [x] `trellis-core` crate váz (`lib.rs`, `types.rs`, `config.rs` – `TrellisConfig` struktúra, `pipeline.rs` – üres `render()` függvény)
* [x] `trellis-wasm` crate váz (`lib.rs` – `wasm-bindgen` `render()` stub)
* [x] `trellis-cli` crate váz (`main.rs` – `clap` CLI, `render` parancs, placeholder kimenet)
* [x] `cargo test --workspace` – minden crate lefordul, üres tesztek zöldek
* [x] `cargo run -p trellis-cli -- --version` kiírja a verziószámot
* [x] Benchmark fixture fájlok létrehozása (`tests/benchmarks/fixtures/b01..b12.mmd`)

---

## M2 – Mermaid parser (Fázis 1) – Flowchart

> **Cél:** A parser flowchart szintaxist feldolgozza és `Graph` AST-t ad vissza.
> **Smoke test:** `cargo run -p trellis-cli -- validate test.mmd` → "OK, 5 node, 4 edge"

* [x] Tokenizer (`tokenizer.rs`) – soronkénti feldolgozás, kulcsszavak felismerése
* [x] Flowchart parser (`flowchart.rs`) – `graph`/`flowchart` direktíva, `direction` kiolvasás (TB/BT/LR/RL)
* [x] Csomópont-definíció parsing (id, label, shape: `[]`, `()`, `{}`, `(())`, stb.)
* [x] Él-definíció parsing (`-->`, `---`, `-.->`, `==>`, `--text-->`, stb.)
* [x] Subgraph parsing (`subgraph id` ... `end` blokkok, egymásba ágyazás)
* [x] Szövegméret becslés (`calculateTextWidth`, `calculateTextHeight` – font metrika nélkül, karakter-alapú heurisztika)
* [x] `validate` CLI parancs implementálás (parser hívás + összesítő kiírás)
* [x] Unit tesztek: B01 (linear chain), B02 (wide branch), B04 (diamond), B07 (cycle), B08 (nested subgraph)

---

## M3 – Csomópont-elhelyezés (Fázis 2) – Sugiyama flowchart-hoz

> **Cél:** A flowchart csomópontjai koordinátákat kapnak a Sugiyama algoritmussal.
> **Smoke test:** `cargo run -p trellis-cli -- render b01.mmd --metrics` → `{"nodes": 5, "layers": 5, ...}` (SVG még placeholder)

* [x] Ciklustörés (`placement/sugiyama.rs` – DFS-alapú `breakCycles`)
* [x] Réteg-hozzárendelés (`assignLayers` – longest path algoritmus)
* [x] Rétegsorrend optimalizálás (`orderWithinLayers` – barycenter heurisztika, iteratív)
* [x] Koordináta-hozzárendelés (`assignCoordinates` – direction-függő: TB/BT/LR/RL, centrálás)
* [x] Grid-re kerekítés (`snap.rs` – `snapToGrid`, spirális keresés ütközésnél)
* [x] Pipeline integráció: `pipeline.rs` meghívja a placement-et, koordinátákat debug kiírás (JSON `--metrics`)
* [x] Unit tesztek: rétegezés helyessége, barycenter konvergencia, ciklustörés

---

## M4 – Grid felépítés + Port-kiosztás (Fázis 3-4)

> **Cél:** A routing rács felépül, a portok kiosztásra kerülnek.
> **Smoke test:** `--metrics` kiírja a grid méretét és port-kiosztást

* [x] Cellaméret számítás (`grid/params.rs` – `calculateCellSize`, sűrűség-alapú)
* [x] Rács kiterjedés számítás (`calculateGridExtent` – biztonsági szorzó)
* [x] Grid felépítés (`grid/builder.rs` – `buildGrid`, csomópontok alatti cellák blokkolása)
* [x] Szög→oldal konverzió (`ports/assignment.rs` – `angleToSide`, 4 szektor)
* [x] Élek oldalankénti csoportosítása (szomszédok irányából)
* [x] Túlcsordulás kezelés (`ports/overflow.rs` – szomszédos oldalra áthelyezés)
* [x] Oldalon belüli rendezés (`sortEdgesOnSide` – merőleges tengely szerinti sorrend)
* [x] Port pozíciók kiszámítása (`ports/positions.rs` – egyenletes elosztás az oldalon)
* [x] Unit tesztek: grid méret B01-B05 fixture-ökre, port szimmetria ellenőrzés

---

## M5 – Él-routing: A* pathfinding (Fázis 5-6)

> **Cél:** Az élek ortogonális útvonalat kapnak a rácson. Ez az első mérföldkő ahol valódi SVG kimenet születik.
> **Smoke test:** `cargo run -p trellis-cli -- render b01.mmd -o b01.svg` → megtekinthető SVG élek nélkül... nem: **élekkel**!

* [x] Routing prioritás számítás (`routing/priority.rs` – fokszám + távolság + torlódás hibrid pontozás)
* [x] A* implementáció (`routing/astar.rs` – `routeEdge`, `PriorityQueue`, `heuristic`)
* [x] Költségfüggvény (`routing/cost.rs` – `BASE_COST`, `BEND_COST`, `ADJACENT_COST`, `CROSSING_COST`, `BLOCKED_COST`)
* [x] 4-irányú szomszédság (`getNeighbors` – ortogonális mozgás)
* [x] Út lefoglalás (`routing/commit.rs` – `commitPath`, szomszédos cellák költségnövelése)
* [x] Többszörös élek detektálása (`routing/multi_edge.rs` – `detectMultiEdges`, `canonicalKey`)
* [x] Többszörös élek routing-ja (`routeMultiEdges` – szomszédos portok + önálló A*)
* [x] Teljes routing ciklus (`routeAllEdges` – prioritás szerinti sorrend, multi-edge kezelés)
* [x] Unit tesztek: B01 (egyenes út), B03 (K₃,₃ – keresztezés elkerülhetetlen), B06 (multi-edge)

---

## M6 – SVG rendering (Fázis 9) – alapvető kimenet

> **Cél:** Működő SVG kimenet: csomópontok + élek + nyilak. Az első "vizuálisan értékelhető" eredmény.
> **Smoke test:** `trellis render b04.mmd -o b04.svg` → böngészőben megnyitva olvasható diagram

* [x] Csomópont renderelés (`render/nodes.rs` – rect, rounded, diamond + címke SVG)
* [x] Útvonal egyszerűsítés (`render/edges.rs` – `simplifyPath`, collinear pontok eltávolítása)
* [x] Lekerekített sarkok (`generateRoundedPolyline` – SVG `Q` quadratic Bezier)
* [x] Él renderelés (`render/edges.rs` – polyline generálás, stroke stílusok: solid/dotted/thick)
* [x] Nyílhegy renderelés (SVG marker def + irány-számítás)
* [x] SVG dokumentum összeállítás (`render/svg.rs` – `buildSVG`, viewBox, style blokk)
* [x] PNG export (`render/png.rs` – `resvg` SVG→PNG konverzió)
* [x] CLI `render` parancs teljes bekötése (parser → placement → grid → ports → routing → SVG → fájl)
* [x] CLI `-f png` és `-f svg` kapcsoló működés
* [ ] Vizuális ellenőrzés: B01–B07 fixture-ök renderelése és manuális átnézés

---

## M7 – Zsákutca kezelés (Fázis 7)

> **Cél:** Ha egy él nem routeolható, a rendszer automatikusan megoldja.
> **Smoke test:** B03 (K₃,₃) hiba nélkül renderelődik, esetleg keresztezéssel

* [ ] Rip-up and reroute (`deadlock/rip_up.rs` – blokkoló élek azonosítása, visszavonás, újra-routing)
* [ ] Blokkoló élek azonosítása (`findBlockingEdges` – módosított A* tracking-gel)
* [ ] Rács növelés (`deadlock/expand.rs` – `expandGridAndRetry`, 1.5x skálázás, teljes újra-routing)
* [ ] Keresztezéses fallback (`deadlock/fallback.rs` – OCCUPIED cellák véges költséggel átjárhatók)
* [ ] Keresztezési pontok megjelölése (`cell.crossing = true`)
* [ ] Keresztezési híd renderelés (`render/crossing.rs` – fehér háttér kör + ív SVG)
* [ ] `handleDeadlock` 3-szintű védelem bekötése a routing ciklusba
* [ ] Unit tesztek: szándékosan zsúfolt gráf ahol rip-up szükséges

---

## M8 – Él címke elhelyezés (Fázis 8)

> **Cél:** Az élek szöveges címkéi megjelennek, ütközésmentesen.
> **Smoke test:** Címkés élek olvashatóan jelennek meg a fixture-ökben

* [ ] Szegmens kiválasztás (`labels/placement.rs` – `selectBestSegment`, középső > leghosszabb)
* [ ] Pozíció-jelöltek generálása (vízszintes: fölötte/alatta, függőleges: balra/jobbra)
* [ ] Ütközés-detektálás (`labels/collision.rs` – csomópontokkal, élekkel, más címkékkel)
* [ ] Eltolás a szegmens mentén (`labels/slide.rs` – középponttól kifelé keresés)
* [ ] Fallback pozíció (ha minden ütközik: eredeti középpont)
* [ ] Címke SVG renderelés (háttér téglalap + szöveg)
* [ ] Teljes pipeline bekötés: routing után, SVG renderelés előtt
* [ ] Unit tesztek: címke-ütközés szcenáriók

---

## M9 – Subgraph kezelés (Fázis 2 kiegészítés)

> **Cél:** Subgraph-ok vizuálisan csoportosítva jelennek meg, kerettel és címkével.
> **Smoke test:** B08 (nested subgraph) és B09 (subgraph edges) renderelődik

* [ ] Subgraph fa felépítése (`placement/subgraph.rs` – `buildSubgraphTree`, szülő-gyerek mapping)
* [ ] Rekurzív elhelyezés bottom-up (`placeSubgraphRecursive` – virtuális csomópontok, lokális→globális offset)
* [ ] Bounding box számítás (padding + label height)
* [ ] Subgraph élek feloldása (`resolveSubgraphEdges` – virtuális csomópontok, `blocksGrid: false`)
* [ ] Subgraph keret renderelés (`render/subgraph.rs` – szaggatott keret, háttérszín mélység szerint, címke)
* [ ] Z-order: subgraph háttér → élek → csomópontok → címkék
* [ ] Unit tesztek: B08 (nested), B09 (subgraph edges)

---

## M10 – Class diagram + ER diagram parser és elhelyezés

> **Cél:** A három fókusz diagramtípus mindegyike működik.
> **Smoke test:** B12 (class hierarchy) és B11 (100 node ER) renderelődik

* [ ] Class diagram parser (`class_diagram.rs` – osztályok, metódusok, relációk: extends/implements/association)
* [ ] Class diagram elhelyezés (`placement/class.rs` – hibrid: öröklődési fa Sugiyama + asszociáció laterális)
* [ ] ER diagram parser (`er_diagram.rs` – entitások, attribútumok, relációk: 1:1, 1:N, N:M)
* [ ] ER diagram elhelyezés (`placement/er.rs` – Fruchterman-Reingold force-directed)
* [ ] `detectType` frissítés a tokenizer-ben (flowchart/classDiagram/erDiagram felismerés)
* [ ] Pipeline routing: diagramtípus-függő elhelyezés-választás (`placeByDiagramType`)
* [ ] Unit tesztek: B11 (ER), B12 (class) fixture-ök

---

## M11 – Dekompozíció (Fázis 13)

> **Cél:** Nagy gráfok (50+ node) kezelése klaszterezéssel.
> **Smoke test:** B10 (50 node flowchart) és B11 (100 node ER) elfogadható idő alatt renderelődik

* [ ] `TrellisConfig` dekompozíciós mezők: `decomposition`, `decompositionThreshold`
* [ ] Louvain közösségdetektálás (`decomposition/clustering.rs` – modularitás-alapú klaszterezés)
* [ ] Hibrid klaszterezés (subgraph-ok tisztelete + Louvain a maradékra)
* [ ] SINGLE mód (`decomposition/single.rs` – klaszterenkénti belső routing + globális rács + klaszterek közti routing)
* [ ] MULTI mód (`decomposition/multi.rs` – összesítő diagram + klaszterenkénti részletek, `DiagramSet` kimenet)
* [ ] Pipeline elágazás (`renderDiagram` – NONE/SINGLE/MULTI mód-választás)
* [ ] CLI `--decomposition` és `--decomposition-threshold` kapcsolók bekötése
* [ ] Integrációs tesztek: B10, B11 fixture-ök mindhárom módban

---

## M12 – CLI teljes funkciókészlet

> **Cél:** Az összes CLI parancs működik, beleértve batch módot, preprocessort, és licenckezelést.
> **Smoke test:** `trellis preprocess doc.md -o out.md --image-dir img/` → Mermaid blokkok képekre cserélve

* [ ] `render` stdin mód (`trellis render - -f svg` – pipe-olható)
* [ ] `render-batch` parancs (könyvtár bejárás, .mmd szűrés, párhuzamos renderelés)
* [ ] `validate` parancs (parser + összesítő, szintaxis hibák kiírása)
* [ ] `preprocess` parancs (markdown Mermaid blokkok → képhivatkozások)
* [ ] `license --activate` / `--status` / `--deactivate` (Lemon Squeezy API kliens, `~/.trellis/license.json`)
* [ ] Licenc ellenőrzés a `render` és `render-batch` parancsokban (10 node limit, PNG-only, vízjel)
* [ ] `--metrics` JSON kimenet stderr-re (crossings, bends, render_ms, grid_utilization)
* [ ] `--config` fájl betöltés (`~/.trellis/config.toml`)
* [ ] Pandoc Lua filter (`filters/trellis-filter.lua`) megírása és tesztelése
* [ ] Exit kódok: 0 = OK, 1 = parse hiba, 2 = render hiba, 3 = licenc hiba

---

## M13 – WASM build + VS Code extension

> **Cél:** A VS Code extension működik: .mmd fájl megnyitás → preview panel → renderelt diagram.
> **Smoke test:** VS Code-ban F5 → .mmd fájl megnyitva → preview panelen renderelt diagram

* [ ] `trellis-wasm` crate: `render()` és `render_with_metrics()` wasm-bindgen API
* [ ] `scripts/build-wasm.sh` – wasm-pack build (web target)
* [ ] VS Code extension váz (`package.json`, `tsconfig.json`, `extension.ts`)
* [ ] WASM bridge (`wasm-bridge.ts` – WASM betöltés, `render()` hívás)
* [ ] Preview panel (`preview.ts` – webview, SVG megjelenítés)
* [ ] Preview webview template (`media/preview.html`)
* [ ] Fájl mentés figyelés (`.mmd` fájl változáskor automatikus újra-render)
* [ ] Freemium korlátok (`license.ts` – 10 node limit, vízjel, PNG-only export)
* [ ] Export funkció (`export.ts` – PNG/SVG mentés)
* [ ] `vsce package` → `.vsix` fájl előállítás

---

## M14 – IntelliJ plugin

> **Cél:** IntelliJ plugin működik: .mmd fájl → tool window → renderelt diagram.
> **Smoke test:** IntelliJ-ben Run Plugin → .mmd fájl → preview

* [ ] `scripts/build-wasm.sh` kiegészítés bundler targettel (IntelliJ-hez)
* [ ] Plugin váz (`build.gradle.kts`, `plugin.xml`, `TrellisPlugin.kt`)
* [ ] WASM bridge Chicory runtime-mal (`WasmBridge.kt`)
* [ ] Preview tool window (`PreviewPanel.kt` – JCEF/SVG megjelenítés)
* [ ] Fájl változás figyelés (IntelliJ VFS listener)
* [ ] JetBrains Marketplace licencelés (`LicenseManager.kt` – `LicensingFacade`)
* [ ] Export (`ExportAction.kt` – PNG/SVG)
* [ ] `./gradlew buildPlugin` → `.zip` fájl

---

## M15 – Docker + CI/CD

> **Cél:** Docker image és GitHub Actions CI pipeline működik.
> **Smoke test:** `docker run --rm -v $(pwd):/data ghcr.io/trellis/trellis:latest trellis render /data/test.mmd -o /data/test.svg`

* [ ] `docker/Dockerfile` – multi-stage build (builder + slim runtime)
* [ ] `scripts/build-docker.sh` – image build + tag
* [ ] `scripts/build-cli.sh` – cross-compile: Linux x86_64, macOS x86_64/ARM, Windows
* [ ] GitHub Actions CI workflow: `cargo test` → `cargo bench` → WASM build → CLI build → Docker build
* [ ] GitHub Actions: artifact upload (.vsix, .zip, CLI binárísok, Docker push)
* [ ] GitHub Releases automatizáció (tag push → release draft + binárisok)
* [ ] README.md (telepítés, használat, példák, badge-ek)

---

## M16 – Benchmark és finomhangolás

> **Cél:** A költségfüggvény konstansai optimalizálva, a teljesítmény mérve és dokumentálva.
> **Smoke test:** `cargo bench` lefut, eredmények a `target/criterion/` alatt, minden benchmark elfogadható idő alatt renderelődik

* [ ] Criterion benchmark runner (`tests/benchmarks/src/bench.rs`) – B01-B12 fixture-ök
* [ ] Invariáns tesztek (`tests/integration/invariants.rs`):
    * [ ] Fedésmentesség: nincs két él ugyanazon a rácsponton
    * [ ] Blokkolás: nincs él csomóponton átmenő rácsponton
    * [ ] Ortogonalitás: minden él csak vízszintes/függőleges szegmensekből áll
    * [ ] Összefüggőség: minden él összefüggő útvonal a portok között
    * [ ] Port egyediség: nincs két él ugyanazon a porton
* [ ] Költségfüggvény finomhangolás: `BEND_COST`, `ADJACENT_COST`, `CROSSING_COST` értékek empirikus tesztelése a B01-B12 fixture-ökön
* [ ] Teljesítmény-profiling nagy gráfokon (B10, B11)
* [ ] Eredmények dokumentálása (render idő, crossing count, bend count fixture-önként)

---

## Mérföldkő → Spec fázis mapping

| Mérföldkő | Spec fázis(ok) | Fő kimenet |
|---|---|---|
| **M1** | – | Leforduló monorepo, CLI skeleton |
| **M2** | Fázis 1 | Flowchart parser |
| **M3** | Fázis 2 | Sugiyama elhelyezés |
| **M4** | Fázis 3-4 | Grid + portok |
| **M5** | Fázis 5-6 | A* routing |
| **M6** | Fázis 9 | SVG kimenet ← **első vizuális eredmény** |
| **M7** | Fázis 7 | Zsákutca kezelés |
| **M8** | Fázis 8 | Él címkék |
| **M9** | Fázis 2 (subgraph) | Subgraph vizualizáció |
| **M10** | Fázis 1-2 (class, ER) | Három diagramtípus |
| **M11** | Fázis 13 | Dekompozíció |
| **M12** | CLI | Teljes CLI + Pandoc filter |
| **M13** | – | VS Code extension |
| **M14** | – | IntelliJ plugin |
| **M15** | – | Docker + CI/CD |
| **M16** | – | Benchmark + optimalizáció |

---

## Kritikus út

```
M1 → M2 → M3 → M4 → M5 → M6 (első vizuális eredmény)
                                  │
                                  ├→ M7 → M8 → M9 (render minőség)
                                  │
                                  ├→ M10 (class + ER)
                                  │
                                  ├→ M11 (dekompozíció)
                                  │
                                  ├→ M12 (CLI teljes) → M15 (Docker/CI)
                                  │
                                  └→ M13 (VS Code) ──┐
                                                      ├→ Release
                                  └→ M14 (IntelliJ) ──┘

M16 (benchmark) bármikor futtatható M6 után
```

A **legfontosabb mérföldkő az M6** – itt lesz először vizuálisan értékelhető kimenet. Minden ami utána jön, finomítás és platform-terjesztés.
