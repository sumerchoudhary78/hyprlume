import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.components
import qs.theme

Item {
    id:root
    required property var backend
    property bool reducedEffects:false
    property var draft:JSON.parse(JSON.stringify(backend.settings.values))
    property int revision:backend.settings.revision
    property int pending:0
    property bool dirty:false
    property int tab:Math.max(0,Math.min(9,Number(Quickshell.env("HYPRLUME_SETTINGS_TAB")) || 0))
    readonly property bool fileDialogVisible: desktopPage.item?.fileDialogVisible || false
    property bool lua:true
    property string message:""
    function close(){backend.send({op:"panel",panel:"settings",open:false});}
    function edit(section,key,value){
        const next=JSON.parse(JSON.stringify(draft));
        if (section==="voice" || section==="llm") next[section][key]=value;
        else next.preferences[section][key]=value;
        draft=next;dirty=true;message="Unsaved changes";
    }
    function accept(state){root.forceActiveFocus();draft=JSON.parse(JSON.stringify(state.values));revision=state.revision;dirty=false;message=state.message;}
    function save(){
        const appearance=draft.preferences.appearance;
        if(!Number.isFinite(appearance.scale)||appearance.scale<0.85||appearance.scale>1.2){message="UI scale must be between 0.85 and 1.20";return;}
        if(!Number.isFinite(appearance.refraction)||appearance.refraction<0||appearance.refraction>24){message="Glass refraction must be between 0 and 24";return;}
        if(!Number.isInteger(draft.voice.threads)||draft.voice.threads<1||draft.voice.threads>4){message="Voice inference threads must be between 1 and 4";return;}
        if(!Number.isInteger(draft.llm.timeout_seconds)||draft.llm.timeout_seconds<1||draft.llm.timeout_seconds>120){message="Model timeout must be between 1 and 120 seconds";return;}
        pending=backend.send({op:"settings",request:{action:"save",revision:revision,values:draft}})||0;if(!pending)message=backend.error||"The desktop service is disconnected";}
    function reload(){pending=backend.send({op:"settings",request:{action:"reload"}})||0;}
    Connections {
        target:root.backend
        function onSettingsChanged(){if (!root.dirty && !root.pending) root.accept(root.backend.settings);}
        function onResponse(id,ok,result,error){if (root.pending!==id)return;root.pending=0;if(ok)root.accept(result);else root.message=error;}
    }
    focus:true
    Keys.onEscapePressed:close()
    Component.onCompleted:{accept(backend.settings);forceActiveFocus();}
    Rectangle {anchors.fill:parent; color:"#98050b18"; MouseArea {anchors.fill:parent; onClicked:root.close()}}
    GlassSurface {
        transform: PanelEntrance { reducedEffects: root.reducedEffects }
        anchors.centerIn:parent; width:Math.min(790,parent.width-32); height:Math.min(730,parent.height-32)
        radius:Theme.radiusPanel; reducedEffects:root.reducedEffects
        MouseArea {anchors.fill:parent; acceptedButtons:Qt.AllButtons}
        ColumnLayout {
            anchors.fill:parent; anchors.margins:22; spacing:14
            RowLayout {
                Layout.fillWidth:true
                Icon {name:"preferences-system"; size:24; color:Theme.cyan}
                Text {Layout.fillWidth:true; text:"Make it yours"; color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:24}
                IconTile {objectName:"closeSettings"; iconName:"close"; size:30; onClicked:root.close()}
            }
            Flow {
                Layout.fillWidth: true
                spacing: 5
                Repeater {
                    model:["Appearance","Voice & models","Shortcuts","Services","Idle & sleep","Portals","Input","Credentials","Desktop","Displays"]
                    GlassButton {required property string modelData; required property int index; objectName:"settingsTab"+index; text:modelData; compact:true; primary:root.tab===index; onClicked:root.tab=index}
                }
            }
            ScrollView {
                id:scroll
                Layout.fillWidth:true; Layout.fillHeight:true; clip:true
                contentWidth:availableWidth
                ColumnLayout {
                    width:scroll.availableWidth; spacing:18
                    ColumnLayout {
                        visible:root.tab===0; Layout.fillWidth:true; spacing:20
                        SettingsToggle {Layout.fillWidth:true; controlName:"settingsReduced"; label:"Reduced effects"; description:"Pause liquid motion and use simpler glass materials."; checked:root.draft.preferences.appearance.reduced_effects; onToggled:value=>root.edit("appearance","reduced_effects",value)}
                        SettingsToggle {Layout.fillWidth:true; controlName:"settingsMetrics"; label:"Live system metrics"; description:"Show CPU, memory and GPU readings while Overview is open."; checked:root.draft.preferences.appearance.metrics; onToggled:value=>root.edit("appearance","metrics",value)}
                        Text {text:"Accent"; color:Theme.inkDim; font.family:Theme.fontSans; font.pixelSize:13}
                        RowLayout {
                            Repeater {model:["cyan","violet","mint"]; GlassButton {required property string modelData; objectName:"settingsAccent"+modelData; text:modelData; primary:root.draft.preferences.appearance.accent===modelData; onClicked:root.edit("appearance","accent",modelData)}}
                        }
                        SettingsField {Layout.fillWidth:true; controlName:"settingsScale"; label:"UI scale (0.85–1.20)"; value:String(root.draft.preferences.appearance.scale); limit:5; onEdited:value=>root.edit("appearance","scale",Number(value))}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsRefraction"; label:"Glass refraction (0–24)"; value:String(root.draft.preferences.appearance.refraction); limit:5; onEdited:value=>root.edit("appearance","refraction",Number(value))}
                    }
                    ColumnLayout {
                        visible:root.tab===1; Layout.fillWidth:true; spacing:16
                        Text {Layout.fillWidth:true; text:"English models run locally. Save paths before or after downloading them. Voice starts off; these changes apply after restarting the daemon."; wrapMode:Text.Wrap; color:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:12}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsAsr"; label:"Speech-to-text model directory"; value:root.draft.voice.asr_dir; onEdited:value=>root.edit("voice","asr_dir",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsKws"; label:"Wake-word model directory"; value:root.draft.voice.kws_dir; onEdited:value=>root.edit("voice","kws_dir",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsTts"; label:"Text-to-speech model directory"; value:root.draft.voice.tts_dir; onEdited:value=>root.edit("voice","tts_dir",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsKeywords"; label:"Wake-word keywords file"; value:root.draft.voice.keywords_file; onEdited:value=>root.edit("voice","keywords_file",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsThreads"; label:"Voice inference threads (1–4)"; value:String(root.draft.voice.threads); limit:1; onEdited:value=>root.edit("voice","threads",Number(value))}
                        SettingsToggle {Layout.fillWidth:true; controlName:"settingsSpeak"; label:"Speak results"; checked:root.draft.voice.speak_results; onToggled:value=>root.edit("voice","speak_results",value)}
                        SettingsToggle {Layout.fillWidth:true; controlName:"settingsExecute"; label:"Run direct voice commands"; description:"Allow supported reversible direct commands. Model-generated plans still need review."; checked:root.draft.voice.execute_commands; onToggled:value=>root.edit("voice","execute_commands",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsEndpoint"; label:"Optional local HTTP model endpoint (blank disables it)"; value:root.draft.llm.endpoint||""; onEdited:value=>root.edit("llm","endpoint",value.trim()||null)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsModel"; label:"Decision model name"; value:root.draft.llm.model; limit:128; onEdited:value=>root.edit("llm","model",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsTimeout"; label:"Model request timeout (1–120 seconds)"; value:String(root.draft.llm.timeout_seconds); limit:3; onEdited:value=>root.edit("llm","timeout_seconds",Number(value))}
                    }
                    ColumnLayout {
                        visible:root.tab===2; Layout.fillWidth:true; spacing:14
                        Text {Layout.fillWidth:true; text:"Save your shortcuts, then use Input → Write profile to include them in Hyprland. The declarations below are also available for manual setup."; wrapMode:Text.Wrap; color:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:12}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsKeyLauncher"; label:"Applications"; value:root.draft.preferences.shortcuts.launcher; limit:48; onEdited:value=>root.edit("shortcuts","launcher",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsKeyWindows"; label:"Window switcher"; value:root.draft.preferences.shortcuts.windows; limit:48; onEdited:value=>root.edit("shortcuts","windows",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsKeyTools"; label:"Clipboard / capture"; value:root.draft.preferences.shortcuts.tools; limit:48; onEdited:value=>root.edit("shortcuts","tools",value)}
                        SettingsField {Layout.fillWidth:true; controlName:"settingsKeyLock"; label:"Session lock"; value:root.draft.preferences.shortcuts.lock; limit:48; onEdited:value=>root.edit("shortcuts","lock",value)}
                        RowLayout {GlassButton {text:"Lua"; compact:true; primary:root.lua; onClicked:root.lua=true} GlassButton {text:"hyprlang"; compact:true; primary:!root.lua; onClicked:root.lua=false}}
                        TextArea {Layout.fillWidth:true; Layout.preferredHeight:180; readOnly:true; selectByMouse:true; text:root.lua ? root.backend.settings.lua : root.backend.settings.hyprlang; wrapMode:TextEdit.WrapAnywhere; color:Theme.cyan; font.family:Theme.fontMono; font.pixelSize:11; background:Rectangle {radius:12; color:"#90102034"; border.color:Theme.rim}}
                    }
                    ColumnLayout {
                        visible:root.tab===3; Layout.fillWidth:true; spacing:22
                        Text {Layout.fillWidth:true; text:"Startup defaults apply after restarting the daemon. Use each service panel to change its current state."; wrapMode:Text.Wrap; color:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:12}
                        SettingsToggle {Layout.fillWidth:true; controlName:"settingsClipboardStartup"; label:"Start clipboard history enabled"; description:"Opt in to volatile text collection. Password offers may not be marked sensitive by their source."; checked:root.draft.preferences.services.clipboard_history; onToggled:value=>root.edit("services","clipboard_history",value)}
                        SettingsToggle {Layout.fillWidth:true; controlName:"settingsDndStartup"; label:"Start in Do Not Disturb"; description:"Keep notification history without showing popups."; checked:root.draft.preferences.services.notifications_dnd; onToggled:value=>root.edit("services","notifications_dnd",value)}
                    }
                    IdleSettings { visible:root.tab===4; Layout.fillWidth:true; policy:root.draft.preferences.idle; status:root.backend.idle; onEdited:(key,value)=>root.edit("idle",key,value) }
                    Loader {
                        active:root.tab===5
                        visible:active
                        Layout.fillWidth:true
                        sourceComponent:PortalStatus { backend:root.backend }
                    }
                    Loader {
                        active: root.tab === 6
                        visible: active
                        Layout.fillWidth: true
                        sourceComponent: InputSettings {
                            backend: root.backend
                            profile: root.draft.preferences.input
                            revision: root.revision
                            saved: !root.dirty && !root.pending
                            onEdited: (key, value) => root.edit("input", key, value)
                        }
                    }
                    Loader {
                        active: root.tab === 7
                        visible: active
                        Layout.fillWidth: true
                        sourceComponent: CredentialSettings { backend: root.backend }
                    }
                    Loader {
                        id: desktopPage
                        active: root.tab === 8
                        visible: active
                        Layout.fillWidth: true
                        sourceComponent: DesktopSettings {
                            backend: root.backend
                            desktop: root.draft.preferences.desktop
                            appearance: root.draft.preferences.appearance
                            saved: !root.dirty && !root.pending
                            revision: root.revision
                            onEdited: (section, key, value) => root.edit(section, key, value)
                        }
                    }
                    Loader {
                        active: root.tab === 9
                        visible: active
                        Layout.fillWidth: true
                        sourceComponent: DisplaySettings { backend: root.backend }
                    }
                }
            }
            Text {Layout.fillWidth:true; visible:root.backend.settings.restart_required; text:"Restart needed for saved voice/model or startup preferences."; color:Theme.amber; font.family:Theme.fontSans; font.pixelSize:12}
            Text {Layout.fillWidth:true; text:root.message || root.backend.error || root.backend.settings.message; textFormat:Text.PlainText; wrapMode:Text.Wrap; color:Theme.inkDim; font.family:Theme.fontSans; font.pixelSize:11; maximumLineCount:3; elide:Text.ElideRight}
            RowLayout {
                Layout.fillWidth:true
                Text {Layout.fillWidth:true; text:root.dirty ? "Unsaved changes" : "Saved locally"; color:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:11}
                GlassButton {objectName:"settingsReload"; text:"Reload / discard"; enabled:!root.pending; onClicked:root.reload()}
                GlassButton {objectName:"settingsSave"; text:root.pending ? "Saving…" : "Save changes"; primary:true; enabled:root.dirty && !root.pending; onClicked:root.save()}
            }
        }
    }
}
