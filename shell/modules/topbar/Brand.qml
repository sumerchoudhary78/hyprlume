import QtQuick
import qs.components
import qs.theme

Row {
    property bool compact: false
    spacing: Theme.px(8)

    Icon {
        name: "bolt"
        size: Theme.px(22)
        color: Theme.cyan
        anchors.verticalCenter: parent.verticalCenter
    }

    Text {
        visible: !parent.compact
        text: "Hyprlume"
        color: Theme.ink
        font.family: Theme.fontSans
        font.pixelSize: Theme.px(16)
        font.weight: Font.DemiBold
        anchors.verticalCenter: parent.verticalCenter
    }
}
