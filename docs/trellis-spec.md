# Trellis – Mermaid-kompatibilis diagram renderer

## Grid-alapú útvonaltervező algoritmus DaaC megoldásokhoz

**Verzió:** 0.3 (subgraph kezelés + dekompozíció)
**Dátum:** 2026-02-08

---

## Tartalomjegyzék

1. [Problématér](#1-problématér)
2. [Megoldás áttekintése](#2-megoldás-áttekintése)
3. [Architekturális pipeline](#3-architekturális-pipeline)
4. [Fázis 1 – Mermaid parsing](#4-fázis-1--mermaid-parsing)
5. [Fázis 2 – Csomópont-elhelyezés](#5-fázis-2--csomópont-elhelyezés)
6. [Fázis 3 – Grid paraméterek és felépítés](#6-fázis-3--grid-paraméterek-és-felépítés)
7. [Fázis 4 – Port-kiosztás](#7-fázis-4--port-kiosztás)
8. [Fázis 5 – Routing prioritás](#8-fázis-5--routing-prioritás)
9. [Fázis 6 – Él-routing (A* pathfinding)](#9-fázis-6--él-routing-a-pathfinding)
10. [Fázis 7 – Zsákutca kezelés](#10-fázis-7--zsákutca-kezelés)
11. [Fázis 8 – Él címke elhelyezés](#11-fázis-8--él-címke-elhelyezés)
12. [Fázis 9 – SVG rendering](#12-fázis-9--svg-rendering)
13. [Dekompozíció (nagy gráfok kezelése)](#13-dekompozíció-nagy-gráfok-kezelése)
14. [Piaci pozícionálás](#14-piaci-pozícionálás)
15. [Tesztelési stratégia](#15-tesztelési-stratégia)
16. [Nyitott kérdések](#16-nyitott-kérdések)

---

## 1. Problématér

### PROBLEM_0001 – Rossz layout minőség
A DaaC rendering megoldások (különösen a Mermaid + Dagre) átláthatatlan rajzokat készítenek. A Dagre egy régi, karbantartatlan könyvtár (utolsó érdemi fejlesztés ~2018), egyszerű Sugiyama-alapú algoritmussal, minimális élkeresztezés-optimalizálással.

### PROBLEM_0002 – DaaC értékvesztés
A rossz layoutok miatt a felhasználók jellemzően **eldobják a DaaC kimenetet** és GUI eszközben (draw.io, Figma) rajzolják újra. Ez a DaaC teljes értékajánlatát (verziókezelhetőség, automatizálhatóság, szinkronban tartás) semmissé teszi.

### PROBLEM_0003 – MI generált diagramok
Az LLM-ek (ChatGPT, Claude, Copilot) natívan Mermaid kódot generálnak. A leggyakoribb MI-s diagramkérések (architektúra, osztálystruktúra, adatmodell) pont azok a típusok, ahol a Dagre layout a leggyengébb.

### PROBLEM_0004 – Ügyfélnek adható kimenet
A kézi átdolgozás eliminálása mérhető üzleti értéket teremt. A célszint: **belső tech kommunikáció** ("jól olvasható") és **ügyféldokumentáció** ("professzionális megjelenés").

### PROBLEM_0005 – Gráfelméleti korlát
A Kuratowski-tétel értelmében a valós SW architektúrák gráfjai jellemzően nem síkgráfok, ezért **egyenes élekkel a keresztezés matematikailag elkerülhetetlen**. Már 3 frontend + 3 backend service (K₃,₃) elég a probléma megjelenéséhez.

### PROBLEM_0006 – Élfedés és keresztezés
Két különböző vizuális probléma, eltérő súlyossággal:

| Probléma | Hatás | Tolerancia |
|----------|-------|-----------|
| **Élfedés** (két él azonos útvonalon) | Információvesztés – az olvasó nem is tudja, hogy több él van | **Nulla tolerancia** |
| **Élkeresztezés** (két él metszi egymást) | Zavaró, de az információ megmarad | **Minimalizálandó**, vizuális jelölés megengedett |

---

## 2. Megoldás áttekintése

### Alapötlet

A megoldás a **VLSI maze routing** technikát adaptálja diagram renderelésre:

1. A rajztér egy **négyzetrácsra** kerül
2. A komponensek **téglalapokként** a rácsra helyezkednek
3. Az élek **útvonaltervező algoritmussal** (A*) kerülnek routeolásra a rácson
4. Egy lefoglalt útvonal blokkolja a további éleket → **fedésmentesség garantált by design**
5. A szomszédos rácspontok költsége megnő → élek egymástól elkülönülve futnak
6. Az eredmény: **ortogonális élek**, professzionális megjelenéssel

### Design elvek

- **Fedésmentesség by construction** – nem utólagos javítás, hanem a rendszer úgy épül fel, hogy fedés nem keletkezhet
- **Mermaid szintaxis kompatibilitás** – ugyanazt a bemenetet fogadja, mint a Mermaid
- **Nem cél a Mermaid belső pipeline-ba integrálódás** – önálló renderer

### Támogatott diagramtípusok

| Típus | Támogatott | Indoklás |
|-------|-----------|----------|
| Flowchart / graph | ✅ Fókusz | Leggyakoribb, legnagyobb layout probléma |
| Class diagram | ✅ Fókusz | Öröklődés + asszociáció = sűrű gráf |
| ER diagram | ✅ Fókusz | Sok reláció, N:M kapcsolatok |
| Sequence diagram | ❌ Nem szükséges | Jelenlegi layout elfogadható |
| Gantt diagram | ❌ Nem szükséges | Jelenlegi layout elfogadható |

---

## 3. Architekturális pipeline

```
┌──────────────────────────────────────────────────────────┐
│                   Trellis Pipeline                      │
│                                                          │
│  ┌─────────┐   ┌──────────┐   ┌───────────┐             │
│  │ Mermaid  │──>│ Csomópont│──>│   Grid    │             │
│  │ Parse    │   │ elhelyez.│   │ felépítés │             │
│  │ (Fázis 1)│   │ (Fázis 2)│   │ (Fázis 3) │             │
│  └─────────┘   └──────────┘   └─────┬─────┘             │
│                                      │                    │
│  ┌─────────┐   ┌──────────┐   ┌─────▼─────┐             │
│  │   SVG   │<──│ Címke    │<──│   Port    │             │
│  │ Render  │   │ elhelyez.│   │ kiosztás  │             │
│  │(Fázis 9)│   │ (Fázis 8)│   │ (Fázis 4) │             │
│  └─────────┘   └──────────┘   └─────┬─────┘             │
│                     ▲                │                    │
│                     │          ┌─────▼─────┐             │
│                ┌────┴─────┐   │  Routing  │             │
│                │ Zsákutca  │<──│ prioritás │             │
│                │  kezelés  │   │ (Fázis 5) │             │
│                │ (Fázis 7) │   └───────────┘             │
│                └────┬─────┘                              │
│                     │                                    │
│                ┌────▼─────┐                              │
│                │ A* route │                              │
│                │+ multi-él│                              │
│                │(Fázis 6) │                              │
│                └──────────┘                              │
└──────────────────────────────────────────────────────────┘
```

---

## 4. Fázis 1 – Mermaid parsing

### Cél
A Mermaid szintaxist gráf adatstruktúrává alakítani.

### Bemenet
Szabványos Mermaid szintaxis (flowchart, classDiagram, erDiagram).

### Kimenet

```
Graph {
  type: "flowchart" | "classDiagram" | "erDiagram"
  direction: "TB" | "BT" | "LR" | "RL"  // flowchart esetén
  nodes: [
    Node {
      id: string
      label: string
      shape: "rect" | "rounded" | "diamond" | "circle" | ...
      width: number   // szöveg alapján számított
      height: number
      subgraph: string | null
    }
  ]
  edges: [
    Edge {
      source: string  // node id
      target: string  // node id
      label: string | null
      type: "solid" | "dotted" | "thick"
      arrowHead: "arrow" | "none" | "circle" | ...
    }
  ]
  subgraphs: [
    Subgraph {
      id: string
      label: string
      children: string[]  // node id-k
    }
  ]
}
```

### Pseudocode

```
FUNCTION parseMermaid(source: string) -> Graph:
    tokens = tokenize(source)
    diagramType = detectType(tokens)  // "flowchart", "classDiagram", "erDiagram"
    
    SWITCH diagramType:
        CASE "flowchart":
            RETURN parseFlowchart(tokens)
        CASE "classDiagram":
            RETURN parseClassDiagram(tokens)
        CASE "erDiagram":
            RETURN parseERDiagram(tokens)

FUNCTION parseFlowchart(tokens) -> Graph:
    graph = new Graph(type: "flowchart")
    graph.direction = extractDirection(tokens)  // TB, BT, LR, RL
    
    FOR EACH statement IN tokens:
        IF statement is nodeDefinition:
            node = parseNode(statement)
            node.width = calculateTextWidth(node.label) + PADDING
            node.height = calculateTextHeight(node.label) + PADDING
            graph.nodes.add(node)
        ELSE IF statement is edgeDefinition:
            edge = parseEdge(statement)
            graph.edges.add(edge)
        ELSE IF statement is subgraphDefinition:
            subgraph = parseSubgraph(statement)
            graph.subgraphs.add(subgraph)
    
    RETURN graph
```

### Megjegyzés
A Mermaid parser implementálható a Mermaid projekt saját parser-jének újrafelhasználásával, vagy egy dedikált PEG/ANTLR grammar-rel. Ez implementációs döntés.

---

## 5. Fázis 2 – Csomópont-elhelyezés

### Cél
Minden csomóponthoz (x, y) koordinátát rendelni, figyelembe véve a diagram típusát és a gráf struktúráját.

### Stratégia diagramtípusonként

| Diagram típus | Elhelyezési algoritmus | Indoklás |
|--------------|----------------------|----------|
| Flowchart | Sugiyama (rétegezett) | Irányított gráf, `direction` direktíva adja a fő irányt |
| Class diagram | Hibrid (Sugiyama + laterális) | Öröklődés = hierarchia, asszociáció = laterális |
| ER diagram | Force-directed (Fruchterman-Reingold) | Nincs természetes hierarchia, hálózatos struktúra |

---

### 5.1 Flowchart – Sugiyama elhelyezés

A `direction` direktíva (TB/BT/LR/RL) meghatározza a rétegezés irányát.

#### 5.1.1 Ciklustörés

```
FUNCTION breakCycles(graph: Graph) -> Graph:
    // DFS-alapú ciklus detektálás
    visited = {}
    inStack = {}
    reversedEdges = []
    
    FUNCTION dfs(node):
        visited[node] = true
        inStack[node] = true
        
        FOR EACH edge WHERE edge.source == node:
            IF inStack[edge.target]:
                // Ciklus detektálva – él megfordítása
                edge.reversed = true
                reversedEdges.add(edge)
                SWAP(edge.source, edge.target)
            ELSE IF NOT visited[edge.target]:
                dfs(edge.target)
        
        inStack[node] = false
    
    FOR EACH node IN graph.nodes:
        IF NOT visited[node]:
            dfs(node)
    
    RETURN graph  // megfordított élekkel
```

#### 5.1.2 Réteg-hozzárendelés

```
FUNCTION assignLayers(graph: Graph) -> Map<Node, int>:
    layers = {}
    
    // Longest path algoritmus
    // Forrás csomópontok (nincs bejövő él) → 0. réteg
    sources = graph.nodes.filter(n => inDegree(n) == 0)
    
    FOR EACH source IN sources:
        layers[source] = 0
    
    // Topologikus sorrend szerinti bejárás
    FOR EACH node IN topologicalSort(graph):
        IF node NOT IN layers:
            layers[node] = 0
        FOR EACH edge WHERE edge.source == node:
            target = edge.target
            layers[target] = MAX(layers[target] OR 0, layers[node] + 1)
    
    RETURN layers
```

#### 5.1.3 Rétegen belüli sorrend (Barycenter heurisztika)

```
FUNCTION orderWithinLayers(graph: Graph, layers: Map<Node, int>) -> Map<Node, int>:
    // layers: node → layer index
    // Visszatérés: node → pozíció a rétegen belül
    
    positions = {}
    maxLayer = MAX(layers.values())
    
    // Kezdeti sorrend: ahogy a forrásban definiálva vannak
    FOR EACH layer IN 0..maxLayer:
        nodesInLayer = getNodesInLayer(layers, layer)
        FOR i, node IN enumerate(nodesInLayer):
            positions[node] = i
    
    // Iteratív finomítás (tipikusan 10-20 iteráció elég)
    FOR iteration IN 1..MAX_ITERATIONS:
        improved = false
        
        // Felülről lefelé
        FOR layer IN 1..maxLayer:
            nodesInLayer = getNodesInLayer(layers, layer)
            FOR EACH node IN nodesInLayer:
                // Barycenter: szomszédok átlagos pozíciója az előző rétegben
                neighbors = getNeighborsInLayer(node, layer - 1)
                IF neighbors.length > 0:
                    barycenter = AVG(positions[n] FOR n IN neighbors)
                    positions[node] = barycenter
            
            // Rendezés barycenter szerint
            sorted = sortByPosition(nodesInLayer, positions)
            FOR i, node IN enumerate(sorted):
                IF positions[node] != i:
                    improved = true
                positions[node] = i
        
        // Lentről felfelé (ugyanaz, fordított irányban)
        FOR layer IN (maxLayer-1)..0:
            // ... ugyanaz a logika, de layer+1 szomszédokkal
        
        IF NOT improved:
            BREAK
    
    RETURN positions
```

#### 5.1.4 Koordináta-hozzárendelés

```
FUNCTION assignCoordinates(
    graph: Graph,
    layers: Map<Node, int>,
    positions: Map<Node, int>,
    direction: string
) -> Map<Node, {x, y}>:
    
    coordinates = {}
    
    LAYER_SPACING = 100  // rétegek közti távolság (pixelben)
    NODE_SPACING = 80    // csomópontok közti távolság rétegen belül
    
    FOR EACH node IN graph.nodes:
        layer = layers[node]
        pos = positions[node]
        
        SWITCH direction:
            CASE "TB", "TD":
                coordinates[node] = {
                    x: pos * NODE_SPACING,
                    y: layer * LAYER_SPACING
                }
            CASE "BT":
                coordinates[node] = {
                    x: pos * NODE_SPACING,
                    y: -layer * LAYER_SPACING  // fordított
                }
            CASE "LR":
                coordinates[node] = {
                    x: layer * LAYER_SPACING,
                    y: pos * NODE_SPACING
                }
            CASE "RL":
                coordinates[node] = {
                    x: -layer * LAYER_SPACING,
                    y: pos * NODE_SPACING
                }
    
    // Centrálás: a rétegen belüli csomópontokat középre igazítjuk
    FOR EACH layer IN 0..maxLayer:
        nodesInLayer = getNodesInLayer(layers, layer)
        totalWidth = SUM(node.width FOR node IN nodesInLayer) 
                     + (nodesInLayer.length - 1) * NODE_SPACING
        offset = -totalWidth / 2
        FOR EACH node IN nodesInLayer:
            coordinates[node].x += offset
            offset += node.width + NODE_SPACING
    
    RETURN coordinates
```

---

### 5.2 Class diagram – Hibrid elhelyezés

```
FUNCTION placeClassDiagram(graph: Graph) -> Map<Node, {x, y}>:
    // 1. Öröklődési élek kigyűjtése
    inheritanceEdges = graph.edges.filter(e => 
        e.type IN ["extends", "implements", "inheritance"])
    associationEdges = graph.edges.filter(e =>
        e.type NOT IN ["extends", "implements", "inheritance"])
    
    // 2. Öröklődési fa/erdő felépítése
    inheritanceGraph = buildSubgraph(graph.nodes, inheritanceEdges)
    
    // 3. Sugiyama elhelyezés az öröklődési fára
    layers = assignLayers(inheritanceGraph)
    positions = orderWithinLayers(inheritanceGraph, layers)
    coordinates = assignCoordinates(inheritanceGraph, layers, positions, "TB")
    
    // 4. Asszociált osztályok elhelyezése
    unplaced = graph.nodes.filter(n => n NOT IN coordinates)
    
    FOR EACH node IN unplaced:
        // Megkeressük a legtöbb asszociációval rendelkező már elhelyezett szomszédot
        bestNeighbor = findMostConnectedPlacedNeighbor(node, associationEdges, coordinates)
        
        IF bestNeighbor != null:
            // A szomszéd mellé helyezzük (laterálisan)
            coordinates[node] = {
                x: coordinates[bestNeighbor].x + bestNeighbor.width + NODE_SPACING,
                y: coordinates[bestNeighbor].y
            }
        ELSE:
            // Nincs elhelyezett szomszéd – a legalsó réteg alá
            coordinates[node] = findFreePosition(coordinates)
    
    // 5. Átfedés-feloldás
    resolveOverlaps(coordinates, graph.nodes)
    
    RETURN coordinates
```

---

### 5.3 ER diagram – Force-directed elhelyezés

```
FUNCTION placeERDiagram(graph: Graph) -> Map<Node, {x, y}>:
    // Fruchterman-Reingold algoritmus
    
    AREA = estimateArea(graph.nodes.length)
    K = SQRT(AREA / graph.nodes.length)  // ideális távolság
    TEMPERATURE = AREA / 10              // kezdeti max. elmozdulás
    COOLING_RATE = 0.95
    MAX_ITERATIONS = 100
    
    // Kezdeti pozíciók: véletlenszerű vagy kör mentén
    coordinates = {}
    FOR i, node IN enumerate(graph.nodes):
        angle = (2 * PI * i) / graph.nodes.length
        coordinates[node] = {
            x: AREA/2 + (AREA/3) * COS(angle),
            y: AREA/2 + (AREA/3) * SIN(angle)
        }
    
    FOR iteration IN 1..MAX_ITERATIONS:
        displacements = {node: {dx: 0, dy: 0} FOR node IN graph.nodes}
        
        // Taszító erő: minden csomópont-pár között
        FOR EACH (u, v) IN allNodePairs(graph.nodes):
            delta = coordinates[u] - coordinates[v]
            distance = MAX(length(delta), 0.01)
            repulsiveForce = (K * K) / distance
            direction = normalize(delta)
            displacements[u] += direction * repulsiveForce
            displacements[v] -= direction * repulsiveForce
        
        // Vonzó erő: élek mentén
        FOR EACH edge IN graph.edges:
            u = edge.source
            v = edge.target
            delta = coordinates[v] - coordinates[u]
            distance = MAX(length(delta), 0.01)
            attractiveForce = (distance * distance) / K
            direction = normalize(delta)
            displacements[u] += direction * attractiveForce
            displacements[v] -= direction * attractiveForce
        
        // Elmozdulás alkalmazása (hőmérséklet-korláttal)
        FOR EACH node IN graph.nodes:
            displacement = displacements[node]
            magnitude = length(displacement)
            IF magnitude > 0:
                clampedMagnitude = MIN(magnitude, TEMPERATURE)
                coordinates[node] += normalize(displacement) * clampedMagnitude
        
        TEMPERATURE *= COOLING_RATE
    
    RETURN coordinates
```

### 5.4 Grid-re kerekítés (közös lépés)

Bármely elhelyezési stratégia után a folytonos koordinátákat a rácsra kell illeszteni:

```
FUNCTION snapToGrid(
    coordinates: Map<Node, {x, y}>,
    cellSize: number
) -> Map<Node, {x, y}>:
    
    snapped = {}
    occupied = Set()
    
    // Rendezés: a legtöbb éllel rendelkező csomópontokat először
    sortedNodes = sortByDegreeDescending(coordinates.keys())
    
    FOR EACH node IN sortedNodes:
        // Legközelebbi rácspont
        gridX = ROUND(coordinates[node].x / cellSize) * cellSize
        gridY = ROUND(coordinates[node].y / cellSize) * cellSize
        
        // Ütközés-ellenőrzés (figyelembe véve a csomópont méretét)
        WHILE overlapsWithOccupied(gridX, gridY, node.width, node.height, occupied):
            // Spirális keresés a legközelebbi szabad helyre
            (gridX, gridY) = spiralSearch(gridX, gridY, cellSize, occupied, node)
        
        snapped[node] = {x: gridX, y: gridY}
        markOccupied(occupied, gridX, gridY, node.width, node.height, cellSize)
    
    RETURN snapped
```

### 5.5 Subgraph kezelés

A Mermaid subgraph-ok egymásba ágyazható csoportokat alkotnak. A kezelésük a csomópont-elhelyezés részét képezi, mert a subgraph-on belüli csomópontoknak egymás közelében kell lenniük.

#### Döntések
- A subgraph keret **csak vizuális** – az élek szabadon átmehetnek rajta a routing során
- A beágyazás **korlátlan mélységű** (rekurzív)
- A subgraph-oknak **lehetnek saját élei** – ezek a keret szélén kapnak portot (virtuális csomópont)

#### 5.5.1 Subgraph fa felépítése

```
STRUCTURE SubgraphTree:
    id: string
    label: string
    children: SubgraphTree[]      // al-subgraph-ok
    nodes: Node[]                 // közvetlen csomópontok (nem al-subgraph-ban)
    boundingBox: BoundingBox      // kiszámított keret

FUNCTION buildSubgraphTree(graph: Graph) -> SubgraphTree:
    root = new SubgraphTree(id: "ROOT", label: null, children: [], nodes: [])
    
    // Subgraph → szülő subgraph mapping felépítése
    parentMap = {}
    FOR EACH subgraph IN graph.subgraphs:
        parentMap[subgraph.id] = findParentSubgraph(subgraph, graph.subgraphs)
    
    // Fa felépítése
    subgraphNodes = {}
    FOR EACH subgraph IN graph.subgraphs:
        subgraphNodes[subgraph.id] = new SubgraphTree(
            id: subgraph.id,
            label: subgraph.label,
            children: [],
            nodes: getDirectNodes(subgraph, graph)
        )
    
    // Szülő-gyerek kapcsolatok
    FOR EACH subgraph IN graph.subgraphs:
        parent = parentMap[subgraph.id]
        IF parent != null:
            subgraphNodes[parent].children.add(subgraphNodes[subgraph.id])
        ELSE:
            root.children.add(subgraphNodes[subgraph.id])
    
    // Subgraph-on kívüli csomópontok a gyökérhez
    FOR EACH node IN graph.nodes:
        IF NOT isInAnySubgraph(node, graph.subgraphs):
            root.nodes.add(node)
    
    RETURN root

FUNCTION findParentSubgraph(
    child: Subgraph,
    allSubgraphs: Subgraph[]
) -> string | null:
    bestParent = null
    bestSize = INFINITY
    
    FOR EACH candidate IN allSubgraphs:
        IF candidate.id == child.id: CONTINUE
        IF containsAll(candidate.children, child.children):
            IF candidate.children.length < bestSize:
                bestParent = candidate.id
                bestSize = candidate.children.length
    
    RETURN bestParent
```

#### 5.5.2 Rekurzív elhelyezés (bottom-up)

Az elhelyezés a legmélyebb subgraph-októl a gyökér felé halad. Minden szinten a gyerek subgraph-ok **virtuális csomópontként** jelennek meg a szülő elrendezésében.

```
FUNCTION placeWithSubgraphs(
    graph: Graph,
    subgraphTree: SubgraphTree
) -> Map<Node, {x, y}>:
    
    coordinates = {}
    subgraphBoxes = {}
    
    placeSubgraphRecursive(subgraphTree, graph, coordinates, subgraphBoxes)
    
    RETURN coordinates

FUNCTION placeSubgraphRecursive(
    subtree: SubgraphTree,
    graph: Graph,
    coordinates: Map<Node, {x, y}>,
    subgraphBoxes: Map<string, BoundingBox>
):
    SUBGRAPH_PADDING = 40
    LABEL_HEIGHT = 25
    
    // ── 1. Gyerek subgraph-ok rekurzív feldolgozása ──
    FOR EACH child IN subtree.children:
        placeSubgraphRecursive(child, graph, coordinates, subgraphBoxes)
    
    // ── 2. Virtuális csomópontok: közvetlen node-ok + gyerek subgraph-ok ──
    virtualNodes = []
    
    FOR EACH node IN subtree.nodes:
        virtualNodes.add(VirtualNode(
            id: node.id, width: node.width, height: node.height,
            isReal: true, realNode: node
        ))
    
    FOR EACH child IN subtree.children:
        box = subgraphBoxes[child.id]
        virtualNodes.add(VirtualNode(
            id: child.id, width: box.width, height: box.height,
            isReal: false, subgraphId: child.id
        ))
    
    // ── 3. Szinten belüli élek ──
    internalEdges = getEdgesWithinLevel(graph.edges, virtualNodes)
    
    // ── 4. Elhelyezés a diagram típusa szerint ──
    IF virtualNodes.length == 0:
        subgraphBoxes[subtree.id] = BoundingBox(
            x: 0, y: 0,
            width: 2 * SUBGRAPH_PADDING,
            height: 2 * SUBGRAPH_PADDING + LABEL_HEIGHT
        )
        RETURN
    
    localGraph = buildLocalGraph(virtualNodes, internalEdges)
    localCoordinates = placeByDiagramType(localGraph, graph.type, graph.direction)
    
    // ── 5. Lokális → globális koordináták ──
    FOR EACH vNode IN virtualNodes:
        localPos = localCoordinates[vNode.id]
        
        IF vNode.isReal:
            coordinates[vNode.realNode] = {
                x: localPos.x + SUBGRAPH_PADDING,
                y: localPos.y + SUBGRAPH_PADDING + LABEL_HEIGHT
            }
        ELSE:
            offsetX = localPos.x + SUBGRAPH_PADDING
            offsetY = localPos.y + SUBGRAPH_PADDING + LABEL_HEIGHT
            offsetSubgraphContents(vNode.subgraphId, offsetX, offsetY,
                                   coordinates, subgraphBoxes)
    
    // ── 6. Bounding box kiszámítása ──
    IF subtree.id != "ROOT":
        minX = MIN(coord.x FOR coord IN relevantCoordinates) - SUBGRAPH_PADDING
        maxX = MAX(coord.x + node.width FOR ...) + SUBGRAPH_PADDING
        minY = MIN(coord.y FOR coord IN relevantCoordinates) - SUBGRAPH_PADDING - LABEL_HEIGHT
        maxY = MAX(coord.y + node.height FOR ...) + SUBGRAPH_PADDING
        
        subgraphBoxes[subtree.id] = BoundingBox(
            x: minX, y: minY,
            width: maxX - minX, height: maxY - minY
        )

FUNCTION offsetSubgraphContents(
    subgraphId: string,
    offsetX: number,
    offsetY: number,
    coordinates: Map<Node, {x, y}>,
    subgraphBoxes: Map<string, BoundingBox>
):
    box = subgraphBoxes[subgraphId]
    box.x += offsetX
    box.y += offsetY
    
    FOR EACH node IN getNodesInSubgraph(subgraphId):
        coordinates[node].x += offsetX
        coordinates[node].y += offsetY
    
    FOR EACH childId IN getChildSubgraphs(subgraphId):
        offsetSubgraphContents(childId, offsetX, offsetY, coordinates, subgraphBoxes)
```

#### 5.5.3 Subgraph élek feloldása

A Mermaid-ben subgraph-oknak is lehetnek élei (pl. `Frontend --> Backend`). Ezeket **virtuális csomóponttá** alakítjuk: a subgraph bounding box-a lesz a virtuális csomópont mérete, a portok a keret szélére kerülnek.

A virtuális csomópont **nem blokkolja a rácsot** – a többi él szabadon átmehet a subgraph területén.

```
FUNCTION resolveSubgraphEdges(
    graph: Graph,
    subgraphBoxes: Map<string, BoundingBox>
) -> (Edge[], Node[]):
    
    resolvedEdges = []
    virtualNodes = []
    
    FOR EACH edge IN graph.edges:
        sourceIsSubgraph = isSubgraph(edge.source, graph)
        targetIsSubgraph = isSubgraph(edge.target, graph)
        
        IF NOT sourceIsSubgraph AND NOT targetIsSubgraph:
            resolvedEdges.add(edge)
        ELSE:
            resolvedEdge = new Edge(edge)
            
            IF sourceIsSubgraph:
                vNodeId = "virtual_" + edge.source
                resolvedEdge.source = vNodeId
                IF NOT alreadyRegistered(vNodeId, virtualNodes):
                    virtualNodes.add(Node(
                        id: vNodeId,
                        width: subgraphBoxes[edge.source].width,
                        height: subgraphBoxes[edge.source].height,
                        x: subgraphBoxes[edge.source].x,
                        y: subgraphBoxes[edge.source].y,
                        isVirtual: true,
                        blocksGrid: false
                    ))
            
            IF targetIsSubgraph:
                vNodeId = "virtual_" + edge.target
                resolvedEdge.target = vNodeId
                IF NOT alreadyRegistered(vNodeId, virtualNodes):
                    virtualNodes.add(Node(
                        id: vNodeId,
                        width: subgraphBoxes[edge.target].width,
                        height: subgraphBoxes[edge.target].height,
                        x: subgraphBoxes[edge.target].x,
                        y: subgraphBoxes[edge.target].y,
                        isVirtual: true,
                        blocksGrid: false
                    ))
            
            resolvedEdges.add(resolvedEdge)
    
    RETURN (resolvedEdges, virtualNodes)
```

---

## 6. Fázis 3 – Grid paraméterek és felépítés

### Cél
A routing rács méretének és felbontásának meghatározása.

### 6.1 Cellaméret számítás

```
FUNCTION calculateCellSize(nodes: Node[], edges: Edge[]) -> number:
    // A legkisebb komponens méretéből indulunk ki
    minDimension = MIN(
        MIN(node.width FOR node IN nodes),
        MIN(node.height FOR node IN nodes)
    )
    
    // Routing faktor: a gráf sűrűségéhez igazítva
    edgeDensity = edges.length / nodes.length
    
    IF edgeDensity < 1.5:
        R = 4   // ritka gráf
    ELSE IF edgeDensity <= 3.0:
        R = 5   // átlagos
    ELSE:
        R = 6   // sűrű gráf
    
    cellSize = FLOOR(minDimension / R)
    
    // Minimum cellaméret biztosítása
    cellSize = MAX(cellSize, 5)
    
    RETURN cellSize
```

### 6.2 Rács kiterjedés számítás

```
FUNCTION calculateGridExtent(
    coordinates: Map<Node, {x, y}>,
    nodes: Node[],
    edges: Edge[]
) -> {width: number, height: number}:
    
    // Bounding box
    minX = MIN(coordinates[n].x FOR n IN nodes)
    maxX = MAX(coordinates[n].x + n.width FOR n IN nodes)
    minY = MIN(coordinates[n].y FOR n IN nodes)
    maxY = MAX(coordinates[n].y + n.height FOR n IN nodes)
    
    boundingWidth = maxX - minX
    boundingHeight = maxY - minY
    
    // Biztonsági szorzó (K)
    edgeDensity = edges.length / nodes.length
    maxDegree = MAX(degree(n) FOR n IN nodes)
    hasSubgraphs = ANY(n.subgraph != null FOR n IN nodes)
    
    K = 1.5
    IF edgeDensity > 2.0:   K += 0.3
    IF maxDegree > 6:       K += 0.2
    IF hasSubgraphs:        K += 0.2
    
    RETURN {
        width:  CEIL(boundingWidth * K),
        height: CEIL(boundingHeight * K)
    }
```

### 6.3 Grid felépítés

```
FUNCTION buildGrid(
    extent: {width, height},
    cellSize: number,
    coordinates: Map<Node, {x, y}>,
    nodes: Node[]
) -> Grid:
    
    cols = CEIL(extent.width / cellSize)
    rows = CEIL(extent.height / cellSize)
    
    grid = new Grid(rows, cols)
    
    // Alapértelmezett költség minden cellának
    FOR EACH cell IN grid:
        cell.cost = 1.0
        cell.state = FREE
    
    // Komponensek alatti cellák blokkolása
    // FONTOS: virtuális csomópontok (subgraph élek) NEM blokkolnak!
    FOR EACH node IN nodes:
        IF node.isVirtual:
            CONTINUE
        
        pos = coordinates[node]
        startCol = FLOOR(pos.x / cellSize)
        endCol = CEIL((pos.x + node.width) / cellSize)
        startRow = FLOOR(pos.y / cellSize)
        endRow = CEIL((pos.y + node.height) / cellSize)
        
        FOR row IN startRow..endRow:
            FOR col IN startCol..endCol:
                grid[row][col].state = BLOCKED
                grid[row][col].cost = INFINITY
                grid[row][col].owner = node.id
    
    RETURN grid
```

---

## 7. Fázis 4 – Port-kiosztás

### Cél
Minden élhez csatlakozási pontot (portot) rendelni a forrás- és célcsomópont szélén.

### 7.1 Oldal-hozzárendelés

```
FUNCTION assignPorts(
    graph: Graph,
    coordinates: Map<Node, {x, y}>
) -> Map<Edge, {sourcePort: Port, targetPort: Port}>:
    
    portAssignments = {}
    
    // Gyűjtsük össze csomópontonként az éleket
    nodeEdges = groupEdgesByNode(graph.edges)
    
    FOR EACH node IN graph.nodes:
        edges = nodeEdges[node]
        nodeCenter = center(coordinates[node], node.width, node.height)
        
        // Oldalankénti csoportosítás
        sides = { TOP: [], RIGHT: [], BOTTOM: [], LEFT: [] }
        
        FOR EACH edge IN edges:
            // A másik végpont
            otherNode = (edge.source == node.id) ? edge.target : edge.source
            otherCenter = center(coordinates[otherNode], otherNode.width, otherNode.height)
            
            // Szög számítás
            angle = atan2(otherCenter.y - nodeCenter.y, otherCenter.x - nodeCenter.x)
            angleDeg = (degrees(angle) + 360) % 360
            
            // Szektor meghatározás
            side = angleToSide(angleDeg)
            sides[side].add({edge: edge, angle: angleDeg, otherNode: otherNode})
        
        // Túlcsordulás kezelés
        sides = handleOverflow(sides, node)
        
        // Oldalon belüli rendezés
        FOR EACH side IN [TOP, RIGHT, BOTTOM, LEFT]:
            sortEdgesOnSide(sides[side], side)
        
        // Port pozíciók kiosztása
        assignPortPositions(node, coordinates[node], sides, portAssignments)
    
    RETURN portAssignments
```

### 7.2 Szög → oldal konverzió

```
FUNCTION angleToSide(angleDeg: number) -> Side:
    // 0° = jobbra, 90° = le, 180° = balra, 270° = fel
    // (matematikai konvenció, y tengely lefelé nő)
    
    IF angleDeg >= 315 OR angleDeg < 45:
        RETURN RIGHT
    ELSE IF angleDeg >= 45 AND angleDeg < 135:
        RETURN BOTTOM
    ELSE IF angleDeg >= 135 AND angleDeg < 225:
        RETURN LEFT
    ELSE:
        RETURN TOP
```

### 7.3 Túlcsordulás kezelése

```
FUNCTION handleOverflow(
    sides: Map<Side, EdgeList>,
    node: Node
) -> Map<Side, EdgeList>:
    
    // Oldalankénti max port szám (a méret alapján)
    maxPorts = {
        TOP:    FLOOR(node.width / MIN_PORT_SPACING),
        BOTTOM: FLOOR(node.width / MIN_PORT_SPACING),
        LEFT:   FLOOR(node.height / MIN_PORT_SPACING),
        RIGHT:  FLOOR(node.height / MIN_PORT_SPACING)
    }
    
    FOR EACH side IN [TOP, RIGHT, BOTTOM, LEFT]:
        WHILE sides[side].length > maxPorts[side]:
            // A határhoz legközelebb eső szögű élet mozgatjuk
            edgeToMove = findEdgeClosestToBoundary(sides[side], side)
            neighborSide = getClockwiseNeighborSide(side)
            
            sides[side].remove(edgeToMove)
            sides[neighborSide].add(edgeToMove)
    
    RETURN sides
```

### 7.4 Oldalon belüli rendezés

```
FUNCTION sortEdgesOnSide(edges: EdgeList, side: Side):
    // A merőleges tengely mentén rendezünk
    
    SWITCH side:
        CASE TOP, BOTTOM:
            // Cél X koordinátája szerint, balról jobbra
            edges.sortBy(e => coordinates[e.otherNode].x, ASCENDING)
        CASE LEFT, RIGHT:
            // Cél Y koordinátája szerint, fentről lefelé
            edges.sortBy(e => coordinates[e.otherNode].y, ASCENDING)
```

### 7.5 Port pozíciók kiszámítása

```
FUNCTION assignPortPositions(
    node: Node,
    pos: {x, y},
    sides: Map<Side, EdgeList>,
    portAssignments: Map<Edge, Ports>  // output
):
    FOR EACH side IN [TOP, RIGHT, BOTTOM, LEFT]:
        edges = sides[side]
        n = edges.length
        IF n == 0: CONTINUE
        
        FOR i IN 0..n-1:
            fraction = (i + 1) / (n + 1)  // egyenletes elosztás
            
            SWITCH side:
                CASE TOP:
                    portX = pos.x + node.width * fraction
                    portY = pos.y
                CASE BOTTOM:
                    portX = pos.x + node.width * fraction
                    portY = pos.y + node.height
                CASE LEFT:
                    portX = pos.x
                    portY = pos.y + node.height * fraction
                CASE RIGHT:
                    portX = pos.x + node.width
                    portY = pos.y + node.height * fraction
            
            port = {x: portX, y: portY, side: side}
            
            // Port → legközelebbi rácspont
            port.gridX = ROUND(portX / cellSize)
            port.gridY = ROUND(portY / cellSize)
            
            edge = edges[i]
            IF edge.source == node.id:
                portAssignments[edge].sourcePort = port
            ELSE:
                portAssignments[edge].targetPort = port
```

---

## 8. Fázis 5 – Routing prioritás

### Cél
Meghatározni, hogy az élek milyen sorrendben kerüljenek routeolásra.

### Hibrid pontozás

```
FUNCTION calculateRoutingPriority(
    graph: Graph,
    coordinates: Map<Node, {x, y}>,
    grid: Grid
) -> Edge[]:
    
    // Súlyok
    W_DEGREE = 3.0      // fokszám súlya
    W_DISTANCE = 2.0     // távolság súlya
    W_CONGESTION = 1.0   // szomszédos foglaltság súlya
    
    priorities = {}
    
    FOR EACH edge IN graph.edges:
        sourceNode = getNode(edge.source)
        targetNode = getNode(edge.target)
        
        // 1. Fokszám-komponens: magasabb fokszámú csomópontok élei előbb
        maxDegree = MAX(degree(sourceNode), degree(targetNode))
        degreeScore = maxDegree
        
        // 2. Távolság-komponens: távolabbi csomópontok élei előbb
        //    (hogy a legtöbb szabadságot kapják)
        dist = manhattanDistance(
            coordinates[sourceNode],
            coordinates[targetNode]
        )
        distanceScore = dist / grid.maxDimension  // normalizált
        
        // 3. Szomszédsági torlódás: mennyi foglalt rácspont van
        //    a forrás és cél környezetében
        congestionScore = estimateCongestion(
            coordinates[sourceNode],
            coordinates[targetNode],
            grid
        )
        
        priorities[edge] = W_DEGREE * degreeScore
                         + W_DISTANCE * distanceScore
                         + W_CONGESTION * congestionScore
    
    // Csökkenő prioritás szerint rendezés (legmagasabb először)
    RETURN sortDescending(graph.edges, priorities)
```

### Torlódás-becslés

```
FUNCTION estimateCongestion(
    sourcePos: {x, y},
    targetPos: {x, y},
    grid: Grid
) -> number:
    // A forrás-cél közötti téglalap területén
    // megszámoljuk a foglalt rácspontok arányát
    
    minX = MIN(sourcePos.x, targetPos.x) / cellSize
    maxX = MAX(sourcePos.x, targetPos.x) / cellSize
    minY = MIN(sourcePos.y, targetPos.y) / cellSize
    maxY = MAX(sourcePos.y, targetPos.y) / cellSize
    
    totalCells = (maxX - minX + 1) * (maxY - minY + 1)
    blockedCells = 0
    
    FOR row IN minY..maxY:
        FOR col IN minX..maxX:
            IF grid[row][col].state != FREE:
                blockedCells += 1
    
    RETURN blockedCells / totalCells
```

---

## 9. Fázis 6 – Él-routing (A* pathfinding)

### Cél
Minden élhez ortogonális útvonalat találni a rácson, fedés- és (lehetőleg) keresztezésmentesen.

### 9.1 A* algoritmus adaptálva

```
FUNCTION routeEdge(
    grid: Grid,
    sourcePort: Port,
    targetPort: Port
) -> Path | null:
    
    start = {row: sourcePort.gridY, col: sourcePort.gridX}
    goal = {row: targetPort.gridY, col: targetPort.gridX}
    
    openSet = PriorityQueue()
    openSet.add(start, priority: 0)
    
    cameFrom = {}
    gScore = {start: 0}               // tényleges költség eddig
    fScore = {start: heuristic(start, goal)}  // becsült teljes költség
    
    WHILE openSet is not empty:
        current = openSet.popLowest()
        
        IF current == goal:
            RETURN reconstructPath(cameFrom, current)
        
        FOR EACH neighbor IN getNeighbors(current, grid):
            // Mozgás költsége
            moveCost = calculateMoveCost(grid, current, neighbor, cameFrom)
            tentativeG = gScore[current] + moveCost
            
            IF tentativeG < (gScore[neighbor] OR INFINITY):
                cameFrom[neighbor] = current
                gScore[neighbor] = tentativeG
                fScore[neighbor] = tentativeG + heuristic(neighbor, goal)
                
                IF neighbor NOT IN openSet:
                    openSet.add(neighbor, priority: fScore[neighbor])
    
    RETURN null  // Nem található út → zsákutca kezelés
```

### 9.2 Heurisztika

```
FUNCTION heuristic(from: Cell, to: Cell) -> number:
    // Manhattan-távolság (optimista becslés ortogonális rácson)
    RETURN ABS(from.row - to.row) + ABS(from.col - to.col)
```

### 9.3 Költségfüggvény – Az algoritmus szíve

```
CONSTANT BASE_COST = 1.0
CONSTANT BEND_COST = 2.0           // iránytváltás költsége
CONSTANT ADJACENT_COST = 5.0       // foglalt él melletti rácspontra lépés
CONSTANT CROSSING_COST = 50.0      // másik élet keresztező lépés
CONSTANT BLOCKED_COST = INFINITY   // foglalt rácspont (nem átjárható)

FUNCTION calculateMoveCost(
    grid: Grid,
    current: Cell,
    neighbor: Cell,
    cameFrom: Map
) -> number:
    
    // 1. Alapköltség
    cost = BASE_COST
    
    // 2. Blokkolt cella
    IF grid[neighbor.row][neighbor.col].state == BLOCKED:
        RETURN INFINITY
    
    // 3. Foglalt cella (már routeolt él)
    IF grid[neighbor.row][neighbor.col].state == OCCUPIED:
        RETURN INFINITY  // Fedésmentesség garanciája!
    
    // 4. Irányváltás (kanyar) büntetés
    IF cameFrom[current] exists:
        prevDirection = direction(cameFrom[current], current)
        newDirection = direction(current, neighbor)
        IF prevDirection != newDirection:
            cost += BEND_COST
    
    // 5. Szomszédos foglalt él büntetés
    FOR EACH adjacentCell IN getAdjacentCells(neighbor):
        IF grid[adjacentCell.row][adjacentCell.col].state == OCCUPIED:
            cost += ADJACENT_COST
    
    // 6. Keresztezés büntetés (nem tiltott, csak drága)
    IF wouldCrossExistingEdge(current, neighbor, grid):
        cost += CROSSING_COST
    
    RETURN cost
```

### 9.4 Szomszédos cellák (4-irányú mozgás)

```
FUNCTION getNeighbors(cell: Cell, grid: Grid) -> Cell[]:
    neighbors = []
    
    // Csak ortogonális szomszédok (fel, le, balra, jobbra)
    FOR EACH (dRow, dCol) IN [(-1,0), (1,0), (0,-1), (0,1)]:
        newRow = cell.row + dRow
        newCol = cell.col + dCol
        
        IF isInBounds(newRow, newCol, grid):
            neighbors.add({row: newRow, col: newCol})
    
    RETURN neighbors
```

### 9.5 Út lefoglalása

```
FUNCTION commitPath(grid: Grid, path: Path, edgeId: string):
    FOR EACH cell IN path:
        grid[cell.row][cell.col].state = OCCUPIED
        grid[cell.row][cell.col].owner = edgeId
    
    // Szomszédos cellák költségének növelése
    FOR EACH cell IN path:
        FOR EACH adjacent IN getAdjacentCells(cell):
            IF grid[adjacent.row][adjacent.col].state == FREE:
                grid[adjacent.row][adjacent.col].cost += ADJACENT_COST
```

### 9.6 Teljes routing ciklus

```
FUNCTION routeAllEdges(
    graph: Graph,
    grid: Grid,
    portAssignments: Map<Edge, Ports>,
    prioritizedEdges: Edge[]
) -> Map<Edge, Path>:
    
    routes = {}
    
    // ── Többszörös élek detektálása ──
    multiEdgeGroups = detectMultiEdges(graph.edges)
    processedEdges = Set()
    
    FOR EACH edge IN prioritizedEdges:
        IF edge IN processedEdges:
            CONTINUE
        
        groupKey = canonicalKey(edge.source, edge.target)
        
        IF groupKey IN multiEdgeGroups:
            // Többszörös él-csoport → együtt routeoljuk
            edgeGroup = multiEdgeGroups[groupKey]
            groupRoutes = routeMultiEdges(edgeGroup, grid, coordinates)
            routes.merge(groupRoutes)
            processedEdges.addAll(edgeGroup)
        ELSE:
            // Egyszerű él → normál routing
            sourcePort = portAssignments[edge].sourcePort
            targetPort = portAssignments[edge].targetPort
            
            path = routeEdge(grid, sourcePort, targetPort)
            
            IF path != null:
                commitPath(grid, path, edge.id)
                routes[edge] = path
            ELSE:
                routes[edge] = handleDeadlock(grid, edge, routes, portAssignments)
            
            processedEdges.add(edge)
    
    RETURN routes
```

### 9.7 Többszörös élek detektálása

```
FUNCTION detectMultiEdges(edges: Edge[]) -> Map<string, Edge[]>:
    groups = {}
    
    FOR EACH edge IN edges:
        // Irányfüggetlen kulcs (A→B és B→A ugyanaz a csoport)
        key = canonicalKey(edge.source, edge.target)
        
        IF key NOT IN groups:
            groups[key] = []
        groups[key].add(edge)
    
    // Csak a többszörösöket adjuk vissza
    RETURN groups.filter(g => g.length > 1)

FUNCTION canonicalKey(a: string, b: string) -> string:
    IF a < b: RETURN a + "→" + b
    ELSE:     RETURN b + "→" + a
```

### 9.8 Többszörös élek routing-ja

A többszörös élek kezelésének kulcsa: **szomszédos portok kiosztása** + **önálló A* routing** mindegyik élre. A szomszédos portok és az `ADJACENT_COST` együttesen biztosítják, hogy az élek természetesen párhuzamosan fussanak, miközben az A* garantálja a fedés- és keresztezésmentességet.

```
FUNCTION routeMultiEdges(
    edgeGroup: Edge[],
    grid: Grid,
    coordinates: Map<Node, {x, y}>
) -> Map<Edge, Path>:

    routes = {}
    n = edgeGroup.length

    sourceNode = getNode(edgeGroup[0].source)
    targetNode = getNode(edgeGroup[0].target)

    // ── 1. Szomszédos portok kiosztása ──
    side = determineSide(coordinates[sourceNode], coordinates[targetNode])
    sourcePorts = allocateAdjacentPorts(sourceNode, side, n)
    
    oppositeSide = getOppositeSide(side)
    targetPorts = allocateAdjacentPorts(targetNode, oppositeSide, n)

    // ── 2. Routing sorrend: középső él először ──
    //    A középső él kapja a legközvetlenebb utat,
    //    a szélsők szimmetrikusan kifelé routeolódnak.
    middleIdx = FLOOR(n / 2)
    order = [middleIdx]
    FOR offset IN 1..n:
        IF middleIdx - offset >= 0:
            order.add(middleIdx - offset)
        IF middleIdx + offset < n:
            order.add(middleIdx + offset)

    // ── 3. Önálló A* routing mindegyikre ──
    FOR EACH i IN order:
        path = routeEdge(grid, sourcePorts[i], targetPorts[i])
        
        IF path != null:
            commitPath(grid, path, edgeGroup[i].id)
            routes[edgeGroup[i]] = path
        ELSE:
            routes[edgeGroup[i]] = handleDeadlock(
                grid, edgeGroup[i], routes, portAssignments
            )

    RETURN routes
```

---

## 10. Fázis 7 – Zsákutca kezelés

### Cél
Kezelni azokat az eseteket, amikor egy él nem routeolható.

### Többszintű védelem

```
FUNCTION handleDeadlock(
    grid: Grid,
    failedEdge: Edge,
    existingRoutes: Map<Edge, Path>,
    portAssignments: Map<Edge, Ports>
) -> Path:
    
    // ── 1. SZINT: Rip-up and Reroute ──
    path = ripUpAndReroute(grid, failedEdge, existingRoutes, portAssignments)
    IF path != null:
        RETURN path
    
    // ── 2. SZINT: Rács növelés ──
    path = expandGridAndRetry(grid, failedEdge, existingRoutes, portAssignments)
    IF path != null:
        RETURN path
    
    // ── 3. SZINT: Keresztezés engedélyezése (fallback) ──
    RETURN routeWithCrossingsAllowed(grid, failedEdge, portAssignments)
```

### 10.1 Rip-up and Reroute

```
FUNCTION ripUpAndReroute(
    grid: Grid,
    failedEdge: Edge,
    existingRoutes: Map<Edge, Path>,
    portAssignments: Map<Edge, Ports>,
    maxIterations: int = 5
) -> Path | null:
    
    FOR iteration IN 1..maxIterations:
        // Azonosítsuk a blokkoló éleket
        blockingEdges = findBlockingEdges(grid, failedEdge, portAssignments)
        
        IF blockingEdges.isEmpty():
            RETURN null  // Nem élek blokkolják → más a probléma
        
        // Blokkoló élek visszavonása
        FOR EACH blockingEdge IN blockingEdges:
            uncommitPath(grid, existingRoutes[blockingEdge])
            existingRoutes.remove(blockingEdge)
        
        // Sikertelen él routeolása (most több hely van)
        path = routeEdge(grid, 
            portAssignments[failedEdge].sourcePort,
            portAssignments[failedEdge].targetPort)
        
        IF path != null:
            commitPath(grid, path, failedEdge.id)
            
            // Visszavont élek újra-routeolása
            allSuccess = true
            FOR EACH blockingEdge IN blockingEdges:
                rePath = routeEdge(grid,
                    portAssignments[blockingEdge].sourcePort,
                    portAssignments[blockingEdge].targetPort)
                
                IF rePath != null:
                    commitPath(grid, rePath, blockingEdge.id)
                    existingRoutes[blockingEdge] = rePath
                ELSE:
                    allSuccess = false
                    BREAK
            
            IF allSuccess:
                RETURN path
            ELSE:
                // Visszaállítás és újra próba
                uncommitPath(grid, path)
                // ... restore previous state
        
    RETURN null  // maxIterations elérve
```

### 10.2 Blokkoló élek azonosítása

```
FUNCTION findBlockingEdges(
    grid: Grid,
    failedEdge: Edge,
    portAssignments: Map<Edge, Ports>
) -> Edge[]:
    
    source = portAssignments[failedEdge].sourcePort
    target = portAssignments[failedEdge].targetPort
    
    // A* futtatása úgy, hogy OCCUPIED cellákat is meglátogatjuk
    // (de magas költséggel), és megjegyezzük mely éleket érintjük
    touchedEdges = Set()
    
    // Módosított A*: OCCUPIED cellák átjárhatók, de feljegyezzük
    path = routeEdgeWithTracking(grid, source, target, touchedEdges)
    
    // A leggyakrabban érintett élek a blokkolók
    RETURN touchedEdges.sortByFrequency().take(3)
```

### 10.3 Rács növelés

```
FUNCTION expandGridAndRetry(
    grid: Grid,
    failedEdge: Edge,
    existingRoutes: Map<Edge, Path>,
    portAssignments: Map<Edge, Ports>
) -> Path | null:
    
    EXPANSION_FACTOR = 1.5
    
    // Új, nagyobb rács
    newGrid = new Grid(
        rows: CEIL(grid.rows * EXPANSION_FACTOR),
        cols: CEIL(grid.cols * EXPANSION_FACTOR)
    )
    
    // Koordináták arányos skálázása
    scaleAllCoordinates(coordinates, EXPANSION_FACTOR)
    
    // Portok újraszámolása
    newPortAssignments = assignPorts(graph, coordinates)
    
    // Grid újrafelépítése
    newGrid = buildGrid(extent, cellSize, coordinates, nodes)
    
    // Teljes újra-routing
    newRoutes = routeAllEdges(graph, newGrid, newPortAssignments, prioritizedEdges)
    
    IF failedEdge IN newRoutes AND newRoutes[failedEdge] != null:
        // Globális állapot frissítése
        grid = newGrid
        existingRoutes = newRoutes
        portAssignments = newPortAssignments
        RETURN newRoutes[failedEdge]
    
    RETURN null
```

### 10.4 Keresztezéses fallback

```
FUNCTION routeWithCrossingsAllowed(
    grid: Grid,
    failedEdge: Edge,
    portAssignments: Map<Edge, Ports>
) -> Path:
    
    // Módosított A*: OCCUPIED cellák átjárhatók (magas, de véges költséggel)
    // A keresztezési pontokat megjegyezzük a vizuális jelöléshez
    
    modifiedGrid = grid.clone()
    FOR EACH cell IN modifiedGrid:
        IF cell.state == OCCUPIED:
            cell.cost = CROSSING_COST  // Nem INFINITY, hanem magas véges költség
    
    path = routeEdge(modifiedGrid,
        portAssignments[failedEdge].sourcePort,
        portAssignments[failedEdge].targetPort)
    
    // Keresztezési pontok megjelölése
    FOR EACH cell IN path:
        IF grid[cell.row][cell.col].state == OCCUPIED:
            cell.crossing = true  // SVG rendering-nél híd/ív lesz
    
    commitPath(grid, path, failedEdge.id)
    RETURN path
```

---

## 11. Fázis 8 – Él címke elhelyezés

### Cél
Az élcímkéket utólagosan, a routing befejezése után elhelyezni a diagram középső szegmensénél, ütközésmentesen.

### Döntés
A címkék **nem akadályok a routing fázisban** – a routing algoritmust nem terhelik. Ehelyett a routing befejezése után egy önálló fázis helyezi el őket. Ez egyszerűsíti a routing algoritmust, miközben az intelligens címke-elhelyezés biztosítja, hogy a címkék ne takarjanak el más elemeket.

### 11.1 Címke elhelyezés

```
FUNCTION placeAllLabels(
    routes: Map<Edge, Path>,
    allNodes: Node[],
    coordinates: Map<Node, {x, y}>
) -> List<LabelPlacement>:

    placements = []
    placedBBoxes = []  // már elhelyezett címkék bounding box-ai

    FOR EACH (edge, path) IN routes:
        IF edge.label == null:
            CONTINUE
        
        placement = placeLabelOnEdge(
            edge, path, allNodes, routes, placedBBoxes, coordinates
        )
        
        IF placement != null:
            placements.add(placement)

    RETURN placements
```

### 11.2 Egyedi címke elhelyezés

```
FUNCTION placeLabelOnEdge(
    edge: Edge,
    path: SimplifiedPath,
    allNodes: Node[],
    allRoutes: Map<Edge, Path>,
    placedLabels: List<BoundingBox>,
    coordinates: Map<Node, {x, y}>
) -> LabelPlacement:

    label = edge.label
    LABEL_PADDING = 4

    // ── 1. Legjobb szegmens kiválasztása ──
    segments = getSegments(path)
    labelWidth = measureTextWidth(label, fontSize: 12)
    labelHeight = measureTextHeight(label, fontSize: 12)
    segment = selectBestSegment(segments, labelWidth)

    // ── 2. Szegmens irányának meghatározása ──
    segmentDirection = getSegmentDirection(segment)
    segmentMidpoint = midpoint(segment.start, segment.end)

    // ── 3. Pozíció-jelöltek generálása ──
    candidates = []

    IF segmentDirection == HORIZONTAL:
        // Fölötte
        candidates.add({
            x: segmentMidpoint.x - labelWidth / 2,
            y: segmentMidpoint.y - labelHeight - LABEL_PADDING,
            side: "above"
        })
        // Alatta
        candidates.add({
            x: segmentMidpoint.x - labelWidth / 2,
            y: segmentMidpoint.y + LABEL_PADDING,
            side: "below"
        })
    ELSE:  // VERTICAL
        // Balra
        candidates.add({
            x: segmentMidpoint.x - labelWidth - LABEL_PADDING,
            y: segmentMidpoint.y - labelHeight / 2,
            side: "left"
        })
        // Jobbra
        candidates.add({
            x: segmentMidpoint.x + LABEL_PADDING,
            y: segmentMidpoint.y - labelHeight / 2,
            side: "right"
        })

    // ── 4. Legjobb pozíció kiválasztása (ütközésmentes) ──
    FOR EACH candidate IN candidates:
        bbox = BoundingBox(
            x: candidate.x,
            y: candidate.y,
            width: labelWidth + 2 * LABEL_PADDING,
            height: labelHeight + 2 * LABEL_PADDING
        )

        IF NOT collides(bbox, allNodes, allRoutes, placedLabels, coordinates):
            placedLabels.add(bbox)
            RETURN LabelPlacement(
                text: label, x: candidate.x, y: candidate.y, side: candidate.side
            )

    // ── 5. Ha mindkét oldal ütközik: eltolás a szegmens mentén ──
    RETURN slideLabelAlongSegment(
        segment, label, labelWidth, labelHeight,
        allNodes, allRoutes, placedLabels, coordinates
    )
```

### 11.3 Szegmens kiválasztás

```
FUNCTION selectBestSegment(segments: Segment[], labelWidth: number) -> Segment:
    // Preferencia: középső > leghosszabb > bármelyik
    middleIdx = FLOOR(segments.length / 2)
    
    IF length(segments[middleIdx]) >= labelWidth:
        RETURN segments[middleIdx]
    
    // Ha a középső túl rövid, a leghosszabbat választjuk
    longest = segments.sortByLength(DESCENDING)[0]
    RETURN longest
```

### 11.4 Ütközés-detektálás

```
FUNCTION collides(
    bbox: BoundingBox,
    allNodes: Node[],
    allRoutes: Map<Edge, Path>,
    placedLabels: List<BoundingBox>,
    coordinates: Map<Node, {x, y}>
) -> boolean:

    // 1. Csomópontokkal
    FOR EACH node IN allNodes:
        nodeBBox = BoundingBox(
            x: coordinates[node].x, y: coordinates[node].y,
            width: node.width, height: node.height
        )
        IF overlaps(bbox, nodeBBox):
            RETURN true

    // 2. Más élekkel
    FOR EACH (edge, path) IN allRoutes:
        FOR EACH segment IN getSegments(path):
            IF bboxIntersectsSegment(bbox, segment):
                RETURN true

    // 3. Már elhelyezett címkékkel
    FOR EACH placedBBox IN placedLabels:
        IF overlaps(bbox, placedBBox):
            RETURN true

    RETURN false
```

### 11.5 Eltolás a szegmens mentén

```
FUNCTION slideLabelAlongSegment(
    segment: Segment,
    label: string,
    labelWidth: number,
    labelHeight: number,
    allNodes: Node[],
    allRoutes: Map<Edge, Path>,
    placedLabels: List<BoundingBox>,
    coordinates: Map<Node, {x, y}>
) -> LabelPlacement:

    segmentLength = length(segment)
    STEP = 5  // pixelben
    LABEL_PADDING = 4

    // A középponttól kifelé haladva keresünk szabad helyet
    FOR offset IN [0, STEP, -STEP, 2*STEP, -2*STEP, ...]:
        IF ABS(offset) > segmentLength / 2:
            BREAK

        position = pointAlongSegment(segment, 0.5 + offset / segmentLength)
        segDir = getSegmentDirection(segment)
        
        sides = (segDir == HORIZONTAL) ? ["above", "below"] : ["left", "right"]
        
        FOR EACH side IN sides:
            candidate = calculateCandidatePosition(
                position, side, labelWidth, labelHeight, LABEL_PADDING
            )
            bbox = BoundingBox(candidate.x, candidate.y,
                             labelWidth + 2 * LABEL_PADDING,
                             labelHeight + 2 * LABEL_PADDING)

            IF NOT collides(bbox, allNodes, allRoutes, placedLabels, coordinates):
                placedLabels.add(bbox)
                RETURN LabelPlacement(
                    text: label, x: candidate.x, y: candidate.y, side: side
                )

    // Végső fallback: eredeti középpont (elfogadva az ütközést)
    RETURN LabelPlacement(
        text: label,
        x: midpoint(segment).x - labelWidth / 2,
        y: midpoint(segment).y - labelHeight - LABEL_PADDING,
        side: "above"
    )
```

---

## 12. Fázis 9 – SVG rendering

### Cél
A grid-en routeolt útvonalakat vizuális SVG kimenetté alakítani.

### 12.1 Csomópontok renderelése

```
FUNCTION renderNodes(
    nodes: Node[],
    coordinates: Map<Node, {x, y}>
) -> SVGElements:
    
    elements = []
    
    FOR EACH node IN nodes:
        pos = coordinates[node]
        
        SWITCH node.shape:
            CASE "rect":
                elements.add(SVGRect(
                    x: pos.x, y: pos.y,
                    width: node.width, height: node.height,
                    rx: 0  // nincs lekerekítés
                ))
            CASE "rounded":
                elements.add(SVGRect(
                    x: pos.x, y: pos.y,
                    width: node.width, height: node.height,
                    rx: 8  // lekerekített sarkok
                ))
            CASE "diamond":
                elements.add(SVGPolygon(
                    // rombusz pontok
                ))
            // ... egyéb alakzatok
        
        // Címke
        elements.add(SVGText(
            x: pos.x + node.width / 2,
            y: pos.y + node.height / 2,
            text: node.label,
            anchor: "middle",
            dominantBaseline: "central"
        ))
    
    RETURN elements
```

### 12.2 Élek renderelése – Ortogonális polyline

```
FUNCTION renderEdges(
    routes: Map<Edge, Path>,
    cellSize: number
) -> SVGElements:
    
    elements = []
    
    FOR EACH (edge, path) IN routes:
        // Grid cellák → pixel koordináták
        points = []
        FOR EACH cell IN path:
            points.add({
                x: cell.col * cellSize + cellSize / 2,
                y: cell.row * cellSize + cellSize / 2
            })
        
        // Egyszerűsítés: collinear pontok eltávolítása
        simplified = simplifyPath(points)
        
        // SVG polyline generálás lekerekített sarkokkal
        pathData = generateRoundedPolyline(simplified, cornerRadius: 5)
        
        elements.add(SVGPath(
            d: pathData,
            stroke: edgeColor(edge),
            strokeWidth: edgeWidth(edge),
            fill: "none",
            strokeDasharray: dashPattern(edge)  // dotted, dashed, stb.
        ))
        
        // Nyílhegy
        IF edge.arrowHead != "none":
            lastSegment = getLastSegment(simplified)
            elements.add(renderArrowHead(lastSegment, edge.arrowHead))
        
        // Él címke (a Fázis 8-ban meghatározott pozícióból)
        IF edge.label != null AND labelPlacements[edge] exists:
            placement = labelPlacements[edge]
            elements.add(SVGText(
                x: placement.x, y: placement.y,
                text: placement.text,
                fontSize: 12,
                anchor: "middle"
            ))
        
        // Keresztezési pontok vizuális jelölése
        FOR EACH cell IN path:
            IF cell.crossing:
                elements.add(renderCrossingBridge(cell, cellSize))
    
    RETURN elements
```

### 12.3 Útvonal egyszerűsítés

```
FUNCTION simplifyPath(points: Point[]) -> Point[]:
    // Collinear (egy vonalon lévő) pontok eltávolítása
    // Csak a töréspontokat tartjuk meg
    
    IF points.length <= 2:
        RETURN points
    
    simplified = [points[0]]
    
    FOR i IN 1..points.length-2:
        prev = points[i-1]
        curr = points[i]
        next = points[i+1]
        
        // Ha az irány változik, megtartjuk a pontot
        dirPrev = direction(prev, curr)
        dirNext = direction(curr, next)
        
        IF dirPrev != dirNext:
            simplified.add(curr)
    
    simplified.add(points[points.length - 1])
    RETURN simplified
```

### 12.4 Lekerekített sarkok

```
FUNCTION generateRoundedPolyline(points: Point[], cornerRadius: number) -> string:
    // SVG path data generálás lekerekített sarkokkal
    
    IF points.length < 2:
        RETURN ""
    
    pathData = "M " + points[0].x + " " + points[0].y
    
    FOR i IN 1..points.length-2:
        prev = points[i-1]
        curr = points[i]
        next = points[i+1]
        
        // A sarokpont előtt és után egy-egy pont a sugár távolságára
        r = MIN(cornerRadius, 
                distance(prev, curr) / 2,
                distance(curr, next) / 2)
        
        // Sarok előtti pont
        beforeCorner = moveTowards(curr, prev, r)
        // Sarok utáni pont
        afterCorner = moveTowards(curr, next, r)
        
        pathData += " L " + beforeCorner.x + " " + beforeCorner.y
        pathData += " Q " + curr.x + " " + curr.y + " " 
                         + afterCorner.x + " " + afterCorner.y
    
    pathData += " L " + points[points.length-1].x + " " + points[points.length-1].y
    
    RETURN pathData
```

### 12.5 Keresztezési híd

```
FUNCTION renderCrossingBridge(cell: Cell, cellSize: number) -> SVGElement:
    // Kis ív/híd a keresztezési ponton
    // Jelzi az olvasónak, hogy a két vonal nem kapcsolódik
    
    centerX = cell.col * cellSize + cellSize / 2
    centerY = cell.row * cellSize + cellSize / 2
    bridgeRadius = cellSize * 0.4
    
    // Fehér háttér kör (eltakarja a mögöttes vonalat)
    RETURN SVGGroup([
        SVGCircle(cx: centerX, cy: centerY, r: bridgeRadius, 
                  fill: "white", stroke: "none"),
        SVGArc(...)  // kis ív a felülhaladó vonal irányában
    ])
```

### 12.6 Subgraph keretek renderelése

```
FUNCTION renderSubgraphs(
    subgraphTree: SubgraphTree,
    subgraphBoxes: Map<string, BoundingBox>
) -> SVGElements:
    
    elements = []
    renderSubgraphRecursive(subgraphTree, subgraphBoxes, elements, depth: 0)
    RETURN elements

FUNCTION renderSubgraphRecursive(
    subtree: SubgraphTree,
    subgraphBoxes: Map<string, BoundingBox>,
    elements: SVGElements,
    depth: int
):
    IF subtree.id == "ROOT":
        FOR EACH child IN subtree.children:
            renderSubgraphRecursive(child, subgraphBoxes, elements, depth)
        RETURN
    
    box = subgraphBoxes[subtree.id]
    BORDER_RADIUS = 8
    bgColor = getSubgraphColor(depth)
    
    // Háttér
    elements.add(SVGRect(
        x: box.x, y: box.y,
        width: box.width, height: box.height,
        rx: BORDER_RADIUS,
        fill: bgColor,
        stroke: darken(bgColor, 30),
        strokeWidth: 1.5,
        strokeDasharray: "5,3"
    ))
    
    // Címke (bal felső sarok)
    elements.add(SVGText(
        x: box.x + 8, y: box.y + 16,
        text: subtree.label,
        fontSize: 13, fontWeight: "bold",
        fill: darken(bgColor, 60)
    ))
    
    // Gyerekek rekurzív renderelése
    FOR EACH child IN subtree.children:
        renderSubgraphRecursive(child, subgraphBoxes, elements, depth + 1)

FUNCTION getSubgraphColor(depth: int) -> string:
    colors = [
        "#f0f4f8",   // 0. szint – legvilágosabb
        "#e2e8f0",   // 1. szint
        "#cbd5e1",   // 2. szint
        "#94a3b8"    // 3+ szint – legsötétebb
    ]
    RETURN colors[MIN(depth, colors.length - 1)]
```

### 12.7 Teljes renderelési sorrend (z-order)

```
FUNCTION renderAll(
    nodes, coordinates, routes, labelPlacements,
    subgraphTree, subgraphBoxes, cellSize
) -> SVG:
    svgElements = []
    
    // 1. Subgraph háttérek (legalul)
    svgElements.addAll(renderSubgraphs(subgraphTree, subgraphBoxes))
    
    // 2. Élek (középen)
    svgElements.addAll(renderEdges(routes, cellSize, labelPlacements))
    
    // 3. Csomópontok (felül)
    svgElements.addAll(renderNodes(nodes, coordinates))
    
    // 4. Címkék (legfelül)
    svgElements.addAll(renderLabels(labelPlacements))
    
    RETURN buildSVG(svgElements)
```

---

## 13. Dekompozíció (nagy gráfok kezelése)

### Cél
100+ csomópontos gráfok kezelése, ahol a teljes grid-routing túl lassú vagy az eredmény áttekinthetetlen lenne.

### Konfiguráció

```
STRUCTURE TrellisConfig:
    decomposition: "NONE" | "SINGLE" | "MULTI"
    decompositionThreshold: number    // felhasználó által állítható küszöb
                                      // default: 50
```

### Három mód

| Mód | Viselkedés | Felhasználási eset |
|---|---|---|
| **NONE** | Nincs dekompozíció, teljes gráf egyben, lassú renderelés elfogadva | Szabályozott iparágak (pl. orvosi), ahol a diagram struktúrája nem módosítható |
| **SINGLE** | Egyetlen diagram, kétszintű routing (klaszterenkénti belső + globális) | Általános használat, MI kimenet |
| **MULTI** | Több diagram: összesítő + klaszterenkénti részletek | Navigálható dokumentáció, nagy rendszerek |

### 13.1 Pipeline elágazás

```
FUNCTION renderDiagram(graph: Graph, config: TrellisConfig) -> Output:
    nodeCount = graph.nodes.length
    
    IF config.decomposition == "NONE" 
       OR nodeCount <= config.decompositionThreshold:
        
        // NONE mód vagy küszöb alatt: standard pipeline
        IF config.decomposition == "NONE" AND nodeCount > 50:
            estimatedTime = estimateRenderTime(nodeCount, graph.edges.length)
            IF estimatedTime > 5000:
                emitWarning("Becsült renderelési idő: ~" + 
                            CEIL(estimatedTime / 1000) + " másodperc.")
        
        RETURN standardPipeline(graph, config)
    
    ELSE IF config.decomposition == "SINGLE":
        RETURN renderSingleWithDecomposition(graph, config)
    
    ELSE IF config.decomposition == "MULTI":
        RETURN renderMultiWithDecomposition(graph, config)

FUNCTION estimateRenderTime(nodeCount: number, edgeCount: number) -> number:
    avgGridSize = (nodeCount * 50) ^ 2
    avgPathfindTime = avgGridSize * 0.001
    RETURN edgeCount * avgPathfindTime
```

### 13.2 Klaszterezés

A klaszterezés a SINGLE és MULTI módok közös alapja. Három esetet kezel:

```
FUNCTION hybridClustering(
    graph: Graph,
    targetClusterSize: number
) -> Map<Node, string>:
    
    clusterOf = {}
    
    // 1. Subgraph-ban lévő csomópontok: a subgraph = klaszter
    //    (a szerző szándékát tiszteletben tartjuk)
    FOR EACH subgraph IN graph.subgraphs:
        FOR EACH nodeId IN subgraph.children:
            clusterOf[nodeId] = "subgraph_" + subgraph.id
    
    // 2. Maradék csomópontok: automatikus klaszterezés (Louvain)
    unassigned = graph.nodes.filter(n => n.id NOT IN clusterOf)
    
    IF unassigned.length > 0:
        subGraph = buildSubgraph(unassigned, graph.edges)
        autoClusters = detectClusters(subGraph, targetClusterSize)
        
        FOR EACH (node, cluster) IN autoClusters:
            clusterOf[node] = "auto_" + cluster
    
    RETURN clusterOf
```

#### Louvain közösségdetektálás

```
FUNCTION detectClusters(
    graph: Graph,
    targetClusterSize: number
) -> Map<Node, string>:
    
    // Modularitás-alapú: sűrűn összekötött csomópontok egy klaszterbe
    clusterOf = {}
    
    FOR EACH node IN graph.nodes:
        clusterOf[node] = node.id
    
    improved = true
    WHILE improved:
        improved = false
        
        FOR EACH node IN graph.nodes (véletlenszerű sorrendben):
            currentCluster = clusterOf[node]
            bestCluster = currentCluster
            bestModularityGain = 0
            
            neighborClusters = getNeighborClusters(node, graph, clusterOf)
            
            FOR EACH candidateCluster IN neighborClusters:
                // Klaszterméret korlát
                clusterSize = countNodesInCluster(candidateCluster, clusterOf)
                IF clusterSize >= targetClusterSize * 1.5:
                    CONTINUE
                
                gain = calculateModularityGain(
                    node, candidateCluster, graph, clusterOf
                )
                
                IF gain > bestModularityGain:
                    bestModularityGain = gain
                    bestCluster = candidateCluster
            
            IF bestCluster != currentCluster:
                clusterOf[node] = bestCluster
                improved = true
    
    RETURN clusterOf

FUNCTION calculateModularityGain(
    node: Node, targetCluster: string,
    graph: Graph, clusterOf: Map<Node, string>
) -> number:
    // ΔQ modularitás-nyereség (Louvain standard formula)
    m = graph.edges.length
    k_i = degree(node, graph)
    k_i_in = countEdgesToCluster(node, targetCluster, graph, clusterOf)
    sigma_in = sumInternalEdges(targetCluster, clusterOf, graph)
    sigma_tot = sumTotalEdges(targetCluster, clusterOf, graph)
    
    gain = (sigma_in + 2 * k_i_in) / (2 * m)
         - ((sigma_tot + k_i) / (2 * m)) ^ 2
         - sigma_in / (2 * m)
         + (sigma_tot / (2 * m)) ^ 2
         + (k_i / (2 * m)) ^ 2
    
    RETURN gain
```

### 13.3 SINGLE mód – Kétszintű routing

```
FUNCTION renderSingleWithDecomposition(
    graph: Graph,
    config: TrellisConfig
) -> SVG:
    
    // ── 1. Klaszterezés ──
    clusters = hybridClustering(graph, config.decompositionThreshold)
    clusterGroups = groupByClusters(graph.nodes, clusters)
    
    // ── 2. Klaszterenkénti belső routing ──
    clusterResults = {}
    
    FOR EACH (clusterId, nodes) IN clusterGroups:
        internalEdges = graph.edges.filter(e =>
            clusters[e.source] == clusterId AND
            clusters[e.target] == clusterId
        )
        localGraph = buildSubgraph(nodes, internalEdges)
        clusterResults[clusterId] = standardPipeline(localGraph, config)
    
    // ── 3. Klaszterek elhelyezése ──
    clusterBoxes = {}
    FOR EACH (clusterId, result) IN clusterResults:
        clusterBoxes[clusterId] = result.boundingBox
    
    clusterEdges = getInterClusterEdges(graph.edges, clusters)
    clusterGraph = buildClusterGraph(clusterBoxes, clusterEdges)
    clusterPositions = placeByDiagramType(clusterGraph, graph.type, graph.direction)
    
    // ── 4. Belső koordináták eltolása ──
    FOR EACH (clusterId, result) IN clusterResults:
        offset = clusterPositions[clusterId]
        offsetAllCoordinates(result, offset)
    
    // ── 5. Klaszterek közti élek routing-ja ──
    globalGrid = buildGlobalGrid(clusterResults, clusterPositions)
    interClusterRoutes = routeInterClusterEdges(
        clusterEdges, globalGrid, clusterResults
    )
    
    // ── 6. Egyetlen SVG ──
    RETURN assembleSVG(clusterResults, interClusterRoutes)
```

### 13.4 MULTI mód – Több diagram

```
FUNCTION renderMultiWithDecomposition(
    graph: Graph,
    config: TrellisConfig
) -> DiagramSet:
    
    // ── 1-2. Klaszterezés + belső routing ──
    clusters = hybridClustering(graph, config.decompositionThreshold)
    clusterGroups = groupByClusters(graph.nodes, clusters)
    
    clusterResults = {}
    FOR EACH (clusterId, nodes) IN clusterGroups:
        internalEdges = getInternalEdges(graph.edges, clusterId, clusters)
        localGraph = buildSubgraph(nodes, internalEdges)
        clusterResults[clusterId] = standardPipeline(localGraph, config)
    
    // ── 3. Összesítő diagram ──
    overviewGraph = new Graph()
    
    FOR EACH (clusterId, result) IN clusterResults:
        overviewGraph.nodes.add(Node(
            id: clusterId,
            label: clusterId + "\n(" + result.nodeCount + " elem)",
            width: 120, height: 60
        ))
    
    interClusterEdges = getInterClusterEdges(graph.edges, clusters)
    edgeCounts = countEdgesBetweenClusters(interClusterEdges, clusters)
    
    FOR EACH ((fromCluster, toCluster), count) IN edgeCounts:
        overviewGraph.edges.add(Edge(
            source: fromCluster, target: toCluster,
            label: count + " kapcsolat"
        ))
    
    overviewResult = standardPipeline(overviewGraph, config)
    
    // ── 4. Kimenet ──
    RETURN DiagramSet(
        overview: overviewResult.svg,
        details: {clusterId: result.svg FOR (clusterId, result) IN clusterResults}
    )
```

---

## 14. Piaci pozícionálás

### Versenytársak

| Megoldás | Layout típus | Nyelv | Licenc | Grid routing |
|----------|-------------|-------|--------|-------------|
| **Mermaid + Dagre** | Sugiyama (egyszerű) | Mermaid | MIT (ingyenes) | ❌ |
| **TALA (D2)** | Ortogonális + heurisztikus | D2 | Fizetős, zárt | Részben |
| **yFiles** | Ortogonális, sok algoritmus | Saját API | Kereskedelmi | ✅ |
| **ELK** | Rétegezett | Saját API | EPL | ❌ |
| **Trellis** | Grid + A* pathfinding | Mermaid | Freemium | ✅ |

### Egyedi értékajánlat

1. **Mermaid szintaxis kompatibilitás** – az egyetlen jobb-layout megoldás, ami natívan fogadja a Mermaid kódot
2. **Fedésmentesség by construction** – matematikai garancia, nem heurisztika
3. **MI-barát** – LLM-ek kimenete azonnal renderelhető ügyfélminőségben
4. **VLSI routing adaptáció** – jól kutatott algoritmus, új alkalmazási területen

### Technológiai stack

```
┌──────────────────────────────────────────────────┐
│            Trellis Core (Rust)                  │
│    parser + elhelyezés + routing + render         │
│                   │                               │
│          ┌────────┼────────┐                      │
│          │        │        │                      │
│     ┌────▼────┐ ┌─▼──┐ ┌──▼─────┐               │
│     │  WASM   │ │CLI │ │ natív  │               │
│     └────┬────┘ │bin │ │ lib    │               │
│    ┌─────┼────┐ └─┬──┘ └──┬─────┘               │
│ ┌──▼───┐ ┌──▼──┐ │       │                      │
│ │VSCode│ │InteJ│ │       │                      │
│ │ (JS) │ │(Kt) │ │       │                      │
│ └──────┘ └─────┘ │       │                      │
│                   │       │                      │
│              ┌────▼───────▼────┐                 │
│              │  Docker / CI/CD │                 │
│              └─────────────────┘                 │
└──────────────────────────────────────────────────┘
```

| Komponens | Technológia | Indoklás |
|-----------|------------|----------|
| **Core** | Rust → WASM + natív | Platform-független, gyors, mindkét IDE-ben WASM-ként futtatható |
| **CLI** | Rust (natív bináris) | Tesztelés, CI/CD, Docker – nincs WASM overhead |
| **VS Code plugin** | TypeScript + WASM | A WASM modult JS-ből hívja |
| **IntelliJ plugin** | Kotlin + WASM | A WASM modult JVM WASM runtime-ból hívja |
| **Parser** | Rust (saját) | PEG-alapú Mermaid parser a core részeként |

### Disztribúció

| Csatorna | Formátum | Célcsoport |
|----------|---------|-----------|
| VS Code Marketplace | VS Code Extension (.vsix) | VS Code felhasználók |
| JetBrains Marketplace | IntelliJ Plugin (.zip) | IntelliJ/WebStorm/stb. felhasználók |
| GitHub Releases | CLI bináris (Linux/macOS/Windows) | Fejlesztők, CI/CD, Docker |
| Docker Hub | Docker image | Automatizált pipeline-ok, szerviz |
| crates.io | `trellis-cli` crate | Rust fejlesztők (`cargo install`) |

### CLI alkalmazás

#### Cél

Parancssori eszköz, amely Mermaid fájlokat renderel SVG/PNG kimenetté. Két fő felhasználási eset:

1. **Fejlesztői tesztelés** – a core algoritmus gyors tesztelése IDE plugin nélkül
2. **CI/CD és Docker** – automatizált diagram generálás build pipeline-okban, dokumentáció-generátorokban

#### Használat

```bash
# Alap: Mermaid fájl → SVG
trellis render input.mmd -o output.svg

# PNG kimenet
trellis render input.mmd -o output.png

# Stdin → stdout (pipe-olható)
cat input.mmd | trellis render - -f svg > output.svg

# Konfiguráció
trellis render input.mmd -o output.svg \
    --decomposition single \
    --decomposition-threshold 50

# Metrikák kiírása (teszteléshez)
trellis render input.mmd -o output.svg --metrics
# → stdout: {"crossings": 2, "bends": 14, "render_ms": 120, "grid_utilization": 0.34}

# Batch mód (könyvtár összes .mmd fájlja)
trellis render-batch ./diagrams/ -o ./output/ -f svg

# Verzió és licenc info
trellis --version
trellis license --status
trellis license --activate <kulcs>
```

#### Parancsok

| Parancs | Leírás |
|---|---|
| `render <input> -o <output>` | Egyetlen Mermaid fájl renderelése |
| `render-batch <dir> -o <dir>` | Könyvtár összes .mmd fájljának renderelése |
| `license --status` | Aktuális licenc állapot kiírása |
| `license --activate <kulcs>` | Premium licenc aktiválása |
| `license --deactivate` | Licenc deaktiválása (gépváltáshoz) |
| `validate <input>` | Mermaid szintaxis ellenőrzés renderelés nélkül |

#### Kapcsolók

| Kapcsoló | Alapértelmezés | Leírás |
|---|---|---|
| `-o, --output <fájl>` | stdout | Kimeneti fájl (kiterjesztés határozza meg a formátumot) |
| `-f, --format <svg\|png>` | svg | Kimeneti formátum (ha stdout-ra megy) |
| `--decomposition <none\|single\|multi>` | single | Dekompozíciós mód |
| `--decomposition-threshold <n>` | 50 | Dekompozíciós küszöb |
| `--metrics` | false | Metrikák kiírása JSON-ben a stderr-re |
| `--quiet` | false | Csak hibák kiírása |
| `--config <fájl>` | ~/.trellis/config.toml | Konfigurációs fájl |

#### Freemium a CLI-ben

A CLI ugyanazt a freemium modellt követi, mint az IDE pluginek:

| Funkció | Free | Premium |
|---------|------|---------|
| Csomópontok száma | Max 10 | Korlátlan |
| Kimenet formátum | PNG (vízjellel) | PNG + SVG (vízjel nélkül) |
| Batch mód | ❌ | ✅ |
| `--metrics` | ✅ (teszteléshez mindig elérhető) | ✅ |

A licenc aktiválás a Lemon Squeezy License API-n keresztül történik (ugyanaz, mint a VS Code pluginnál). A licencállapot lokálisan tárolódik: `~/.trellis/license.json`.

```
# Aktiválás
trellis license --activate 38b1460a-5104-4067-a91d-77b872934d51
# → License activated. Tier: premium, Expires: 2027-02-08

# Állapot ellenőrzés
trellis license --status
# → Tier: premium, Expires: 2027-02-08, Machine: vscode-abc123
```

#### Docker használat

```dockerfile
FROM ghcr.io/trellis/trellis:latest

# Mermaid fájlok bemásolása
COPY diagrams/ /data/input/

# Renderelés
RUN trellis render-batch /data/input/ -o /data/output/ -f svg
```

```bash
# Egyszeri renderelés docker-rel
docker run --rm -v $(pwd):/data ghcr.io/trellis/trellis:latest \
    trellis render /data/input.mmd -o /data/output.svg

# CI/CD pipeline-ban (pl. GitHub Actions)
- name: Render diagrams
  uses: docker://ghcr.io/trellis/trellis:latest
  with:
    args: trellis render-batch ./docs/diagrams/ -o ./docs/rendered/ -f svg
```

#### Implementáció

A CLI közvetlenül a `trellis-core` Rust crate-et hívja – **nincs WASM réteg**. Ez natív sebességet és minimális bináris méretet ad.

```
trellis-core (Rust lib)
       │
  ┌────┼────────────┐
  │    │             │
  ▼    ▼             ▼
CLI  WASM         natív lib
(natív Rust)  (IDE pluginok)  (jövőbeli integrációk)
```

### Licencelés – Freemium modell

#### Funkciótáblázat

| Funkció | Free | Premium |
|---------|------|---------|
| Csomópontok száma | Max 10 | Korlátlan |
| Kimenet formátum | PNG (csak) | PNG + SVG + draw.io |
| Vízjel | Igen | Nem |
| Színsémák | Alapértelmezett | Választható |
| Routing minőség | Teljes | Teljes |

#### Architektúra döntés

A **WASM core nem tartalmaz licenc-logikát** – mindig teljes funkcionalitást ad. A korlátozások (node limit, vízjel, export formátumok) kizárólag a plugin rétegben vannak.

```
┌────────────────────────────────────────────────┐
│          Trellis Core (WASM)                  │
│   Nincs licenc-logika – mindig teljes           │
│   funkcionalitás                                │
└──────────────────┬─────────────────────────────┘
                   │
        ┌──────────┼──────────┐
   ┌────▼────┐          ┌────▼──────────┐
   │ VS Code │          │  IntelliJ     │
   │ license │          │  license      │
   │ .ts     │          │  .kt          │
   │         │          │               │
   │ RSA     │          │ LicensingFa-  │
   │ +Lemon  │          │ cade (natív)  │
   │ Squeezy │          │               │
   └─────────┘          └───────────────┘
```

Indoklás: a platformonként eltérő licencelési mechanizmusok így egymástól függetlenül kezelhetők, és a core algoritmus tesztelhetősége nem függ licenc-állapottól.

#### Platform-specifikus validáció

| Platform | Mechanizmus | Indoklás |
|---|---|---|
| **IntelliJ** | JetBrains Marketplace natív licencelés (LicensingFacade API) | Beépített infrastruktúra, a JetBrains kezeli a fizetést, nincs plusz fejlesztés |
| **VS Code** | RSA-aláírt kulcsok + külső fizetési szolgáltató (pl. Lemon Squeezy) | Nincs beépített VS Code licencelési API, saját megoldás szükséges |

#### IntelliJ – Natív Marketplace licencelés

A JetBrains standard kódot biztosít, amely a platform szintjén ellenőrzi a licencet:

```
plugin.xml:
  <product-descriptor code="TRELLIS" release-date="..." release-version="..."
                      optional="true"/>
  // optional=true → freemium mód: a plugin betölt, de a fizetős funkciók 
  //                 aktiválásánál ellenőrizzük a licencet
```

A `LicensingFacade` API-n keresztül:
- Aláírt megerősítés a JetBrains szerverről (online vagy on-premises)
- Nincs privát kulcs a platformban (nem kinyerhető)
- Offline kulcsok is támogatottak

#### VS Code – RSA kriptográfiai validáció

Nincs VS Code Marketplace licencelési API, ezért saját megoldás:

```
Licenckulcs generálás (szerveren, privát kulccsal):
  adat = { email, tier: "premium", lejárat: "2027-01-01", verzió: "1.x" }
  aláírás = RSA_SIGN(privát_kulcs, adat)
  kulcs = BASE64(adat) + "." + BASE64(aláírás)
  → Kulcs eljut a felhasználóhoz (email)

Validáció (pluginban, publikus kulccsal):
  (adat, aláírás) = PARSE(kulcs)
  RSA_VERIFY(publikus_kulcs, adat, aláírás)
  → Érvényes: tier, lejárat, verzió kiolvasható
  → Érvénytelen: Free módban marad
```

A publikus kulcs a pluginba van beágyazva. A privát kulcs soha nem hagyja el a szervert.

**Mit kódolunk a kulcsba:**
- Email cím (azonosítás)
- Tier (premium)
- Lejárati dátum
- Maximális verzió (opcionális, ha verzió-specifikus licencelés kell)

**Hibrid mód (online + offline cache):**
1. Induláskor megpróbálja online ellenőrizni (visszavonás detektálás)
2. Ha nincs internet → cache-elt validáció (RSA aláírás ellenőrzés)
3. Grace period: 30 nap offline használat az utolsó sikeres online ellenőrzés után

**Fizetési szolgáltató:** Lemon Squeezy (vagy Gumroad) – kezeli a fizetést, generálja a kulcsot webhook-on keresztül.

#### Védelmi szintek

| Fenyegetés | Védelem | Megjegyzés |
|---|---|---|
| Kulcsgenerálás | RSA aláírás (2048-bit) | Privát kulcs nélkül nem generálható érvényes kulcs |
| Kulcsmegosztás | Online aktiválási limit + email kötés | Lemon Squeezy activation limit |
| Kód patching | Obfuszkáció (IntelliJ: ProGuard, VS Code: minifikáció) | Nem teljes védelem, de megnehezíti |
| Visszafejtés | A core WASM-ben van (nehezebb decompile-olni mint JS) | WASM bináris formátum |

Megjegyzés: egyetlen kliens-oldali licencelés sem 100%-os. A cél a "józan szintű" védelem – az egyszerű másolást megakadályozni, miközben a legális felhasználókat nem zavarjuk.

#### Fizetési integráció – Vásárlástól a kulcsbeírásig

A cél: a vevő a lehető legkevesebb lépéssel fizessen és használja a prémium funkciót.

##### IntelliJ – Teljesen JetBrains-kezelésű folyamat

A JetBrains Marketplace a teljes fizetési infrastruktúrát biztosítja: checkout, fizetésfeldolgozás, számlázás, visszatérítés, kedvezmények, viszonteladói hálózat.

```
┌─────────────────────────────────────────────────────────┐
│  IntelliJ – Vásárlási folyamat (0 fejlesztés)          │
│                                                         │
│  1. Felhasználó telepíti a plugint                      │
│     └─> IDE "License Activation" ablak jelenik meg      │
│         └─> "Evaluate for free" (trial) VAGY            │
│             "Buy license" gomb                          │
│                                                         │
│  2. "Buy license" → JetBrains Marketplace Pricing oldal │
│     └─> Felhasználó fizet (kártya / PayPal / banki)     │
│         JetBrains kezeli: számla, adó, valuta            │
│                                                         │
│  3. Fizetés után automatikus:                           │
│     └─> Licenckulcs kötődik a JetBrains Account-hoz    │
│     └─> IDE-ben Help → Register → automatikus aktiválás │
│         (JetBrains Account bejelentkezéssel)             │
│                                                         │
│  Fejlesztői teendő: SEMMI a fizetéshez.                 │
│  Csak a LicensingFacade API hívás kell a pluginban.     │
└─────────────────────────────────────────────────────────┘
```

**Licencelési modellek (JetBrains Marketplace):**

| Modell | Leírás | Ajánlott? |
|---|---|---|
| Éves/havi előfizetés fallback nélkül | Csak aktív előfizetéssel használható | ❌ |
| Éves/havi előfizetés fallback-kel | Lejárat után az utolsó verzió örökre használható | ✅ Ajánlott |
| Örökös (perpetual) licenc | Egyszeri fizetés, örök hozzáférés | Megfontolandó |

**Jutalék:** 15% (max 25%, de nem emelhetik hirtelen). Viszonteladói jutalék a JetBrains részéből megy, nem a fejlesztőéből.

**Fejlesztőként szükséges:**
1. `plugin.xml`-ben `<product-descriptor>` deklaráció
2. `LicensingFacade` ellenőrzés a kódban
3. Plugin feltöltés a Marketplace-re "Paid" megjelöléssel
4. Pénzfelvétel beállítása (USD/EUR, banki utalás)

##### VS Code – Lemon Squeezy integráció

A VS Code Marketplace-nek nincs fizetési infrastruktúrája, ezért külső szolgáltató kell. A Lemon Squeezy beépített licenckulcs-generálást ad, nem kell saját szerver a kulcsokhoz.

```
┌─────────────────────────────────────────────────────────────────┐
│  VS Code – Vásárlási folyamat (Lemon Squeezy)                  │
│                                                                 │
│  ┌──────────┐  ┌──────────────┐  ┌────────────┐  ┌──────────┐ │
│  │ Plugin   │  │ Lemon Squeezy│  │  Email     │  │  Plugin  │ │
│  │ "Buy"    │─>│  Checkout    │─>│  + kulcs   │─>│ Activate │ │
│  │ gomb     │  │  (hosted)    │  │  (auto)    │  │          │ │
│  └──────────┘  └──────────────┘  └────────────┘  └──────────┘ │
│                                                                 │
│   1. kattintás   2. fizetés       3. ~30 mp       4. beírja    │
│                                                                 │
│  Teljes idő: fizetéstől a használatig < 1 perc                 │
└─────────────────────────────────────────────────────────────────┘
```

**Részletes folyamat:**

**1. lépés – "Buy Premium" gomb a pluginban**

A plugin egy statusbar gombot vagy parancsot biztosít, ami megnyitja a Lemon Squeezy checkout URL-t:

```
// Lemon Squeezy checkout URL (hosted fizetési oldal)
https://trellis.lemonsqueezy.com/buy/xxxxx

// Opcionálisan custom data a checkout-ban (felhasználó azonosítás):
?checkout[custom][vscode_machine_id]=abc123
?checkout[email]=user@example.com   // előre kitölti
```

A felhasználónak **nem kell regisztrálnia** – a Lemon Squeezy hosted checkout kezeli a fizetést (kártya, PayPal). Az EU-s áfát is automatikusan számolja.

**2. lépés – Fizetés után automatikus kulcskézbesítés**

A Lemon Squeezy a termék beállításban bekapcsolt "Generate License Keys" opcióval automatikusan:
- Generál egy egyedi licenckulcsot (pl. `38b1460a-5104-4067-a91d-77b872934d51`)
- Elküldi emailben a vevőnek a nyugtával együtt
- A vevő a "My Orders" oldalon is megtalálja

Nincs szükség webhook-ra vagy szerverre ehhez a lépéshez – a Lemon Squeezy ezt natívan kezeli.

**3. lépés – Kulcs beírása a pluginba**

A plugin "Activate License" parancsot biztosít. A felhasználó beírja (vagy beilleszti) a kulcsot.

**4. lépés – Validáció (plugin → Lemon Squeezy License API)**

```
Plugin aktiváláskor:
  POST https://api.lemonsqueezy.com/v1/licenses/activate
  Body: {
    license_key: "38b1460a-5104-4067-a91d-77b872934d51",
    instance_name: "VSCode-{machine_id}"
  }

  Válasz (siker):
  {
    "activated": true,
    "license_key": {
      "status": "active",
      "activation_limit": 3,
      "activation_usage": 1,
      "expires_at": "2027-02-08T00:00:00Z"
    },
    "meta": {
      "store_id": 12345,
      "product_id": 67890,
      "customer_email": "user@example.com"
    }
  }

Plugin ellenőrzi:
  ✓ store_id == Trellis store (hard-coded)
  ✓ product_id == Trellis Premium (hard-coded)
  ✓ status == "active"
  → Premium mód bekapcsol
  → Válasz cache-elése (offline grace period-hoz)
```

**5. lépés – Periodikus revalidáció**

```
Minden plugin induláskor:
  POST https://api.lemonsqueezy.com/v1/licenses/validate
  Body: { license_key: "..." }

  Ha valid → cache frissítés, premium mód
  Ha expired → free módra visszaállás, értesítés
  Ha nincs internet → cache-ből olvas (max 30 nap)
```

**Lemon Squeezy konfiguráció (egyszeri beállítás):**

| Beállítás | Érték | Indoklás |
|---|---|---|
| Product name | "Trellis Premium" | |
| Generate License Keys | ✅ Bekapcsolva | Auto kulcsgenerálás |
| License length | 1 Year (vagy Unlimited) | Éves megújítás |
| Activation limit | 3 | Max 3 gép/felhasználó |
| Price | $X / év | |
| Tax handling | ✅ Lemon Squeezy kezeli | EU VAT automatikus |

**Webhook (opcionális, haladó):**

Webhook nem szükséges az alapfolyamathoz, mert a Lemon Squeezy natívan generálja és küldi a kulcsot. Webhook akkor hasznos, ha:
- Saját adatbázisban is nyilván akarod tartani a vásárlásokat
- Egyedi kulcsformátumot akarsz (RSA-aláírt kulcs a Lemon Squeezy kulcs helyett)
- Automatikus értesítéseket akarsz küldeni (pl. Slack, Discord)

```
Webhook használat esetén:
  Lemon Squeezy → POST webhook → saját szerver
  Event: "order_created"
  Szerver: RSA-aláírt kulcsot generál → emailben küldi a vevőnek
```

##### Összehasonlítás: a két platform vásárlási élménye

| Szempont | IntelliJ (JetBrains) | VS Code (Lemon Squeezy) |
|---|---|---|
| **Fizetés** | JetBrains kezeli | Lemon Squeezy kezeli |
| **Kulcskézbesítés** | Automatikus (JB Account) | Email (automatikus) |
| **Aktiválás** | Automatikus (JB Account login) | Kulcs beírása |
| **Kattintás a fizetéstől** | 3-4 klikk | 4-5 klikk |
| **Idő a fizetéstől az aktiválásig** | ~30 mp | ~1 perc |
| **Szerver szükséges?** | ❌ Nem | ❌ Nem (Lemon Squeezy API elég) |
| **Fejlesztői munka** | Minimális (LicensingFacade) | Közepes (API integráció) |
| **Jutalék** | 15% | 5% + 50¢/trx |
| **EU adókezelés** | ✅ JetBrains kezeli | ✅ Lemon Squeezy kezeli |

##### Szerver nélküli architektúra

Fontos döntés: **nincs szükség saját szerverre** az alapműködéshez.

```
┌─────────────────────────────────────────────────┐
│  "No backend" architektúra                      │
│                                                 │
│  IntelliJ plugin ──> JetBrains Marketplace API  │
│                      (licenc validáció)          │
│                                                 │
│  VS Code plugin ───> Lemon Squeezy License API  │
│                      (aktiválás + validáció)     │
│                                                 │
│  Saját szerver: NINCS                            │
│  Adatbázis: NINCS                                │
│  DevOps: NINCS                                   │
└─────────────────────────────────────────────────┘
```

Mindkét platform esetében a fizetési szolgáltató API-ja végzi a validációt. A plugin csak HTTP kéréseket küld, és a választ cache-eli lokálisan. Ez minimalizálja az üzemeltetési költséget és komplexitást.

---

## 15. Tesztelési stratégia

### 15.1 Unit tesztek – fázisonként

#### Fázis 1 – Parser

| Teszt | Bemenet | Elvárt kimenet |
|---|---|---|
| Egyszerű flowchart | `graph TD; A-->B` | 2 node, 1 edge, direction=TD |
| Minden node shape | `A[rect] B(rounded) C{diamond} D((circle))` | Helyes shape minden node-ra |
| Él típusok | `A-->B; A-.->B; A==>B` | solid, dotted, thick |
| Él címkék | `A-->\|"label"\|B` | edge.label == "label" |
| Subgraph | `subgraph X; A; B; end` | 1 subgraph, 2 children |
| Beágyazott subgraph | `subgraph X; subgraph Y; A; end; end` | Fa: X→Y→A |
| Üres subgraph | `subgraph X; end` | 0 children, valid parse |
| Class diagram | `class Animal { +name: string }` | 1 node, 1 attribútum |
| ER diagram | `CUSTOMER \|\|--o{ ORDER : places` | 2 entity, 1 relationship |
| Hibás szintaxis | `graph TD; A-->` | Parse error (nem crash) |

#### Fázis 2 – Csomópont-elhelyezés

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Lineáris lánc (TB) | `A→B→C→D` | layers: A=0, B=1, C=2, D=3 |
| Elágazás | `A→B, A→C` | B és C azonos rétegben |
| Ciklus | `A→B→C→A` | Ciklus megtörve, valid rétegek |
| Barycenter sorrend | `A→D, B→D, C→E` | D és E sorrendje a szülők pozíciójából |
| Subgraph belső elrendezés | Subgraph 3 node-dal | Node-ok a subgraph padding-en belül |
| Beágyazott subgraph méretek | 2 szint mély subgraph | Belső bbox < külső bbox |
| Nincs átfedés | Bármely gráf | Semelyik két node koordináta nem fed |
| Direction LR | `graph LR; A→B` | A.x < B.x, A.y == B.y |

#### Fázis 3 – Grid

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Cellaméret ritka gráf | 5 node, 3 edge | R=4 faktor |
| Cellaméret sűrű gráf | 5 node, 20 edge | R=6 faktor |
| Blokkolt cellák | 1 node (100×50 px) | Megfelelő cellák BLOCKED |
| Virtuális node nem blokkol | Subgraph éllel | Subgraph alatti cellák FREE |
| Grid kiterjedés | Bármely gráf | Grid befogadja az összes node-ot + margó |

#### Fázis 4 – Port-kiosztás

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Egyetlen él lefelé | A(fent) → B(lent) | A: BOTTOM port, B: TOP port |
| Egyetlen él jobbra | A(bal) → B(jobb) | A: RIGHT port, B: LEFT port |
| 4 irányba élek | A→B, A→C, A→D, A→E (4 irány) | 4 különböző oldalon port |
| Túlcsordulás | Keskeny node, 10 él egy oldalra | Szomszédos oldalra áthelyezés |
| Oldalon belüli sorrend | A→B, A→C (B balra, C jobbra) | Portok bal→jobb sorrendben |
| Többszörös élek portjai | A→B 3 éllel | 3 szomszédos port azonos oldalon |

#### Fázis 5 – Routing prioritás

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Magas fokszám előre | Csillag gráf (1 hub + 5 levél) | Hub élei magasabb prioritásúak |
| Távolabb előre | Rövid + hosszú él | Hosszú él magasabb prioritás |
| Determinisztikus sorrend | Ugyanaz a gráf kétszer | Azonos prioritási sorrend |

#### Fázis 6 – Routing (A*)

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Egyenes út | A fent, B lent, semmi közte | Egyenes vertikális path |
| Akadály kerülés | A→B, köztük C blokkolja | Path kikerüli C-t |
| Fedésmentesség | 2 routeolt él | Semelyik cella nem OCCUPIED kétszer |
| Kanyar-minimalizálás | Szabad rács, A→B átlósan | Kevesebb kanyar preferált (BEND_COST) |
| Szomszédos él taszítás | 2 párhuzamos él | Legalább 1 cella távolság |
| Többszörös él (2) | A→B kétszer | 2 külön path, párhuzamosan |
| Többszörös él (3) | A→B háromszor | Középső legegyenesebb, szélsők szimmetrikusak |
| Keresztezés jelölés | K₃,₃ teljes páros gráf | Crossing flag-ek a metszéspontokon |

#### Fázis 7 – Zsákutca kezelés

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Rip-up and reroute | Szűk rács, 3. él elakad | 1-2. él újra-routeolva, 3. sikeres |
| Grid növelés | Minden rip-up sikertelen | Grid 1.5× nagyobb, újra-route |
| Fallback keresztezés | Semmi sem segít | Path létrejön crossing=true cellákkal |
| Nincs végtelenhurok | Lehetetlen eset | Max iteráció után fallback-re vált |

#### Fázis 8 – Címke elhelyezés

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Címke középső szegmensen | Egyenes horizontális él + címke | Címke a szegmens fölött/alatt |
| Nincs átfedés node-dal | Címke + közeli node | Címke bbox nem fedi a node bbox-ot |
| Nincs átfedés más címkével | 2 közeli él címkével | 2 címke bbox nem fed |
| Slide fallback | Mindkét oldal foglalt | Eltolva a szegmens mentén |

#### Fázis 9 – SVG rendering

| Teszt | Bemenet | Ellenőrzés |
|---|---|---|
| Válid SVG | Bármely pipeline kimenet | Wellformed XML, SVG namespace |
| Node renderelés | rect, rounded, diamond | Megfelelő SVG elem (rect, polygon) |
| Él polyline | 3 kanyaros path | SVG path `M...L...Q...L` formátum |
| Lekerekített sarkok | Kanyaros path | Q (quadratic bezier) parancsok |
| Crossing híd | Crossing flag-es cella | Fehér kör + ív SVG elemek |
| Subgraph keret | 1 subgraph | Szaggatott rect, címke, háttérszín |
| Beágyazott subgraph szín | 3 szint mély | Mélyebb = sötétebb háttér |
| Z-order | Subgraph + élek + node-ok | Subgraph legalul, node-ok legfelül |

### 15.2 Integrációs tesztek

Teljes pipeline: Mermaid szöveg → SVG kimenet. Minden sikeres renderelés után az alábbi invariánsokat automatikusan ellenőrizzük:

```
FUNCTION validateOutput(svg: SVG, graph: Graph, routes: Map<Edge, Path>):
    
    // I1. Fedésmentesség: egyetlen rácspont sem tartozik 2 élhez
    FOR EACH cell IN grid:
        ASSERT cell.ownerCount <= 1
            "FAIL: Cella ({cell.row}, {cell.col}) {cell.ownerCount} élhez tartozik"
    
    // I2. Minden él routeolva
    FOR EACH edge IN graph.edges:
        ASSERT edge IN routes
            "FAIL: {edge.source}→{edge.target} él nincs routeolva"
    
    // I3. Minden node renderelve
    FOR EACH node IN graph.nodes:
        ASSERT svgContainsElement(svg, node.id)
            "FAIL: {node.id} node nincs az SVG-ben"
    
    // I4. Címkés élek címkéje megjelenik
    FOR EACH edge IN graph.edges:
        IF edge.label != null:
            ASSERT svgContainsText(svg, edge.label)
                "FAIL: '{edge.label}' címke nem jelenik meg"
    
    // I5. Válid SVG
    ASSERT isWellFormedXML(svg)
    ASSERT hasCorrectSVGNamespace(svg)
    
    // I6. Nincs node átfedés
    FOR EACH (nodeA, nodeB) IN allNodePairs:
        ASSERT NOT overlaps(bbox(nodeA), bbox(nodeB))
            "FAIL: {nodeA.id} és {nodeB.id} átfed"
    
    // I7. Subgraph-ok tartalmazzák a node-jaikat
    FOR EACH subgraph IN graph.subgraphs:
        subgraphBox = subgraphBoxes[subgraph.id]
        FOR EACH nodeId IN subgraph.children:
            nodeBox = bbox(nodes[nodeId])
            ASSERT contains(subgraphBox, nodeBox)
                "FAIL: {nodeId} kívül esik {subgraph.id} keretén"
```

### 15.3 Benchmark diagramok

Ismert nehéz esetek gyűjteménye, amelyeken a layout minőséget és teljesítményt mérjük.

| # | Név | Leírás | Nehézség | Mit tesztel |
|---|---|---|---|---|
| B1 | **Lineáris lánc** | `A→B→C→D→E` | Könnyű | Alapvető rétegezés, egyenes élek |
| B2 | **Széles elágazás** | 1 node → 8 node | Könnyű | Port-kiosztás sok port egy oldalon |
| B3 | **K₃,₃ teljes páros** | 3 frontend → 3 backend, minden mindegyikhez | Nehéz | Nem síkgráf, keresztezés elkerülhetetlen |
| B4 | **Gyémánt** | `A→B, A→C, B→D, C→D` | Közepes | Barycenter sorrend, konvergens élek |
| B5 | **Csillag** | 1 hub ↔ 10 levél | Közepes | Sok port egy node-on, routing torlódás |
| B6 | **Többszörös élek** | `A→B` háromszor (HTTP, gRPC, health) | Közepes | Párhuzamos routing, port-szomszédság |
| B7 | **Mély lánc + visszaél** | `A→B→C→D→E→A` (ciklus) | Közepes | Ciklustörés + visszaél routing |
| B8 | **3 szint subgraph** | Cloud > Backend > Services > 3 node | Közepes | Rekurzív elhelyezés, beágyazott keretek |
| B9 | **Subgraph élek** | 2 subgraph, `Frontend→Backend` + belső élek | Közepes | Virtuális csomópont, keret-port |
| B10 | **50 node flowchart** | 50 node, ~80 él, vegyes struktúra | Nehéz | Teljesítmény, dekompozíciós küszöb |
| B11 | **100 node ER** | 100 entitás, N:M relációk | Nagyon nehéz | Force-directed + SINGLE dekompozíció |
| B12 | **Osztályhierarchia** | 5 szint öröklődés + 15 asszociáció | Nehéz | Hibrid elhelyezés, sűrű laterális élek |

#### Mérési metrikák a benchmarkokon

| Metrika | Leírás | Cél |
|---|---|---|
| **Keresztezések száma** | Hány élpár metszi egymást | Minimalizálni |
| **Fedések száma** | Hány él fut azonos útvonalon | 0 (invariáns) |
| **Kanyarok száma** | Összes irányváltás az éleken | Minimalizálni |
| **Renderelési idő** | Parse-tól SVG-ig (ms) | <1s a B1-B9-re, <5s a B10-B12-re |
| **Grid kihasználtság** | Foglalt cellák / összes cella | Informatív (nem cél) |

---

## 16. Nyitott kérdések

A következő iterációban tisztázandó kérdések:

### Algoritmikus
1. **A* költségfüggvény finomhangolása** – a konstansok (BEND_COST, ADJACENT_COST, CROSSING_COST) optimális értékei empirikus tesztelést igényelnek a benchmark diagramokon (lásd 15.3)

### Lezárt döntések (nem nyitott kérdés)
- ~~Él címkék~~ → Utólagos elhelyezés, routing után (Fázis 8)
- ~~Többszörös élek~~ → Szomszédos portok + önálló A* routing (Fázis 6.7-6.8)
- ~~Subgraph kezelés~~ → Vizuális keret, rekurzív, virtuális csomópontok subgraph élekhez (Fázis 2.5)
- ~~Dekompozíció~~ → NONE/SINGLE/MULTI felhasználói választás (Fázis 13)
- ~~Inkrementális routing~~ → Nem szükséges. Teljes újraszámolás elfogadható
- ~~Mermaid parser~~ → Saját parser (nem Mermaid fork)
- ~~Technológiai stack~~ → Rust core + WASM (lásd 14. szekció)
- ~~Licencelés~~ → Freemium + Premium (lásd 14. szekció)
- ~~Disztribúció~~ → IDE pluginok: VS Code + IntelliJ (lásd 14. szekció)
- ~~Tesztelési stratégia~~ → Unit + integrációs + benchmark (lásd 15. szekció)
