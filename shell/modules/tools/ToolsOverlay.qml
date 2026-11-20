import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

Item {
    id: root
    required property var backend
    property string outputName:""
    property bool reducedEffects: false
    readonly property bool clipboardMode: backend.snapshot.desktop_panel === "clipboard"
    readonly property var clipboard: backend.clipboard
    readonly property var capture: backend.capture
    function close() { backend.send({op:"panel",panel:clipboardMode ? "clipboard" : "capture",open:false}); }
    function clip(request) {backend.send({op:"clipboard",request:request});}
    function captureRequest(request) {backend.send({op:"capture",request:request});}
    focus: true
    Component.onCompleted: forceActiveFocus()
    Keys.onEscapePressed: close()
    Rectangle {anchors.fill: parent; color:"#98050b18"; MouseArea {anchors.fill:parent; onClicked:root.close()}}
    GlassSurface {
        transform: PanelEntrance { reducedEffects: root.reducedEffects }
        anchors.centerIn: parent; width:Math.min(680,parent.width-32); height:Math.min(650,parent.height-32)
        radius:Theme.radiusPanel; reducedEffects:root.reducedEffects
        MouseArea {anchors.fill:parent; acceptedButtons:Qt.AllButtons}
        ColumnLayout {
            anchors.fill:parent; anchors.margins:22; spacing:15
            RowLayout {
                Layout.fillWidth:true
                Icon {name:"layers"; color:Theme.cyan; size:24}
                Text {Layout.fillWidth:true; text:root.clipboardMode ? "Your clipboard" : "Capture a moment"; color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:24}
                IconTile {objectName:"closeTools"; iconName:"close"; size:30; onClicked:root.close()}
            }
            RowLayout {
                GlassButton {objectName:"toolsClipboard"; text:"Clipboard"; primary:root.clipboardMode; onClicked:root.backend.send({op:"panel",panel:"clipboard",open:true})}
                GlassButton {objectName:"toolsCapture"; text:"Capture"; primary:!root.clipboardMode; onClicked:root.backend.send({op:"panel",panel:"capture",open:true})}
                GlassButton {objectName:"toolsDrives"; text:"Drives"; onClicked:root.backend.send({op:"panel",panel:"storage",open:true})}
                Item {Layout.fillWidth:true}
            }
            RowLayout {
                visible:root.clipboardMode; Layout.fillWidth:true
                GlassButton {objectName:"clipboardEnable"; text:root.clipboard.enabled ? "Pause history" : "Enable history"; primary:root.clipboard.enabled; enabled:root.clipboard.available; onClicked:root.clip({action:"enable",enabled:!root.clipboard.enabled})}
                Item {Layout.fillWidth:true}
                GlassButton {objectName:"clipboardClear"; text:"Clear all"; enabled:root.clipboard.items.length>0; onClicked:root.clip({action:"clear"})}
            }
            Text {
                Layout.fillWidth:true; visible:root.clipboardMode
                text:root.clipboard.available ? "Opt-in text history · memory only · up to 24 items / 3 KiB each. Known sensitive offers are skipped; pause before copying secrets." : "Install wl-clipboard to use text history."
                wrapMode:Text.Wrap; color:Theme.inkDim; font.family:Theme.fontSans; font.pixelSize:12
            }
            ListView {
                id:history; visible:root.clipboardMode; Layout.fillWidth:true; Layout.fillHeight:true
                model:root.clipboard.items; spacing:8; clip:true
                ScrollBar.vertical:ScrollBar {policy:ScrollBar.AsNeeded}
                delegate:Rectangle {
                    required property var modelData
                    required property int index
                    width:history.width; height:90; radius:15; color:"#5014263d"; border.color:Theme.rim
                    RowLayout {
                        anchors.fill:parent; anchors.margins:12; spacing:12
                        Text {Layout.fillWidth:true; Layout.fillHeight:true; verticalAlignment:Text.AlignVCenter; text:modelData.preview; textFormat:Text.PlainText; wrapMode:Text.Wrap; maximumLineCount:3; elide:Text.ElideRight; color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:13}
                        GlassButton {objectName:"clipboardCopy"+index; text:"Copy"; compact:true; onClicked:root.clip({action:"copy","id":modelData.id})}
                        IconTile {objectName:"clipboardRemove"+index; iconName:"close"; size:28; Accessible.name:"Remove clipboard item"; onClicked:root.clip({action:"remove","id":modelData.id})}
                    }
                }
                Text {anchors.centerIn:parent; visible:history.count===0; text:root.clipboard.enabled ? "Copied text will appear here" : "History is paused"; color:Theme.inkFaint; font.family:Theme.fontSans}
            }
            ColumnLayout {
                visible:!root.clipboardMode; Layout.fillWidth:true; Layout.fillHeight:true; spacing:16
                Rectangle {
                    Layout.fillWidth:true; Layout.preferredHeight:120; radius:20; color:root.capture.recording ? "#502b182a" : "#5013263e"; border.color:root.capture.recording ? Theme.amber : Theme.rim
                    ColumnLayout {
                        anchors.centerIn:parent; spacing:10
                        Icon {Layout.alignment:Qt.AlignHCenter; name:root.capture.recording ? "chart" : "expand"; color:root.capture.recording ? Theme.amber : Theme.cyan; size:30}
                        Text {text:root.capture.recording ? "Recording your screen" : "Ready when you are"; color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:18}
                    }
                }
                RowLayout {
                    Layout.fillWidth:true
                    GlassButton {objectName:"screenshotScreen"; text:"Screenshot"; iconName:"expand"; enabled:root.capture.screenshot_available && !root.capture.busy; onClicked:root.captureRequest({action:"screenshot",region:false})}
                    GlassButton {objectName:"screenshotRegion"; text:"Select area"; enabled:root.capture.screenshot_available && root.capture.region_available && !root.capture.busy; onClicked:root.captureRequest({action:"screenshot",region:true})}
                }
                RowLayout {
                    Layout.fillWidth:true
                    GlassButton {objectName:"recordScreen"; text:"Record this display"; enabled:root.capture.recording_available && !root.capture.busy; onClicked:root.captureRequest({action:"record",region:false,output:root.outputName || null})}
                    GlassButton {objectName:"recordRegion"; text:"Record area"; enabled:root.capture.recording_available && root.capture.region_available && !root.capture.busy; onClicked:root.captureRequest({action:"record",region:true})}
                    GlassButton {objectName:"captureStop"; text:root.capture.recording ? "Stop & save" : "Cancel"; primary:true; visible:root.capture.busy; onClicked:root.captureRequest({action:"stop"})}
                }
                Text {Layout.fillWidth:true; text:root.capture.message; textFormat:Text.PlainText; wrapMode:Text.Wrap; color:Theme.inkDim; font.family:Theme.fontSans; font.pixelSize:13}
                Text {Layout.fillWidth:true; visible:root.capture.output.length>0; text:"Last saved: "+root.capture.output; textFormat:Text.PlainText; wrapMode:Text.WrapAnywhere; color:Theme.cyan; font.family:Theme.fontMono; font.pixelSize:11}
                Item {Layout.fillHeight:true}
                Text {Layout.fillWidth:true; text:root.capture.screenshot_available && root.capture.recording_available && root.capture.region_available ? "Private captures stay in your Pictures/Hyprlume or Videos/Hyprlume folder. Recordings contain no audio." : "Install grim for screenshots, slurp for selection and wf-recorder for silent screen recording."; wrapMode:Text.Wrap; color:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:12}
            }
            Text {Layout.fillWidth:true; visible:text.length>0; text:(root.clipboardMode ? root.clipboard.error : root.capture.error) || root.backend.error; textFormat:Text.PlainText; color:Theme.amber; wrapMode:Text.Wrap; font.family:Theme.fontSans; font.pixelSize:12}
        }
    }
}
