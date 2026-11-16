import QtQuick
import QtQuick.Layouts
import qs.theme

// Title row for a glass card. Trailing controls are supplied by the caller
// through the default property, e.g. a Chip or an icon button.
Item {
    id: root

    property string title: ""
    property string iconName: ""
    property color accent: Theme.cyan
    property real spacing: Theme.px(9)

    default property alias trailing: trailingRow.data

    implicitHeight: Math.max(titleLabel.implicitHeight, Theme.px(20))

    RowLayout {
        anchors.fill: parent
        spacing: root.spacing

        Icon {
            visible: root.iconName.length > 0
            name: root.iconName
            color: root.accent
            size: Theme.px(16)
        }

        Text {
            id: titleLabel
            text: root.title
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontLabel
            font.weight: Font.DemiBold
        }

        Item { Layout.fillWidth: true }

        RowLayout {
            id: trailingRow
            spacing: Theme.px(6)
        }
    }
}
