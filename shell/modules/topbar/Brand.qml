import QtQuick
import QtQuick.Shapes
import qs.components
import qs.theme

// Shell identity: a lit bolt in a rounded frame, the name, and a version tag.
Item {
    id: root

    property bool compact: false

    implicitWidth: row.implicitWidth
    implicitHeight: Theme.px(40)

    Row {
        id: row
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.px(10)

        Item {
            width: Theme.px(34)
            height: Theme.px(34)
            anchors.verticalCenter: parent.verticalCenter

            Rectangle {
                anchors.fill: parent
                radius: Theme.px(11)
                gradient: Gradient {
                    GradientStop { position: 0.0; color: Qt.rgba(0.22, 0.34, 0.46, 0.85) }
                    GradientStop { position: 1.0; color: Qt.rgba(0.08, 0.14, 0.22, 0.9) }
                }
                border.width: 1
                border.color: Qt.rgba(0.55, 0.85, 1.0, 0.35)
            }

            Shape {
                anchors.centerIn: parent
                width: Theme.px(16)
                height: Theme.px(22)
                antialiasing: true

                ShapePath {
                    fillColor: Theme.cyan
                    strokeColor: "transparent"
                    startX: width * 0.58
                    startY: 0
                    PathLine { x: width * 0.14; y: height * 0.56 }
                    PathLine { x: width * 0.44; y: height * 0.56 }
                    PathLine { x: width * 0.38; y: height }
                    PathLine { x: width * 0.86; y: height * 0.40 }
                    PathLine { x: width * 0.56; y: height * 0.40 }
                    PathLine { x: width * 0.58; y: 0 }
                }
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: !root.compact
            text: "Hyprlume"
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontTitle
            font.weight: Font.DemiBold
            font.letterSpacing: 0.2
        }

        Chip {
            anchors.verticalCenter: parent.verticalCenter
            visible: !root.compact
            text: "shell"
            textColor: Theme.cyan
            fillColor: Qt.rgba(0.37, 0.85, 1.0, 0.12)
            strokeColor: Qt.rgba(0.37, 0.85, 1.0, 0.24)
        }
    }
}
