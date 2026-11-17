import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

FocusScope {
    id: root
    required property var agent
    focus: true
    Keys.onEscapePressed: root.agent.cancel()

    Rectangle {
        anchors.fill: parent
        color: "#b0081020"
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    }

    GlassSurface {
        anchors.centerIn: parent
        width: Math.min(520, parent.width - 28)
        height: Math.min(content.implicitHeight + 44, parent.height - 28)
        radius: Theme.radiusPanel
        tintTop: "#f5223451"
        tint: "#fa0a1529"
        glowColor: Theme.amber
        secondaryColor: Theme.cyan

        ScrollView {
            id: scroll
            anchors.fill: parent
            anchors.margins: 22
            clip: true
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                id: content
                width: scroll.availableWidth
                spacing: 14

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 12
                    Icon { name: "lock"; size: 25; color: Theme.amber }
                    Text {
                        Layout.fillWidth: true
                        text: "System authentication"
                        color: Theme.ink
                        font.family: Theme.fontSans
                        font.pixelSize: 22
                    }
                }
                Text {
                    Layout.fillWidth: true
                    text: root.agent.request?.message || "Authenticate to continue."
                    textFormat: Text.PlainText
                    wrapMode: Text.Wrap
                    color: Theme.ink
                    font.family: Theme.fontSans
                    font.pixelSize: 15
                }
                Text {
                    Layout.fillWidth: true
                    text: root.agent.request?.action || ""
                    textFormat: Text.PlainText
                    wrapMode: Text.WrapAnywhere
                    color: Theme.inkDim
                    font.family: Theme.fontMono
                    font.pixelSize: 11
                }
                ComboBox {
                    id: account
                    objectName: "polkitAccount"
                    Layout.fillWidth: true
                    model: root.agent.request?.accounts || []
                    visible: count > 1
                    enabled: root.agent.choosing
                    font.family: Theme.fontSans
                    Accessible.name: "Authentication account"
                }
                Text {
                    Layout.fillWidth: true
                    visible: account.count === 1
                    text: account.currentText
                    textFormat: Text.PlainText
                    elide: Text.ElideRight
                    font.family: Theme.fontSans
                    font.pixelSize: 13
                    color: Theme.cyan
                    Accessible.name: "Authentication account " + text
                }
                Loader {
                    id: field
                    Layout.fillWidth: true
                    active: root.agent.prompt !== null
                    visible: active
                    sourceComponent: ColumnLayout {
                        function submit() {
                            const response = input.text;
                            input.clear();
                            root.agent.respond(response);
                        }
                        spacing: 7
                        Text {
                            id: question
                            Layout.fillWidth: true
                            text: root.agent.prompt?.label || "Response"
                            textFormat: Text.PlainText
                            wrapMode: Text.WrapAnywhere
                            color: Theme.ink
                            font.family: Theme.fontSans
                            font.pixelSize: 13
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 42
                            radius: 12
                            color: "#b0071020"
                            border.color: input.activeFocus ? Theme.cyan : Theme.rim
                            TextInput {
                                id: input
                                objectName: "polkitResponse"
                                anchors.fill: parent
                                anchors.leftMargin: 12
                                anchors.rightMargin: 12
                                verticalAlignment: TextInput.AlignVCenter
                                clip: true
                                color: Theme.ink
                                font.family: Theme.fontSans
                                font.pixelSize: 15
                                maximumLength: 256
                                echoMode: root.agent.prompt?.echo ? TextInput.Normal : TextInput.Password
                                inputMethodHints: Qt.ImhSensitiveData | Qt.ImhNoPredictiveText |
                                    (echoMode === TextInput.Password ? Qt.ImhHiddenText : Qt.ImhNone)
                                activeFocusOnTab: true
                                Accessible.name: question.text
                                onAccepted: field.item.submit()
                                Component.onCompleted: forceActiveFocus()
                                Component.onDestruction: clear()
                            }
                        }
                    }
                }
                Text {
                    Layout.fillWidth: true
                    visible: text.length > 0
                    text: root.agent.notice || (!root.agent.choosing && !root.agent.prompt ? "Waiting for the system…" : "")
                    textFormat: Text.PlainText
                    wrapMode: Text.WrapAnywhere
                    color: root.agent.noticeIsError ? Theme.amber : Theme.inkDim
                    font.family: Theme.fontSans
                    font.pixelSize: 12
                }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    GlassButton {
                        objectName: "polkitCancel"
                        Layout.fillWidth: true
                        text: "Cancel"
                        onClicked: root.agent.cancel()
                    }
                    GlassButton {
                        objectName: "polkitContinue"
                        Layout.fillWidth: true
                        text: root.agent.choosing ? "Continue" : "Authenticate"
                        enabled: root.agent.choosing || field.item !== null
                        primary: true
                        onClicked: {
                            if (root.agent.choosing) root.agent.start(account.currentIndex);
                            else if (field.item) field.item.submit();
                        }
                    }
                }
            }
        }
    }
}
