import QtQuick
import qs.components

GlassButton {
    objectName: "openLauncher"
    signal activated()
    iconName: "system-search"
    compact: true
    text: "Search"
    Accessible.name: "Open application and window search"
    onClicked: activated()
}
