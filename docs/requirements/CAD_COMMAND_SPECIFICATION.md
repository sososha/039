# CAD Command Specification Reference

This document serves as a functional specification list for CAD command development. It outlines the standard commands, their expected inputs, and behaviors, independent of specific implementation details.

## 1. Drawing Commands (作図コマンド)

| Command | Description | Inputs / Interaction | Note |
| :--- | :--- | :--- | :--- |
| **Line** (線分) | Creates straight line segments. | Point 1 (Start) -> Point 2 (End) | Continuous mode allows chaining. |
| **Polyline** (連続線) | Creates a sequence of connected lines as a single entity. | P1 -> P2 -> P3... -> Enter/Close | Can be open or closed. |
| **Double Line** (二重線) | Creates two parallel lines simultaneously. | Width -> Start Point -> End Point | Used for walls, etc. |
| **Circle** (円) | Creates a circle. | Center + Radius / 3 Points / 2 Points (Diameter) / Tangent+Radius | |
| **Arc** (円弧) | Creates a portion of a circle. | 3 Points / Center + Start + End / Start + End + Radius | |
| **Rectangle** (矩形) | Creates a rectangular polyline. | Corner 1 -> Corner 2 (Diagonal) / Center + Corner | |
| **Polygon** (多角形) | Creates a regular polygon. | Center -> Radius (Inscribed/Circumscribed) + Side Count | |
| **Spline** (スプライン) | Creates a smooth curve passing through or near points. | Control Points P1...Pn -> Enter | NURBS or Bezier based. |
| **Hatch** (ハッチング) | Fills a closed area with a pattern. | Select Boundary / Pick Internal Point | Solid, Pattern (Lines, Cross, Dots). |
| **Text** (文字) | Creates text annotations. | Insert Point -> Height -> Rotation -> Content | Single line or Multi-line. |
| **Dimension** (寸法) | Creates dimension lines. | Point 1 -> Point 2 -> Text Location | Linear, Aligned, Angular, Radial. |

## 2. Modification Commands (編集コマンド)

| Command | Description | Inputs / Interaction | Note |
| :--- | :--- | :--- | :--- |
| **Move** (移動) | Moves selected entities. | Select -> Base Point -> Destination Point | |
| **Copy** (複写) | Copies selected entities. | Select -> Base Point -> Destination Point | Multiple copy mode. |
| **Array** (配列複写) | Creates multiple copies in a pattern. | Select -> Type (Rect/Polar/Path) -> Params (Rows/Cols/Angle) | |
| **Rotate** (回転) | Rotates entities around a point. | Select -> Center Point -> Angle (or Reference Points) | |
| **Scale** (伸縮/尺度) | Resizes entities relative to a point. | Select -> Base Point -> Scale Factor | |
| **Stretch** (伸縮) | Stretches entities by moving vertices within a selection window. | Window Select (Crossing) -> Base Point -> Displacement | Only moves vertices inside window. |
| **Mirror** (鏡像) | Creates a mirrored copy. | Select -> Mirror Line (2 Points) | Option to keep original. |
| **Offset** (オフセット) | Creates a parallel copy at a distance. | Distance -> Select Object -> Side to Offset | |
| **Trim** (トリム) | Cuts an object at a cutting edge. | Select Cutting Edge -> Select Object to Trim | |
| **Extend** (延長) | Extends an object to a boundary. | Select Boundary Edge -> Select Object to Extend | |
| **Fillet** (フィレット) | Rounds the corner between two lines. | Radius -> Select Line 1 -> Select Line 2 | |
| **Chamfer** (面取り) | Bevels the corner between two lines. | Distance 1 -> Distance 2 -> Select Line 1 -> Select Line 2 | |
| **Explode** (分解) | Breaks a complex entity into simpler ones. | Select Entity (e.g., Polyline, Block) | Polyline -> Lines/Arcs. |
| **Join** (結合) | Joins collinear or touching lines into one. | Select Collinear/Touching Lines | |

## 3. Selection & Utility (選択・補助)

| Command | Description | Behavior |
| :--- | :--- | :--- |
| **Select** (選択) | Selects objects for modification. | Click (Single), Window (Inside), Crossing (Touch) |
| **Deselect** (解除) | Removes objects from selection. | Shift + Click / Esc to clear all |
| **Delete** (削除) | Removes selected objects. | Select -> Delete Key |
| **Undo** (元に戻す) | Reverts the last action. | Ctrl+Z |
| **Redo** (やり直し) | Re-applies the undone action. | Ctrl+Y |
| **Pan** (画面移動) | Moves the view camera. | Middle Mouse Drag / Space + Drag |
| **Zoom** (ズーム) | Changes view magnification. | Mouse Wheel / Zoom Extents / Zoom Window |

## 4. Snapping & Constraints (スナップ・拘束)

| Feature | Description | Trigger |
| :--- | :--- | :--- |
| **Endpoint** (端点) | Snaps to the end of lines/arcs. | Cursor near endpoint |
| **Midpoint** (中点) | Snaps to the middle of lines/arcs. | Cursor near midpoint |
| **Center** (中心) | Snaps to the center of circles/arcs. | Cursor near center/edge |
| **Intersection** (交点) | Snaps to where two objects cross. | Cursor near intersection |
| **Perpendicular** (垂線) | Snaps perpendicular to an object. | During drawing command |
| **Tangent** (接線) | Snaps tangent to a circle/arc. | During drawing command |
| **Nearest** (近接) | Snaps to the nearest point on an object. | Cursor over object |
| **Parallel** (平行) | Constrains line to be parallel to another. | Guide / Constraint tool |
| **Ortho** (直交) | Restricts movement to X/Y axes. | Shift key / Toggle |

## 5. Layer & Properties (画層・プロパティ)

*   **Layer Management**: Create, Delete, Rename, Hide/Show, Lock/Unlock layers.
*   **Color**: Set color ByLayer or explicitly per object.
*   **Line Type**: Continuous, Dashed, Dotted, etc.
*   **Line Weight**: Thickness of the line.
