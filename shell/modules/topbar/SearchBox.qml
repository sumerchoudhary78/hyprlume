import QtQuick
import qs.components

GlassButton {
    signal activated()
    iconName: "system-search"
    compact: true
    text: "Search"
    Accessible.name: "Open command search"
    onClicked: activated()
}
