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

    implicitHeight: Math.max(row.implicitHeight, Theme.px(20))

    RowLayout {
        id: row
        anchors.fill: parent
        spacing: root.spacing

        Icon {
            visible: root.iconName.length > 0
            name: root.iconName
            color: root.accent
            size: Theme.px(16)
        }

        Text {
            Layout.fillWidth: true
            text: root.title
            textFormat: Text.PlainText
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontLabel
            font.weight: Font.DemiBold
            elide: Text.ElideRight
            Accessible.role: Accessible.Heading
        }

        RowLayout {
            id: trailingRow
            spacing: Theme.px(6)
        }
    }
}
