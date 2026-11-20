import QtQuick
import QtQuick.Shapes
import qs.theme

// Paths are rasterized at the window's device pixel ratio.
Shape {
    id: root
    property var values: []
    property color accent: Theme.cyan
    readonly property real lineWidth: Theme.px(2)
    preferredRendererType: Shape.CurveRenderer

    readonly property var points: {
        if (values.length < 2)
            return [];
        const low = Math.min(...values);
        const high = Math.max(...values);
        const span = high - low;
        const plotWidth = Math.max(0, width - 2 * lineWidth);
        const plotHeight = Math.max(0, height - 2 * lineWidth);
        return values.map((value, index) => Qt.point(
            lineWidth + plotWidth * index / (values.length - 1),
            lineWidth + plotHeight * (span > 0 ? (high - value) / span : 0.5)
        ));
    }

    ShapePath {
        strokeWidth: -1
        fillGradient: LinearGradient {
            y2: root.height
            GradientStop { position: 0; color: Qt.rgba(root.accent.r, root.accent.g, root.accent.b, 0.22) }
            GradientStop { position: 1; color: "transparent" }
        }
        PathPolyline {
            path: root.points.length > 0 ? root.points.concat([
                Qt.point(root.width - root.lineWidth, root.height),
                Qt.point(root.lineWidth, root.height)
            ]) : []
        }
    }

    ShapePath {
        strokeWidth: root.lineWidth
        strokeColor: root.accent
        fillColor: "transparent"
        capStyle: ShapePath.RoundCap
        joinStyle: ShapePath.RoundJoin
        PathPolyline { path: root.points }
    }
}
