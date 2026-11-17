import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    readonly property var snapshot: backend.power
    readonly property var actions: [
        {kind:"suspend",title:"Suspend",icon:"moon"}, {kind:"logout",title:"Log out",icon:"logout"},
        {kind:"reboot",title:"Restart",icon:"refresh"}, {kind:"power_off",title:"Shut down",icon:"power"}
    ]
    radius: Theme.radiusPanel; tintTop: "#f01d2b46"; tint: "#f0081225"
    glowColor: Theme.cyan; secondaryColor: Theme.violet
    focus: true
    Component.onCompleted: forceActiveFocus()
    function close() { backend.send({op:"panel",panel:"power",open:false}); }
    function send(request) { backend.send({op:"power",request:request}); }
    Keys.onEscapePressed: {
        if (root.snapshot.prompt) root.send({action:"cancel",token:root.snapshot.prompt.token});
        else root.close();
    }
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 20; spacing: 15
        enabled: !root.snapshot.prompt
        RowLayout {
            Layout.fillWidth: true; Layout.minimumWidth: root.width - 40
            Icon { name: "power"; size: 23; color: Theme.cyan }
            ColumnLayout {
                Layout.fillWidth: true; spacing: 2
                Text { text: "Power & session"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 23; font.weight: Font.DemiBold }
                Text { text: "Energy for your next orbit."; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 10 }
            }
            IconTile { objectName: "refreshPower"; iconName: "refresh"; size: 28; enabled: !root.snapshot.busy; Accessible.name: "Refresh power state"; onClicked: root.send({action:"refresh"}) }
            IconTile { objectName: "closePower"; iconName: "close"; size: 28; Accessible.name: "Close power and session"; onClicked: root.close() }
        }
        ScrollView {
            id: scroll
            objectName: "powerScroll"
            Layout.fillWidth: true; Layout.fillHeight: true; clip: true
            contentWidth: availableWidth; ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                width: scroll.availableWidth; spacing: 18
                RowLayout {
                    Layout.fillWidth:true
                    GlassButton {objectName:"powerLock";text:"Lock screen";iconName:"lock";enabled:root.backend.locking.available;onClicked:root.backend.send({op:"lock"})}
                    Text {Layout.fillWidth:true;text:root.backend.locking.message;textFormat:Text.PlainText;wrapMode:Text.Wrap;color:Theme.inkFaint;font.family:Theme.fontSans;font.pixelSize:10}
                }
                BatteryCard { Layout.fillWidth: true; battery: root.snapshot.battery; reducedEffects: root.reducedEffects }
                Rectangle {
                    Layout.fillWidth:true; implicitHeight:idleContent.implicitHeight+24
                    radius:16; color:"#38182740"; border.color:Theme.rim
                    ColumnLayout {
                        id:idleContent; anchors.fill:parent; anchors.margins:12; spacing:8
                        RowLayout {
                            Layout.fillWidth:true
                            Text {Layout.fillWidth:true; text:"Presentation mode"; color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:13}
                            GlassButton {objectName:"powerPresentation"; compact:true; text:root.backend.idle.presentation?"On":"Off"; primary:root.backend.idle.presentation; onClicked:root.backend.send({op:"idle",request:{action:"presentation",enabled:!root.backend.idle.presentation}})}
                        }
                        Text {Layout.fillWidth:true; wrapMode:Text.Wrap; text:root.backend.idle.error || (root.backend.idle.presentation?"Idle timers paused. Lock-before-suspend stays active.":root.backend.idle.recording_inhibits?"Recording pauses automatic idle actions.":root.backend.idle.message); textFormat:Text.PlainText; color:root.backend.idle.error?Theme.amber:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:10}
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true; spacing: 8
                    Text { text: "Display brightness"; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 12; font.weight: Font.DemiBold }
                    Repeater {
                        model: root.snapshot.backlights
                        BacklightControl { required property var modelData; required property int index; Layout.fillWidth: true; backend: root.backend; light: modelData; lightIndex: index }
                    }
                    Text { Layout.fillWidth: true; visible: root.snapshot.backlights.length === 0; text: "No supported display backlight"; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11 }
                }
                Rectangle {
                    Layout.fillWidth: true; implicitHeight: profilesContent.implicitHeight + 30
                    radius: 18; color: "#38182740"; border.width: 1; border.color: Theme.rim
                    ColumnLayout {
                        id: profilesContent
                        anchors.fill: parent; anchors.margins: 15; spacing: 12
                        RowLayout {
                            Layout.fillWidth: true
                            Icon { name: "bolt"; size: 18; color: Theme.violet }
                            Text { Layout.fillWidth: true; text: "Power profile"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 14; font.weight: Font.DemiBold }
                        }
                        Flow {
                            Layout.fillWidth: true; spacing: 7
                            Repeater {
                                model: root.snapshot.profiles.choices
                                GlassButton {
                                    required property var modelData
                                    objectName: "powerProfile" + modelData
                                    text: ({"power-saver":"Save energy",balanced:"Balanced",performance:"Performance"})[modelData] || modelData
                                    compact: true; primary: root.snapshot.profiles.active === modelData
                                    opacity: root.snapshot.busy ? 0.42 : 1
                                    enabled: root.snapshot.profiles.available && !root.snapshot.busy && root.snapshot.profiles.active !== modelData
                                    onClicked: root.send({action:"profile",profile:modelData})
                                }
                            }
                        }
                        Text {
                            Layout.fillWidth: true; wrapMode: Text.Wrap
                            text: !root.snapshot.profiles.available ? root.snapshot.profiles.error || "Power profiles daemon is unavailable" : root.snapshot.profiles.degraded ? ({"lap-detected":"Performance limited while the laptop is on your lap.","high-operating-temperature":"Performance limited while the computer cools down."}[root.snapshot.profiles.degraded] || "Performance is temporarily limited by the hardware.") : "Choose how your computer balances energy and speed."
                            textFormat: Text.PlainText; color: root.snapshot.profiles.degraded ? Theme.amber : Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 10
                        }
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true; spacing: 10
                    RowLayout {
                        Layout.fillWidth: true
                        Text { Layout.fillWidth: true; text: "Session controls"; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 12; font.weight: Font.DemiBold }
                        Text { text: root.snapshot.session.available ? "Session " + root.snapshot.session.id : "Unavailable"; color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 10 }
                    }
                    GridLayout {
                        Layout.fillWidth: true; columns: 2; columnSpacing: 10; rowSpacing: 10
                        Repeater {
                            model: root.actions
                            GlassButton {
                                required property var modelData
                                objectName: "powerAction" + modelData.kind
                                Layout.fillWidth: true; text: modelData.title; iconName: modelData.icon
                                enabled: root.snapshot.session.available && !root.snapshot.busy && (modelData.kind==="suspend" ? root.snapshot.session.suspend==="yes" && root.backend.locking.available : ["yes","challenge"].indexOf(root.snapshot.session[modelData.kind]) >= 0)
                                onClicked: root.send({action:"prepare",kind:modelData.kind})
                            }
                        }
                    }
                    Text { Layout.fillWidth: true; wrapMode: Text.Wrap; text: root.snapshot.session.error || "Session actions require a separate confirmation. System inhibitors are respected."; textFormat: Text.PlainText; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 10 }
                }
            }
        }
        Text {
            Layout.fillWidth: true; wrapMode: Text.Wrap
            text: root.backend.error || root.snapshot.error || (root.snapshot.busy ? "Updating power state…" : root.snapshot.message || "Brightness controls support kernel backlights; minimum 5%.")
            textFormat: Text.PlainText; color: root.backend.error || root.snapshot.error ? Theme.amber : Theme.inkFaint
            font.family: Theme.fontSans; font.pixelSize: 10
        }
    }
    Loader {
        anchors.fill: parent; active: !!root.snapshot.prompt
        sourceComponent: SessionReview { backend: root.backend }
        onActiveChanged: if (!active) root.forceActiveFocus()
    }
}
