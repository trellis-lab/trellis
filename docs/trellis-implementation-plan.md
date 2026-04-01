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
* [x] Vizuális ellenőrzés: B01–B07 fixture-ök renderelése és manuális átnézés

---

## M7 – Zsákutca kezelés (Fázis 7)

> **Cél:** Ha egy él nem routeolható, a rendszer automatikusan megoldja.
> **Smoke test:** B03 (K₃,₃) hiba nélkül renderelődik, esetleg keresztezéssel

* [x] Rip-up and reroute (`deadlock/rip_up.rs` – blokkoló élek azonosítása, visszavonás, újra-routing)
* [x] Blokkoló élek azonosítása (`findBlockingEdges` – módosított A* tracking-gel)
* [x] Rács növelés (`deadlock/expand.rs` – `expandGridAndRetry`, 1.5x skálázás, teljes újra-routing)
* [x] Keresztezéses fallback (`deadlock/fallback.rs` – OCCUPIED cellák véges költséggel átjárhatók)
* [x] Keresztezési pontok megjelölése (`cell.crossing = true`)
* [x] Keresztezési híd renderelés (`render/crossing.rs` – fehér háttér kör + ív SVG)
* [x] `handleDeadlock` 3-szintű védelem bekötése a routing ciklusba
* [x] Unit tesztek: szándékosan zsúfolt gráf ahol rip-up szükséges

---

## M8 – Él címke elhelyezés (Fázis 8)

> **Cél:** Az élek szöveges címkéi megjelennek, ütközésmentesen.
> **Smoke test:** Címkés élek olvashatóan jelennek meg a fixture-ökben

* [x] Szegmens kiválasztás (`labels/placement.rs` – `selectBestSegment`, középső > leghosszabb)
* [x] Pozíció-jelöltek generálása (vízszintes: fölötte/alatta, függőleges: balra/jobbra)
* [x] Ütközés-detektálás (`labels/collision.rs` – csomópontokkal, élekkel, más címkékkel)
* [x] Eltolás a szegmens mentén (`labels/slide.rs` – középponttól kifelé keresés)
* [x] Fallback pozíció (ha minden ütközik: eredeti középpont)
* [x] Címke SVG renderelés (háttér téglalap + szöveg)
* [x] Teljes pipeline bekötés: routing után, SVG renderelés előtt
* [x] Unit tesztek: címke-ütközés szcenáriók

---

## M9 – Subgraph kezelés (Fázis 2 kiegészítés)

> **Cél:** Subgraph-ok vizuálisan csoportosítva jelennek meg, kerettel és címkével.
> **Smoke test:** B08 (nested subgraph) és B09 (subgraph edges) renderelődik

* [x] Subgraph fa felépítése (`placement/subgraph.rs` – `buildSubgraphTree`, szülő-gyerek mapping)
* [x] Rekurzív elhelyezés bottom-up (`placeSubgraphRecursive` – virtuális csomópontok, lokális→globális offset)
* [x] Bounding box számítás (padding + label height)
* [x] Subgraph élek feloldása (`resolveSubgraphEdges` – virtuális csomópontok, `blocksGrid: false`)
* [x] Subgraph keret renderelés (`render/subgraph.rs` – szaggatott keret, háttérszín mélység szerint, címke)
* [x] Z-order: subgraph háttér → élek → csomópontok → címkék
* [x] Unit tesztek: B08 (nested), B09 (subgraph edges)

---

## M10 – Class diagram parser és elhelyezés

> **Cél:** A class diagram működik: osztályok metódusokkal/attribútumokkal, 7 relációtípus, hibrid Sugiyama+laterális elhelyezés.
> **Smoke test:** `cargo run -p trellis-cli -- render b12.mmd -o b12.svg` → osztályhierarchia olvasható SVG-ként jelenik meg, öröklődési háromszögekkel és multiplicitás-jelölésekkel

### Parsing

* [x] `detectType` frissítés a tokenizer-ben: `classDiagram` kulcsszó felismerése
* [x] Class diagram parser (`parser/class_diagram.rs`):
    * Osztálydefiníció: `class ClassName { ... }` és önálló `ClassName` deklaráció
    * Sztereotípa parsing: `<<interface>>`, `<<abstract>>`, `<<enumeration>>`, `<<service>>`
    * Attribútum parsing: `visibility type name` (láthatóság: `+` public, `-` private, `#` protected, `~` package; `$` static, `*` abstract)
    * Metódus parsing: `visibility returnType name(params)` – metódusok `()` végűek; `$` static, `*` abstract
    * Megjegyzés: `note for ClassName "text"` és szabad `note "text"`
    * Namespace blokkok (`namespace ns { ... }`) – opcionális
* [x] Reláció parsing (7 típus, mindkét irányban):
    * Öröklődés: `ClassA <|-- ClassB` és `ClassA --|> ClassB`
    * Kompozíció: `ClassA *-- ClassB` és `ClassA --* ClassB`
    * Aggregáció: `ClassA o-- ClassB` és `ClassA --o ClassB`
    * Asszociáció: `ClassA --> ClassB`, `ClassA <-- ClassB`, `ClassA -- ClassB`
    * Realizáció: `ClassA <|.. ClassB` és `ClassA ..|> ClassB`
    * Függőség: `ClassA ..> ClassB` és `ClassA <.. ClassB`
    * Link: `ClassA .. ClassB`
* [x] Multiplicitás parsing mindkét végponton: `ClassA "1" --> "0..*" ClassB`
* [x] Reláció-felirat parsing: `ClassA --> ClassB : labelText`
* [x] AST bővítés (`ast.rs`):
    * `Node.stereotype: Option<String>`
    * `Node.class_attributes: Vec<ClassAttribute>` (`{ visibility, attr_type, name, is_static, is_abstract }`)
    * `Node.class_methods: Vec<ClassMethod>` (`{ visibility, return_type, name, params, is_static, is_abstract }`)
    * `Edge.class_edge_type: Option<ClassEdgeType>` (7 variáns: `Inheritance`, `Composition`, `Aggregation`, `Association`, `Realization`, `Dependency`, `Link`)
    * `Edge.source_multiplicity: Option<String>` / `Edge.target_multiplicity: Option<String>`

### Elhelyezés (placement)

* [x] `placement/class.rs` – `placeClassDiagram` hibrid algoritmus:
    * Öröklődési + realizációs éleket kigyűjti → `inheritanceGraph`
    * `inheritanceGraph`-on Sugiyama: `breakCycles` → `assignLayers` → `orderWithinLayers` → `assignCoordinates(direction: "TB")`
    * Asszociáció/aggregáció/kompozíció/függőség: még nem elhelyezett csomópontokat a legtöbb éllel rendelkező elhelyezett szomszéd mellé teszi (laterálisan, `findMostConnectedPlacedNeighbor`)
    * `resolveOverlaps`: átfedő csomópontok eltolása (spirális kereséssel, mint `snap.rs`-ben)
* [x] `pipeline.rs` bővítés: `placeByDiagramType` → `classDiagram` eset bekötése

### Renderelés

* [x] `render/class_shapes.rs` – háromrészes osztálydoboz:
    * Felső rész (name compartment): osztálynév bold + sztereotípa (`<<stereotype>>` dőlt, középre)
    * Középső rész: attribútumok soronként (`+ type name`; abstract = dőlt, static = aláhúzott)
    * Alsó rész: metódusok soronként (`+ returnType name(params)`; abstract = dőlt, static = aláhúzott)
    * Üres compartment-nél is megjelenik az elválasztó vonal
    * Doboz szélessége: `MAX(névhossz, leghosszabb attribútum, leghosszabb metódus) + PADDING`
* [x] Élvégjel renderelés – új SVG marker definitiók `render/svg.rs`-ben:
    * Öröklődés: üres háromszög (hollow triangle) a célcsomóponton
    * Realizáció: üres háromszög + szaggatott vonal
    * Kompozíció: tömör rombusz a forráscsomóponton
    * Aggregáció: üres rombusz a forráscsomóponton
    * Asszociáció: nyíl (open arrowhead) a célcsomóponton
    * Függőség: nyíl + szaggatott vonal (`stroke-dasharray`)
    * Link: végjelölő nélkül
* [x] Multiplicitás-felirat renderelés: kis szöveg az él végpontjai közelében (csomóponttól 10–15 px-re)
* [x] Z-order: osztálydobozok → élek → multiplicitás-feliratok → reláció-feliratok

### Tesztelés

* [x] Unit tesztek (`parser/class_diagram.rs` `#[cfg(test)]`):
    * Minden relációtípus parse-olása helyes `ClassEdgeType`-ra
    * Multiplicitás parsing: `"1"`, `"0..*"`, `"1..n"` és hasonlók
    * Sztereotípa parsing: `<<interface>>`, `<<abstract>>`
    * Attribútum/metódus láthatósági szimbólum helyes kiosztása
* [x] Integrációs teszt: B12 (class hierarchy) teljes pipeline-on átmegy, SVG-ben az öröklődési háromszög jelen van

---

## M11 – Entity Relationship (ER) diagram parser és elhelyezés

> **Cél:** Az ER diagram működik: entitások attribútumokkal, crow's foot jelölés mindkét végponton, Fruchterman-Reingold force-directed elhelyezés.
> **Smoke test:** `cargo run -p trellis-cli -- render b11.mmd -o b11.svg` → 100 csomópontos ER diagram renderelődik, crow's foot nyílvégekkel és attribútum-sorokkal

### Parsing

* [x] `detectType` frissítés a tokenizer-ben: `erDiagram` kulcsszó felismerése
* [x] ER diagram parser (`parser/er_diagram.rs`):
    * Entitásdefiníció: `EntityName { type attrName PK, type attrName FK, ... }`
    * Attribútum kulcs-jelölők: `PK` (primary key), `FK` (foreign key), `UK` (unique key) – kombinálhatók
    * Attribútum megjegyzés: `type attrName PK "comment"`
    * Standalone entitás-deklaráció (attribútum-blokk nélkül): `EntityName`
* [x] Reláció parsing – crow's foot jelölések mindkét oldalon:
    * Bal oldali jelölők: `||` (pontosan egy), `|o` (nulla vagy egy), `}|` (egy vagy több), `}o` (nulla vagy több)
    * Jobb oldali jelölők: `||`, `o|`, `|{`, `o{` (tükörszimmetrikus változatok)
    * Kapcsolattípus: `--` (azonosító/identifying, solid vonal), `..` (nem azonosító/non-identifying, dashed vonal)
    * Példák: `EntityA ||--|| EntityB : "label"`, `EntityA }|..|{ EntityB : "label"`
* [x] Reláció-felirat parsing: kötelező `"idézőjeles szöveg"` a `:` után
* [x] AST bővítés (`ast.rs`):
    * `Node.er_attributes: Vec<ErAttribute>` (`{ attr_type, name, keys: Vec<KeyType>, comment: Option<String> }`)
    * `Edge.er_source_card: Option<ErCardinality>` (4 variáns: `ZeroOrOne`, `ExactlyOne`, `ZeroOrMore`, `OneOrMore`)
    * `Edge.er_target_card: Option<ErCardinality>`
    * `Edge.er_identifying: Option<bool>` (true = solid, false = dashed)

### Elhelyezés (placement)

* [x] `placement/force_directed.rs` – általános Fruchterman-Reingold algoritmus (ER-agnosztikus):
    * `ForceDirectedConfig { area_factor: f64, cooling_rate: f64, max_iterations: u32 }`
    * `force_directed_placement(nodes: &[VirtualNode], edges: &[&Edge], config: &ForceDirectedConfig) -> HashMap<NodeId, Point>`
    * Taszítóerő minden csomópont-pár között: `K² / distance`; `K = sqrt(AREA / n)`
    * Vonzóerő élek mentén: `distance² / K`
    * Hőmérséklet-csökkentés: `temperature *= cooling_rate` iterációnként
    * Határvédelem: csomópontok nem lépnek a terület határán kívülre
* [x] `placement/er.rs` – ER-specifikus mapping és utófeldolgozás:
    * `placeErDiagram(graph: &Graph) -> HashMap<NodeId, Point>`
    * ER csomópontok szélességét/magasságát attribútumok száma alapján számolja (`n_attrs * LINE_HEIGHT + HEADER_HEIGHT`)
    * `force_directed_placement` hívása ER-specifikus konfigurációval (`cooling_rate: 0.95`, `max_iterations: 100`)
    * `snapToGrid` hívása az eredményre (meglévő `snap.rs` újrafelhasználva)
* [x] `pipeline.rs` bővítés: `placeByDiagramType` → `erDiagram` eset bekötése

### Renderelés

* [x] `render/er_shapes.rs` – ER entitásdoboz:
    * Fejléc: entitásnév (bold, középre igazítva)
    * Attribútum sorok: `[kulcs-ikon] típus neve` (PK = bold, FK = dőlt, UK = aláhúzott)
    * Doboz szélessége: `MAX(névhossz, leghosszabb attribútum-sor) + PADDING`
    * Doboz magassága: `HEADER_HEIGHT + n_attrs * LINE_HEIGHT + PADDING`
* [x] Crow's foot SVG marker definitiók (új markerek `render/er_shapes.rs`-ben, mindkét végpontra alkalmazható):
    * `ExactlyOne` (`||`): két párhuzamos vonal (double tick)
    * `ZeroOrOne` (`|o`): egy vonal + kör
    * `OneOrMore` (`|{`): egy vonal + három szétnyíló vonal (crow's foot)
    * `ZeroOrMore` (`o{`): kör + három szétnyíló vonal
    * Mindkét végpontra külön marker-ref: `sourceMarker` és `targetMarker`
* [x] Él stílusa: `er_identifying == true` → solid vonal; `false` → dashed (`stroke-dasharray`)
* [x] Reláció-felirat renderelés: az él közepén, háttér-téglalap + szöveg (meglévő `labels/placement.rs` újrafelhasználva)
* [x] Z-order: entitásdobozok → élek → crow's foot markerek → reláció-feliratok

### Tesztelés

* [x] Unit tesztek (`parser/er_diagram.rs` `#[cfg(test)]`):
    * Minden kardinalitás-kombináció parse-olása helyes `ErCardinality`-ra
    * `--` vs `..` kapcsolattípus helyes felismerése
    * Attribútum kulcs-jelölők: `PK`, `FK`, `UK`, kombinált (`PK,FK`)
    * Standalone entitás + reláció-felirat parsing
* [x] Integrációs teszt: B11 (100 node ER) teljes pipeline-on átmegy, SVG-ben crow's foot markerek jelen vannak
* [x] Teljesítmény-teszt: B11 force-directed elhelyezés elfogadható idő alatt konvergál (< 2 s)

---

## M12 – C4 diagram parser és elhelyezés

> **Cél:** Az öt C4 diagramszint (Context, Container, Component, Dynamic, Deployment) mindegyike parseolható és renderelhető.
> **Smoke test:** `cargo run -p trellis-cli -- render c4_context.mmd -o c4_context.svg` → Enterprise_Boundary-ban Person, System, System_Ext elemek és Rel kapcsolatok megjelennek

### Parsing

* [x] C4 diagramtípus felismerése a tokenizer-ben (`detectType` – `C4Context`, `C4Container`, `C4Component`, `C4Dynamic`, `C4Deployment` kulcsszavak)
* [x] C4 elem-parser (`parser/c4_diagram.rs` – mind az öt diagramszintet egységes elemtípus-készlettel kezeli)
* [x] Person / Person_Ext parsing (label, description)
* [x] System-elemek parsing: `System`, `SystemDb`, `SystemQueue` és `_Ext` variánsaik (label, description, technológia)
* [x] Container-elemek parsing: `Container`, `ContainerDb`, `ContainerQueue` és `_Ext` variánsaik
* [x] Component-elemek parsing: `Component`, `ComponentDb`, `ComponentQueue` és `_Ext` variánsaik
* [x] Deployment_Node / Node / Node_L / Node_R parsing (nested deployment fák)
* [x] Boundary-blokkok parsing: `Enterprise_Boundary`, `System_Boundary`, `Container_Boundary` – egymásba ágyazva (rekurzív)
* [x] Kapcsolat-parser: `Rel`, `BiRel`, `Rel_U/D/L/R/Back`, `RelIndex` (forrás, cél, label, technológia)
* [x] `UpdateLayoutConfig(?c4ShapeInRow, ?c4BoundaryInRow)` directive parsing (névvel és pozíció alapján is) – silently ignored

### Elhelyezés (placement)

A C4 elhelyezés **nem gráfalgoritmus-alapú** – sorfolyásos (row-flow) elrendezést alkalmaz:

* [x] `placement/c4.rs` – `placeC4Elements`: elemek balról jobbra, `c4ShapeInRow` darabonként sortörés; boundary-k `c4BoundaryInRow` darabonként sortörés
* [x] Boundary bounding box számítás: gyermekelemek koordinátáiból alulról felfelé (bottom-up), padding + feliratmagasság figyelembevételével
* [x] Nested boundary kezelés (rekurzív: belső elemek koordinátái a belső boundary-n belül relatívak, majd globálissá alakítva)
* [x] Deployment_Node fa elhelyezése: mélység szerinti indent, `Node_L`/`Node_R` bal/jobb oldali elrendezés
* [x] `Rel_U/D/L/R` → routing hint: a megadott irány az A* pathfinder kiindulási port-oldalát kényszeríti

### Renderelés

* [x] `render/c4_shapes.rs` – speciális alakzatok:
    * `Person` / `Person_Ext`: emberfigura-ikon (kör fej + váll)
    * `SystemDb` / `ContainerDb` / `ComponentDb` és `_Ext` variánsaik: henger (cylinder)
    * `SystemQueue` / `ContainerQueue` / `ComponentQueue` és `_Ext` variánsaik: sor-ikon (dupla keret)
    * `_Ext` variánsok: szaggatott körvonal (stroke-dasharray)
* [x] Boundary renderelés: `render/c4_boundary.rs` – vékony, folytonos keret, a felirat és a boundary típusa baloldalt alul legyen
* [x] Kapcsolat renderelés: technológia-felirat az élcímke második soraként; `BiRel` → kétirányú nyíl
* [ ] `UpdateElementStyle` directive feldolgozása (egyedi szín/stílus felülírás) – nem implementált
* [ ] `UpdateRelStyle` directive feldolgozása – nem implementált
* [x] Z-order: boundary háttér → elemek → élek → elemcímkék → élcímkék

### Tesztelés

* [x] Unit tesztek: C4Context (Person + System + Enterprise_Boundary + Rel), C4Container (Container + System_Boundary), C4Deployment (Deployment_Node fa) fixture-ök
* [x] `UpdateLayoutConfig` tesztelése: 2 shape/sor, 1 boundary/sor → elhelyezés ellenőrzés

---

## M13 – Benchmark és finomhangolás

> **Cél:** A költségfüggvény konstansai optimalizálva, a teljesítmény mérve és dokumentálva.
> **Smoke test:** `cargo bench` lefut, eredmények a `target/criterion/` alatt, minden benchmark elfogadható idő alatt renderelődik

* [x] Criterion benchmark runner (`tests/benchmarks/src/bench.rs`) – B01-B12 fixture-ök
* [x] Invariáns tesztek (`tests/integration/invariants.rs`):
    * [x] Fedésmentesség: nincs két él ugyanazon a rácsponton
    * [x] Blokkolás: nincs él csomóponton átmenő rácsponton
    * [x] Ortogonalitás: minden él csak vízszintes/függőleges szegmensekből áll
    * [x] Összefüggőség: minden él összefüggő útvonal a portok között
    * [x] Port egyediség: nincs két él ugyanazon a porton
* [x] Költségfüggvény finomhangolás: `BEND_COST`, `ADJACENT_COST`, `CROSSING_COST` értékek empirikus tesztelése a B01-B12 fixture-ökön
* [x] Eredmények dokumentálása (render idő, crossing count, bend count fixture-önként)

---

## M13.1 – Placement algoritmus / diagram szétválasztás refaktorálás

> **Cél:** A placement modul szerkezete következetes legyen: minden algoritmus saját fájlban él, a diagram-specifikus orchestráció külön fájlban hívja azt – ugyanúgy, ahogy a flowchart/Sugiyama (`sugiyama.rs` + `subgraph.rs`) és az ER/force-directed (`force_directed.rs` + `er.rs`) már elkülönül.
> **Smoke test:** `cargo test --workspace` zöld; `cargo clippy --workspace` hiba nélkül; minden diagramtípus SVG kimenettel renderelődik

* [x] `placement/row_flow.rs` létrehozása: `place_in_rows` és `compute_row_heights` generikus, diagram-agnosztikus row-flow függvények kiemelése a `c4.rs`-ből
* [x] `placement/c4.rs` frissítése: a kiemelten algoritmus-függvények helyett a `row_flow` modul hívása; C4-specifikus logika (boundary osztályozás, containment map, subgraph adat) marad `c4.rs`-ben
* [x] `placement/overlap.rs` létrehozása: `overlaps_any` és `find_free_position` generikus ütközésvizsgáló / spirális kereső függvények kiemelése a `class.rs`-ből
* [x] `placement/class.rs` frissítése: a kiemelten algoritmus-függvények helyett az `overlap` modul hívása; class-specifikus logika (öröklődési gráf kiemelés, laterális elhelyezés) marad `class.rs`-ben
* [x] `placement/mod.rs` kiegészítése: `pub mod overlap;` és `pub mod row_flow;` deklaráció hozzáadása
* [x] `cargo test --workspace` – minden teszt zöld
* [x] `cargo clippy --workspace` – nincsenek új hibák

---

## M14 – CLI teljes funkciókészlet

> **Cél:** Az összes CLI parancs működik, beleértve batch módot, preprocessort
> **Smoke test:** `trellis preprocess doc.md -o out.md --image-dir img/` → Mermaid blokkok képekre cserélve

* [x] `render` stdin mód (`trellis render - -f svg` – pipe-olható)
* [x] `render-batch` parancs (könyvtár bejárás, .mmd szűrés, párhuzamos renderelés)
* [x] `validate` parancs (parser + összesítő, szintaxis hibák kiírása)
* [x] `preprocess` parancs (markdown Mermaid blokkok → képhivatkozások)
* [x] `--metrics` JSON kimenet stderr-re (crossings, bends, render_ms, grid_utilization)
* [x] `--config` fájl betöltés (`~/.trellis/config.toml`)
* [x] Pandoc Lua filter (`filters/trellis-filter.lua`) megírása és tesztelése
* [x] Exit kódok: 0 = OK, 1 = parse hiba, 2 = render hiba (licenckezelés → Mc mérföldkő)

---

## M15 – WASM build + VS Code extension

> **Cél:** A VS Code extension működik: .mmd fájl megnyitás → preview panel → renderelt diagram.
> **Smoke test:** VS Code-ban F5 → .mmd fájl megnyitva → preview panelen renderelt diagram

* [x] `trellis-wasm` crate: `render()` és `render_with_metrics()` wasm-bindgen API
* [x] `scripts/build-wasm.sh` – wasm-pack build (web target)
* [x] VS Code extension váz (`package.json`, `tsconfig.json`, `extension.ts`)
* [x] WASM bridge (`wasm-bridge.ts` – WASM betöltés, `render()` hívás)
* [x] Preview panel (`preview.ts` – webview, SVG megjelenítés)
* [x] Preview webview template (`media/preview.html`)
* [x] Fájl mentés figyelés (`.mmd` fájl változáskor automatikus újra-render)
* [x] Export funkció (`export.ts` – PNG/SVG mentés) (licenckorlátok → Mc mérföldkő)
* [ ] `vsce package` → `.vsix` fájl előállítás

---

## M16 – IntelliJ plugin

> **Cél:** IntelliJ plugin működik: .mmd fájl → tool window → renderelt diagram.
> **Smoke test:** IntelliJ-ben Run Plugin → .mmd fájl → preview

* [x] `scripts/build-wasm.sh` kiegészítés bundler targettel (IntelliJ-hez) – web target JCEF-hez
* [x] Plugin váz (`build.gradle.kts`, `plugin.xml`, `TrellisPlugin.kt`)
* [x] WASM bridge JCEF-alapú (`WasmBridge.kt` – erőforrás kinyerés + file:// URIs)
* [x] Preview tool window (`PreviewPanel.kt` – JCEF/SVG megjelenítés)
* [x] Fájl változás figyelés (IntelliJ VFS listener – `MmdFileListener.kt`)
* [x] Export (`ExportAction.kt` – PNG/SVG) (licenckorlátok → Mc mérföldkő)
* [ ] `./gradlew buildPlugin` → `.zip` fájl

---

## M17 – Docker + CI/CD

> **Cél:** Docker image és GitHub Actions CI pipeline működik.
> **Smoke test:** `docker run --rm -v $(pwd):/data ghcr.io/trellis/trellis:latest trellis render /data/test.mmd -o /data/test.svg`

* [x] `docker/Dockerfile.wasm-builder` – multi-stage build (builder + slim runtime)
* [x] `docker/Dockerfile.vscode-builder` – multi-stage build (builder + slim runtime)
* [x] `docker/Dockerfile.intellij-builder` – multi-stage build (builder + slim runtime)
* [x] Pandoc Dockerfile felépítése a teljes markdown rendereléshez
* [x] `scripts/build-wasm-docker.sh` – image build + tag
* [x] `scripts/build-vscode-docker.sh` – image build + tag
* [x] `scripts/build-intellij-docker.sh` – image build + tag
* [x] `scripts/build-cli.sh` – cross-compile: Linux x86_64, macOS x86_64/ARM, Windows
* [x] GitHub Actions CI workflow: `cargo test` → `cargo bench` → WASM build → CLI build → Docker build
* [x] GitHub Actions: artifact upload (.vsix, .zip, CLI binárisok, Docker push)
* [x] GitHub Releases automatizáció (tag push → release draft + binárisok)
* [x] README.md (telepítés, használat, példák, badge-ek)

---

## M18 – Dekompozíció (Fázis 13)

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
* [ ] Teljesítmény-profiling nagy gráfokon (B10, B11)

---

## M19 – Kereskedelmi funkciók (Commercialisation)

> **Cél:** Licenckezelés és freemium korlátok implementálása a CLI-ben, VS Code extensionben és IntelliJ pluginban.
> **Előfeltétel:** M14 (CLI teljes funkciókészlet), M15 (VS Code extension), M16 (IntelliJ plugin)
> **Smoke test:** `trellis license --activate <kulcs>` → "OK, Premium aktiválva" | `trellis render b10.mmd` (free tier, 50 node) → "Free verzió: max 10 csomópont" hibaüzenet (exit 3)

### CLI licenckezelés (`trellis-cli`)

* [ ] `license --activate <kulcs>` / `--status` / `--deactivate` parancs implementálása (`commands/license.rs`)
* [ ] Lemon Squeezy API kliens (`license/lemon.rs`):
    * `activate(key, instance_id) -> LicenseResponse`
    * `validate(key, instance_id) -> ValidationResult`
    * `deactivate(key, instance_id) -> Result<()>`
* [ ] Licencállapot tárolás (`license/state.rs`): `~/.trellis/license.json` – kulcs, tier, lejárat, instance_id, last_validated
* [ ] Online revalidáció 30 napos grace period-dal: ha > 7 nap telt el az utolsó validáció óta → háttérben újraellenőrzés; offline esetén grace period-ig Premium marad
* [ ] Licenc ellenőrzés a `render` parancsban:
    * Free tier: max 10 csomópont (felette → stderr hibaüzenet, exit 3)
    * Free tier: csak PNG kimenet (SVG kísérlet → stderr hibaüzenet, exit 3)
    * Free tier: vízjel hozzáadása az SVG/PNG kimenethez (`add_watermark`)
* [ ] Licenc ellenőrzés a `render-batch` parancsban (ugyanazok a korlátok)
* [ ] Exit kód: 3 = licenc hiba (bekötés `main.rs`-ben)

### VS Code Extension licenckezelés

* [ ] `license.ts` – freemium korlátok:
    * `applyLimits(source, state)` → max 10 csomópont Free tier-en
    * `shouldAddWatermark(state) -> bool`
    * `getAllowedExportFormats(state) -> string[]` (Free: `["png"]`, Premium: `["png", "svg"]`)
* [ ] Licenckulcs tárolás VS Code `SecretStorage`-ban (nem plaintextben)
* [ ] Premium aktiválási UI: `vscode.window.showInputBox` → kulcs bevitel → Lemon Squeezy validálás → visszajelzés
* [ ] Státuszsor jelző: `$(key) Trellis Free` / `$(verified) Trellis Premium`

### IntelliJ Plugin licenckezelés

* [ ] `LicenseManager.kt` – JetBrains Marketplace licencelés (`LicensingFacade`):
    * `LicensingFacade.getInstance()` → plugin licenc ellenőrzése
    * Fallback: saját Lemon Squeezy licenckulcs (ha nem JetBrains Marketplace-en vásárolt)
* [ ] Freemium korlátok IntelliJ-ben: 10 csomópont limit (Free), vízjel, PNG-only export
* [ ] Premium aktiválás dialóg (`LicenseActivationDialog.kt`): kulcs input + validálás + visszajelzés

### Tesztelés

* [ ] CLI: Free tier limit tesztelése – 11 csomópontos diagram → exit 3
* [ ] CLI: Premium aktiválás-deaktiválás flow (mock Lemon Squeezy API-val, `reqwest` mock)
* [ ] CLI: Grace period tesztelése – last_validated > 30 nap → Free tier-re visszaesés
* [ ] CLI: Vízjel megjelenése a kimenetben Free tier-en
* [ ] VS Code: `applyLimits` unit tesztek (`license.test.ts`)

---

## Mérföldkő → Spec fázis mapping

| Mérföldkő | Spec fázis(ok) | Fő kimenet |
|-----------|---|---|
| **M1**    | – | Leforduló monorepo, CLI skeleton |
| **M2**    | Fázis 1 | Flowchart parser |
| **M3**    | Fázis 2 | Sugiyama elhelyezés |
| **M4**    | Fázis 3-4 | Grid + portok |
| **M5**    | Fázis 5-6 | A* routing |
| **M6**    | Fázis 9 | SVG kimenet ← **első vizuális eredmény** |
| **M7**    | Fázis 7 | Zsákutca kezelés |
| **M8**    | Fázis 8 | Él címkék |
| **M9**    | Fázis 2 (subgraph) | Subgraph vizualizáció |
| **M10**   | Fázis 1-2 (class) | Class diagram (hibrid Sugiyama+laterális) |
| **M11**   | Fázis 1-2 (ER) | ER diagram (force-directed, crow's foot) |
| **M12**   | – | C4 diagramtípusok (Context/Container/Component/Dynamic/Deployment) |
| **M13**   | Fázis 13 | Dekompozíció |
| **M14**   | CLI | Teljes CLI + Pandoc filter |
| **M15**   | – | VS Code extension |
| **M16**   | – | IntelliJ plugin |
| **M17**   | – | Docker + CI/CD |
| **M18**   | – | Benchmark + optimalizáció |
| **M19**   | – | Kereskedelmi funkciók (licenckezelés, freemium korlátok) |

---

## Kritikus út

```
M1 → M2 → M3 → M4 → M5 → M6 (első vizuális eredmény)
                                  │
                                  ├→ M7 → M8 → M9 (render minőség)
                                  │
                                  ├→ M10 (class diagram)
                                  │
                                  ├→ M11 (ER diagram)
                                  │
                                  ├→ M12 (C4 diagram)
                                  │
                                  ├→ M13 (dekompozíció)
                                  │
                                  ├→ M14 (CLI teljes) → M17 (Docker/CI)
                                  │
                                  ├→ M15 (VS Code) ──┐
                                  │                   ├→ M18 (benchmark) → Mc (commercialisation) → Release
                                  └→ M16 (IntelliJ) ──┘

M18 (benchmark) bármikor futtatható M6 után; Mc az összes többi mérföldkő után
```

A **legfontosabb mérföldkő az M6** – itt lesz először vizuálisan értékelhető kimenet. Minden ami utána jön, finomítás és platform-terjesztés.
