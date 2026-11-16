import QtQuick
import Quickshell
import Quickshell.Wayland

// Shared window base for every shell surface.
//
// The namespace convention (hyprlume-<surface>) is what the compositor layer
// rules match against for blur and rounding, so it must stay stable.
// Subclasses set anchors, size, layer, and keyboard focus.
PanelWindow {
    id: root

    required property string surfaceName

    color: "transparent"
    exclusiveZone: 0

    WlrLayershell.namespace: "hyprlume-" + surfaceName
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
}
