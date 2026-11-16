pragma Singleton

import QtQuick
import Quickshell

// Design tokens. Everything visual in the shell reads from here; component
// files must not carry raw colours, durations, or font sizes.
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
    // Deep navy glass over a dark wallpaper. Accents are pulled from the
    // reference: cyan for structure, amber for the active element, violet for
    // the agent, mint for healthy state, rose for failure.

    readonly property color ink: "#e9f3fb"
    readonly property color inkDim: "#9fb2c6"
    readonly property color inkFaint: "#6b8095"

    readonly property color cyan: "#5fd8ff"
    readonly property color amber: "#f2a65a"
    readonly property color violet: "#a48bff"
    readonly property color mint: "#74e3b5"
    readonly property color rose: "#ff8f9e"

    readonly property color glassTop: Qt.rgba(0.18, 0.28, 0.38, 0.55)
    readonly property color glass: Qt.rgba(0.06, 0.10, 0.15, 0.78)
    readonly property color glassDeep: Qt.rgba(0.03, 0.06, 0.10, 0.86)
    readonly property color glassInset: Qt.rgba(0.02, 0.05, 0.08, 0.55)

    readonly property color rim: Qt.rgba(0.47, 0.74, 0.86, 0.26)
    readonly property color rimStrong: Qt.rgba(0.55, 0.88, 1.0, 0.55)
    readonly property color divider: Qt.rgba(0.60, 0.78, 0.90, 0.13)

    readonly property color hover: Qt.rgba(1, 1, 1, 0.06)
    readonly property color pressed: Qt.rgba(1, 1, 1, 0.10)

    // -- Typography ----------------------------------------------------------
    // Install `inter-font` and `ttf-jetbrains-mono`; Qt falls back to the
    // system sans/mono when they are missing, so the shell still renders.

    readonly property string fontSans: "Inter"
    readonly property string fontMono: "JetBrains Mono"

    readonly property int fontMicro: px(10)
    readonly property int fontCaption: px(11)
    readonly property int fontBody: px(13)
    readonly property int fontLabel: px(14)
    readonly property int fontTitle: px(17)
    readonly property int fontHeadline: px(22)
    readonly property int fontDisplay: px(56)

    // -- Metrics -------------------------------------------------------------

    readonly property int radiusPanel: px(24)
    readonly property int radiusCard: px(20)
    readonly property int radiusTile: px(14)
    readonly property int radiusPill: px(999)

    readonly property int gap: px(16)
    readonly property int gapSmall: px(10)
    readonly property int gapTight: px(6)
    readonly property int margin: px(20)

    readonly property int barHeight: px(60)
    readonly property int barZone: px(74)
    readonly property int railWidth: px(66)

    readonly property int borderWidth: 1

    // -- Motion --------------------------------------------------------------
    // Short, non-looping transitions only. The shell must not hold a frame
    // timer open while idle.

    readonly property int motionFast: 110
    readonly property int motionBase: 180
    readonly property int motionSlow: 260
    readonly property int easing: Easing.OutCubic

    // -- Workspace dock ------------------------------------------------------
    // The reference shows a fixed icon rail. Those icons are bound to
    // workspaces here, so the rail stays useful instead of decorative.

    readonly property var workspaceIcons: [
        "user-home",
        "view-grid-symbolic",
        "utilities-terminal",
        "package-x-generic",
        "weather-overcast-symbolic",
        "view-list-details",
        "preferences-system"
    ]

    function workspaceIcon(index) {
        if (index < 0)
            return "user-home";
        return workspaceIcons[index % workspaceIcons.length];
    }
}
