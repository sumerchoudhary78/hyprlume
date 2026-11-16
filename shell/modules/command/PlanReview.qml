import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// Review display and explicit confirmation/dismissal; no backend dependency.
ColumnLayout {
    id: root
    property var actions: []
    property bool review: false
    property bool busy: false
    signal confirmed()
    signal cancelled()
    visible: root.actions.length > 0
    Layout.fillWidth: true
    spacing: Theme.px(6)

    Repeater {
        model: root.actions

        delegate: Rectangle {
            required property string modelData
            required property int index

            Layout.fillWidth: true
            implicitHeight: Theme.px(34)
            radius: Theme.radiusTile
            color: Theme.glassInset
            border.width: 1
            border.color: Theme.divider

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.px(10)
                anchors.rightMargin: Theme.px(10)
                spacing: Theme.px(10)

                Text {
                    text: (index + 1).toString().padStart(2, "0")
                    color: Theme.cyan
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.fontCaption
                }

                Text {
                    Layout.fillWidth: true
                    text: modelData
                    color: Theme.ink
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontBody
                    elide: Text.ElideRight
                }
            }
        }
    }

    RowLayout {
        visible: root.review
        spacing: Theme.px(8)

        GlassButton {
            objectName: "confirmPlan"
            text: "Confirm"
            primary: true
            enabled: !root.busy
            onClicked: root.confirmed()
        }

        GlassButton {
            objectName: "cancelPlan"
            text: "Dismiss"
            enabled: !root.busy
            onClicked: root.cancelled()
        }

        Item { Layout.fillWidth: true }
    }
}
