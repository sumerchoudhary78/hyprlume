import QtQuick
import QtQuick.Shapes
import qs.theme

// Consistent outlines without icon-theme, font, or per-icon shader layers.
Item {
    id: root
    property string name: ""
    property string glyph: ""
    property color color: Theme.ink
    property int size: Theme.px(18)
    property real iconOpacity: 1
    implicitWidth: size
    implicitHeight: size
    readonly property string path: {
        const n = name;
        if (n === "hyprlume") return "M5 5V20M18 5V20M5 12H18M17 3L21 1";
        if (n === "sliders") return "M2 7H6M12 7H22M2 17H12M18 17H22M12 7A3 3 0 1 1 6 7A3 3 0 1 1 12 7ZM18 17A3 3 0 1 1 12 17A3 3 0 1 1 18 17Z";
        if (n === "wifi-off") return "M3 3L21 21M2 8C3 7 4 6.5 5.5 6M10 4.5C15 4 19 6 22 8M5 12C7 10 9 9 11 9M16 10C17 10.5 18 11 19 12M8 16C10 13 14 13 16 16M12 20V20.2";
        if (n === "volume-muted") return "M3 9H7L12 5V19L7 15H3ZM16 9L22 15M22 9L16 15";
        if (n === "battery" || n === "battery-charging") return "M2 6H20V18H2ZM22 10V14"
            + (n === "battery-charging" ? "M12 8L8 12H13L10 16" : "");
        if (n === "workspace" || n === "workspace-active") return n === "workspace"
            ? "M4 6H20V18H4Z" : "M4 6H20V18H4ZM8 12L11 15L16 9";
        if (n === "microphone-off") return "M9 6C9 2 15 2 15 6V10M5 11V12C5 21 19 21 19 12M12 19V22M8 22H16M3 3L21 21";
        if (n === "minus") return "M5 12H19";
        if (n === "stop") return "M6 6H18V18H6Z";
        if (n === "chevron-right") return "M8 4L16 12L8 20";
        if (n === "chevron-down") return "M4 8L12 16L20 8";
        if (n === "power") return "M12 2V12M6 5C-2 11 3 23 12 23C21 23 26 11 18 5";
        if (n === "brightness") return "M16 12A4 4 0 1 1 8 12A4 4 0 1 1 16 12M12 1V4M12 20V23M1 12H4M20 12H23M4 4L6 6M18 18L20 20M4 20L6 18M18 6L20 4";
        if (n === "refresh") return "M20 8C17 0 3 3 3 12C3 22 18 25 21 15M20 2V8H14";
        if (n === "moon") return "M21 15C11 18 6 8 11 2C-2 4 0 22 12 22C16 22 19 20 21 15Z";
        if (n === "logout") return "M10 3H3V21H10M10 12H22M17 7L22 12L17 17";
        if (n === "wifi") return "M2 8C7 2 17 2 22 8M5 12C9 7 15 7 19 12M8 16C10 13 14 13 16 16M12 20V20.2";
        if (n === "bluetooth") return "M12 2V22L19 16L5 5M5 19L19 8L12 2";
        if (n.indexOf("terminal") >= 0) return "M4 6L10 12L4 18M13 18H21";
        if (n.indexOf("flash") >= 0 || n === "bolt") return "M14 2L4 14H11L10 22L21 9H14Z";
        if (n.indexOf("home") >= 0) return "M3 11L12 3L21 11M6 9V21H10V15H14V21H18V9";
        if (n.indexOf("grid") >= 0) return "M4 4H10V10H4ZM14 4H20V10H14ZM4 14H10V20H4ZM14 14H20V20H14Z";
        if (n.indexOf("microphone") >= 0) return "M9 6C9 2 15 2 15 6V12C15 16 9 16 9 12ZM5 11V12C5 21 19 21 19 12V11M12 19V22M8 22H16";
        if (n.indexOf("volume") >= 0) return "M3 9H7L12 5V19L7 15H3ZM16 8C19 10 19 14 16 16M19 5C24 9 24 15 19 19";
        if (n.indexOf("search") >= 0) return "M16 16L22 22M18 10A8 8 0 1 1 2 10A8 8 0 1 1 18 10";
        if (n.indexOf("cloud") >= 0 || n.indexOf("overcast") >= 0) return "M6 18C0 18 0 10 6 10C6 1 19 1 19 10C25 10 25 18 19 18Z";
        if (n.indexOf("package") >= 0 || n.indexOf("container") >= 0) return "M12 2L22 7V17L12 22L2 17V7ZM2 7L12 12L22 7M12 12V22M7 4.5L17 9.5";
        if (n.indexOf("science") >= 0 || n === "sparkles") return "M12 2L14.5 9.5L22 12L14.5 14.5L12 22L9.5 14.5L2 12L9.5 9.5ZM20 2V6M18 4H22";
        if (n.indexOf("script") >= 0) return "M6 2H15L20 7V22H4V2ZM14 2V8H20M9 12L7 15L9 18M13 12L15 15L13 18";
        if (n.indexOf("preferences") >= 0) return "M9 3H15L16 6L20 7L22 12L20 17L16 18L15 21H9L8 18L4 17L2 12L4 7L8 6ZM16 12A4 4 0 1 1 8 12A4 4 0 1 1 16 12";
        if (n.indexOf("list") >= 0 || n === "layers") return "M2 8L12 3L22 8L12 13ZM2 12L12 17L22 12M2 16L12 21L22 16";
        if (n.indexOf("monitor") >= 0 || n === "chart") return "M3 20V13M9 20V7M15 20V3M21 20V10";
        if (n.indexOf("folder") >= 0) return "M2 6H9L11 8H22V20H2ZM2 8V4H9L11 6H20V8";
        if (n.indexOf("go-next") >= 0 || n === "workflow") return "M6 3H10V7H6ZM16 17H20V21H16ZM2 17H6V21H2ZM8 7V11H18V17M8 11H4V17";
        if (n.indexOf("go-previous") >= 0) return "M15 5L8 12L15 19";
        if (n.indexOf("go-up") >= 0) return "M5 12L12 5L19 12M12 5V22";
        if (n.indexOf("go-down") >= 0) return "M5 12L12 19L19 12M12 2V19";
        if (n === "code") return "M8 5L2 12L8 19M16 5L22 12L16 19M14 3L10 21";
        if (n === "check") return "M4 12L9 18L21 5";
        if (n === "close") return "M5 5L19 19M19 5L5 19";
        if (n === "plus") return "M12 4V20M4 12H20";
        if (n === "expand") return "M3 9V3H9M15 3H21V9M21 15V21H15M9 21H3V15";
        if (n === "bell") return "M5 16V10C5 1 19 1 19 10V16L22 19H2ZM9 22H15";
        if (n === "cpu") return "M5 5H19V19H5ZM9 9H15V15H9ZM8 1V5M16 1V5M8 19V23M16 19V23M1 8H5M1 16H5M19 8H23M19 16H23";
        if (n === "globe") return "M22 12A10 10 0 1 1 2 12A10 10 0 1 1 22 12M2 12H22M12 2C5 8 5 16 12 22C19 16 19 8 12 2";
        if (n === "lock") return "M5 10H19V22H5ZM8 10V6C8 0 16 0 16 6V10M12 14V18";
        if (n === "drive") return "M2 14L6 5H18L22 14V20H2ZM2 14H22M16 17H18";
        if (n === "keyboard") return "M2 5H22V19H2ZM6 9H8M11 9H13M16 9H18M6 13H8M11 13H13M16 13H18M7 16H17";
        if (n === "play") return "M7 3L22 12L7 21Z";
        return "M4 4H20V20H4ZM8 8H16M8 12H16M8 16H12";
    }
    Shape {
        width: 24; height: 24
        scale: root.size / 24
        transformOrigin: Item.TopLeft
        opacity: root.iconOpacity
        preferredRendererType: Shape.CurveRenderer
        antialiasing: true
        visible: root.glyph.length === 0
        ShapePath {
            strokeColor: root.color
            strokeWidth: 2
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathSvg { path: root.path }
        }
    }
    Text {
        anchors.centerIn: parent
        visible: root.glyph.length > 0
        text: root.glyph
        color: root.color
        font.pixelSize: root.size
        opacity: root.iconOpacity
    }
}
