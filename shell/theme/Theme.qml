pragma Singleton

import QtQuick
import Quickshell

// Shared palette, typography and interaction timing. Spatial landmarks live
// together in Dashboard.qml; component geometry stays with its component.
//
// Sizes are logical pixels for a 1920x1080 workspace. Hyprland already applies
// the monitor scale factor, so a 4K screen at scale 2 needs no extra maths.
// `scale` is only a user preference for a denser or looser shell.
Singleton {
    id: theme

    // -- Scale ---------------------------------------------------------------

    property real scale: 1.0
    function px(value) {
        return Math.round(value * scale);
    }

    // -- Colour --------------------------------------------------------------
    // Cool neutral surfaces and one configurable action accent. Status colors
    // are paired with labels or shapes.

    property bool highContrast: false
    property string wallpaperPath: ""

    readonly property color ink: highContrast ? "#ffffff" : "#eef6f7"
    readonly property color inkDim: highContrast ? "#f0f6f7" : "#b5c7cf"
    readonly property color inkFaint: highContrast ? "#e1f0f4" : "#9db2bd"

    property string accentName: "cyan"
    readonly property color cyan: accentName === "violet" ? "#bba6ff" : accentName === "mint" ? "#94eab9" : "#8ce3ea"
    readonly property color amber: "#f3c286"
    readonly property color violet: "#b497ff"
    readonly property color mint: "#68f0b1"
    readonly property color rose: "#ff8f9e"

    readonly property color glassTop: highContrast ? "#0b1115" : "#1a2932"
    readonly property color glass: highContrast ? "#080d11" : "#111e27"
    readonly property color glassDeep: highContrast ? "#080d11" : "#111e27"
    readonly property color glassInset: highContrast ? "#080d11" : "#800a141c"
    readonly property color bar: highContrast ? "#090f13" : "#f20d1922"
    readonly property color field: highContrast ? "#05090c" : "#0a151d"
    readonly property color control: highContrast ? "#18252b" : "#14212b"
    readonly property color selection: Qt.rgba(cyan.r, cyan.g, cyan.b, highContrast ? 0.22 : 0.12)
    readonly property color onAccent: "#0b191f"
    readonly property color focusRing: highContrast ? "#ffffff" : cyan

    readonly property color rim: Qt.rgba(0.70, 0.84, 0.88, highContrast ? 0.85 : 0.22)
    readonly property color rimStrong: Qt.rgba(0.72, 0.90, 0.94, highContrast ? 0.96 : 0.49)
    readonly property color divider: Qt.rgba(0.72, 0.84, 0.89, highContrast ? 0.54 : 0.15)

    readonly property color hover: Qt.rgba(1, 1, 1, 0.06)
    readonly property color pressed: Qt.rgba(1, 1, 1, 0.10)

    property real glassRefraction: 16

    // -- Typography ----------------------------------------------------------
    // Install `inter-font` and `ttf-jetbrains-mono`; Qt falls back to the
    // system sans/mono when they are missing, so the shell still renders.

    readonly property string fontSans: "Inter"
    readonly property string fontMono: "JetBrains Mono"

    readonly property int fontMicro: px(10)
    readonly property int fontCaption: px(12)
    readonly property int fontBody: px(14)
    readonly property int fontLabel: px(15)
    readonly property int fontTitle: px(18)
    readonly property int fontHeadline: px(22)
    readonly property int fontDisplay: px(56)

    // -- Metrics -------------------------------------------------------------

    readonly property int radiusPanel: px(20)
    readonly property int radiusCard: px(16)
    readonly property int radiusTile: px(10)
    readonly property int radiusPill: px(999)

    readonly property int gap: px(16)
    readonly property int gapSmall: px(10)
    readonly property int gapTight: px(6)
    readonly property int margin: px(20)

    readonly property int barHeight: px(54)
    readonly property int barZone: barHeight

    readonly property int borderWidth: 1

    // -- Motion --------------------------------------------------------------
    // Short, non-looping transitions only. The shell must not hold a frame
    // timer open while idle.

    readonly property int motionFast: 110
    readonly property int motionBase: 180
    readonly property int motionSlow: 260
    readonly property int easing: Easing.OutCubic
}
