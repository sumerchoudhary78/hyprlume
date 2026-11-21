import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Templates as T
import qs.components
import qs.theme

T.Button {
    id: root
    required property string iconName
    required property string detail
    Layout.fillWidth: true
    Layout.preferredWidth: 1
    implicitHeight: Theme.px(62)
    padding: Theme.px(10)
    hoverEnabled: true; focusPolicy: Qt.TabFocus
    Accessible.name: text + ", " + detail
    ToolTip.visible: hovered && (label.truncated || hint.truncated)
    ToolTip.text: Accessible.name
    ToolTip.delay: 550
    Keys.onReturnPressed: click()
    opacity: enabled ? 1 : 0.55

    background: Rectangle {
        radius: Theme.px(10)
        color: root.down ? Theme.pressed : root.hovered ? Qt.lighter(Theme.control, 1.15)
            : root.highlighted ? Theme.selection : Theme.control
        border.width: root.visualFocus ? 2 : 1
        border.color: root.visualFocus ? Theme.focusRing : root.highlighted || Theme.highContrast ? Theme.rimStrong : "transparent"
    }
    contentItem: RowLayout {
        spacing: Theme.px(9)
        Icon {
            name: root.iconName
            size: Theme.px(18)
            color: root.highlighted ? Theme.cyan : Theme.inkDim
        }
        ColumnLayout {
            Layout.fillWidth: true; Layout.minimumWidth: 0
            spacing: Theme.px(2)
            Text {
                id: label
                Layout.fillWidth: true
                text: root.text; elide: Text.ElideRight
                color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: Theme.px(13); font.weight: Font.DemiBold
            }
            Text {
                id: hint
                Layout.fillWidth: true
                text: root.detail; textFormat: Text.PlainText; elide: Text.ElideRight
                color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption
            }
        }
    }
    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
