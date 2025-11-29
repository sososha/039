# CAD Core Specification — Objects & Operations (Architectural scope)

## 1️⃣ Object Model (概要)

- Document
  - `id: DocumentId`
  - buildings: [BuildingId]

- Building
  - `id: BuildingId`
  - levels: [LevelId]
  - grids: [GridId]
  - elements: [ElementId]

- Level
  - `id: LevelId`
  - elevation: f64
  - name: String

- Grid
  - `id: GridId`
  - axes_x: [GridAxis]
  - axes_y: [GridAxis]

- Element (抽象)
  - `id: ElementId`
  - kind: ElementKind (Wall/Floor/Roof/Column/Beam/Openings...)
  - placement: Placement (参照レベル/グリッド/ローカル座標)
  - geom_ref: GeometryRef (BRep/CSG/Sketchへの参照)

## 2️⃣ Operations (APIレベルのイメージ)

- Document/Building
  - `create_document() -> DocumentId`
  - `add_building(doc: DocumentId) -> BuildingId`

- Level
  - `add_level(building: BuildingId, elevation: f64, name: &str) -> LevelId`
  - `set_level_elevation(LevelId, f64)`

- Grid
  - `add_grid_axis_x(building: BuildingId, label: &str, offset: f64)`
  - `add_grid_axis_y(building: BuildingId, label: &str, offset: f64)`

- Elements
  - `create_wall(building: BuildingId, level: LevelId, start: Point3, end: Point3, thickness: f64, height: f64) -> ElementId`
  - `create_slab(building: BuildingId, level: LevelId, polygon: [Point3], thickness: f64) -> ElementId`
  - `create_column(building: BuildingId, level: LevelId, position: Point3, section: SectionSpec, height: f64) -> ElementId`
  - `create_opening(host: ElementId, profile: OpeningProfile) -> ElementId`

- Editing
  - `move_element(ElementId, Vector3)`
  - `rotate_element(ElementId, Axis, angle: f64)`
  - `change_type(ElementId, new_type)`

## 3️⃣ Geometry & KernelShape

- GeometryRef
  - 内部的には BRep/CSG/Sketch を保持。
  - 各 Element は 1つ以上の `KernelShape` 実装を提供し、Rendering Core にメッシュ/ポリラインを渡す。

- KernelShape 要件
  - `fn tessellate(&self, params: &TessParams) -> MeshData`
  - f64 のBRepから f32 MeshData を生成し、誤差は TessParams 以内。

## 4️⃣ Selection / Mapping

- Mapping Tables
  - `ElementId -> [EntityId]` (レンダリングのEntity)
  - `EntityId -> (ElementId, Option<FaceId>, Option<EdgeId>)`

- 選択の基本単位は ElementId。必要に応じて FaceId/EdgeId レベルまで降りる。

## 5️⃣ Error / Consistency

- 操作前条件が満たされない場合、明示的なエラーを返す（例: 無効な LevelId や host のない Opening など）。
- 幾何的に不正な編集（自己交差など）はコアが検出し、かならずエラーを返すか、明示的な invalid 状態に遷移させる。
