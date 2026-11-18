import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var backend
    required property var desktop
    required property var appearance
    required property bool saved
    required property int revision
    readonly property bool fileDialogVisible: picker.visible
    signal edited(string section, string key, var value)

    property int section: 0
    property int pending: 0
    property string message: ""
    property string error: ""
    property string profilePath: ""
    property string mime: "x-scheme-handler/https"
    property string desktopId: ""
    property var association: null
    spacing: 14

    function send(request) {
        error = "";
        message = "";
        pending = backend.send({op: "desktop_preferences", request: request}) || 0;
        if (!pending) error = backend.error;
    }

    function query(type) {
        root.mime = type;
        root.association = null;
        root.desktopId = "";
        root.send({action: "association", mime: type});
    }

    Component.onCompleted: query(mime)
    Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (id !== root.pending) return;
            root.pending = 0;
            if (!ok) { root.error = error; return; }
            if (result.mime) {
                root.association = result;
                root.desktopId = result.desktop_id || "";
            }
            if (result.profile) root.profilePath = result.profile;
            root.message = result.message || "";
        }
        function onOnlineChanged() {
            if (!root.backend.online) root.pending = 0;
            else if (root.section === 0) root.query(root.mime);
        }
    }

    Flow {
        Layout.fillWidth: true
        spacing: 6
        Repeater {
            model: ["Default apps", "Appearance & access", "Language & time"]
            GlassButton {
                required property string modelData
                required property int index
                objectName: "desktopSettingsSection" + index
                text: modelData
                compact: true
                primary: root.section === index
                onClicked: root.section = index
            }
        }
    }

    ColumnLayout {
        Layout.fillWidth: true
        visible: root.section === 0
        spacing: 12
        Flow {
            Layout.fillWidth: true
            spacing: 6
            Repeater {
                model: [
                    {label: "Web HTTPS", mime: "x-scheme-handler/https"},
                    {label: "Web HTTP", mime: "x-scheme-handler/http"},
                    {label: "File manager", mime: "inode/directory"},
                    {label: "Email", mime: "x-scheme-handler/mailto"},
                    {label: "PDF", mime: "application/pdf"},
                    {label: "Text", mime: "text/plain"}
                ]
                GlassButton {
                    required property var modelData
                    objectName: "desktopMime_" + modelData.label.replace(/ /g, "")
                    text: modelData.label
                    compact: true
                    primary: root.mime === modelData.mime
                    enabled: !root.pending
                    onClicked: root.query(modelData.mime)
                }
            }
        }
        RowLayout {
            Layout.fillWidth: true
            SettingsField {
                Layout.fillWidth: true
                controlName: "desktopMime"
                label: "MIME type or URL scheme"
                value: root.mime
                limit: 128
                onEdited: value => { root.mime = value.trim(); root.association = null; root.desktopId = ""; }
            }
            GlassButton { text: "Look up"; compact: true; enabled: !!root.mime && !root.pending; Layout.alignment: Qt.AlignBottom; onClicked: root.query(root.mime) }
        }
        Text {
            Layout.fillWidth: true
            visible: !!root.association
            text: "Current default  ·  " + (root.association?.app_name || "None")
            textFormat: Text.PlainText
            wrapMode: Text.Wrap
            color: Theme.ink
            font.pixelSize: 14
        }
        Flow {
            Layout.fillWidth: true
            spacing: 6
            Repeater {
                model: root.association?.choices || []
                GlassButton {
                    required property var modelData
                    text: modelData.name
                    compact: true
                    primary: root.desktopId === modelData.id
                    onClicked: root.desktopId = modelData.id
                }
            }
        }
        RowLayout {
            Layout.fillWidth: true
            SettingsField {
                Layout.fillWidth: true
                controlName: "desktopAssociationApp"
                label: "Installed application desktop ID"
                value: root.desktopId
                limit: 180
                onEdited: value => root.desktopId = value.trim()
            }
            GlassButton {
                objectName: "desktopSetDefault"
                text: "Set default"
                primary: true
                compact: true
                Layout.alignment: Qt.AlignBottom
                enabled: !!root.mime && !!root.desktopId && !root.pending
                onClicked: root.send({action: "set_association", mime: root.mime, desktop_id: root.desktopId})
            }
        }
    }

    ColumnLayout {
        Layout.fillWidth: true
        visible: root.section === 1
        spacing: 12
        RowLayout {
            Layout.fillWidth: true
            SettingsField {
                Layout.fillWidth: true
                controlName: "desktopWallpaper"
                label: "Wallpaper image path"
                value: root.desktop.wallpaper
                limit: 768
                onEdited: value => root.edited("desktop", "wallpaper", value)
            }
            GlassButton { objectName: "desktopWallpaperBrowse"; text: "Browse…"; compact: true; Layout.alignment: Qt.AlignBottom; onClicked: picker.open() }
            GlassButton { text: "Restore default"; compact: true; Layout.alignment: Qt.AlignBottom; enabled: !!root.desktop.wallpaper; onClicked: root.edited("desktop", "wallpaper", "") }
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "desktopGtkTheme"
            label: "GTK theme (e.g. Adwaita:dark, blank inherits)"
            value: root.desktop.gtk_theme
            limit: 80
            onEdited: value => root.edited("desktop", "gtk_theme", value.trim())
        }
        SettingsToggle {
            Layout.fillWidth: true
            controlName: "desktopFusion"
            label: "Use Qt Fusion style"
            checked: root.desktop.qt_style === "fusion"
            onToggled: value => root.edited("desktop", "qt_style", value ? "fusion" : "system")
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "desktopQtScale"
            label: "Qt app scale (1.0–2.0)"
            value: String(root.desktop.qt_scale)
            limit: 6
            onEdited: value => root.edited("desktop", "qt_scale", Number(value))
        }
        RowLayout {
            Layout.fillWidth: true
            Text { text: "App color preference"; color: Theme.ink; font.pixelSize: 14 }
            Item { Layout.fillWidth: true }
            Repeater {
                model: ["system", "dark", "light"]
                GlassButton {
                    required property string modelData
                    objectName: "desktopColor_" + modelData
                    text: modelData[0].toUpperCase() + modelData.slice(1)
                    compact: true
                    primary: root.appearance.app_color === modelData
                    onClicked: root.edited("appearance", "app_color", modelData)
                }
            }
        }
        SettingsToggle {
            Layout.fillWidth: true
            controlName: "desktopContrast"
            label: "High contrast"
            description: "Brighten Hyprlume text and borders and request higher contrast from portal-aware apps."
            checked: root.appearance.high_contrast
            onToggled: value => root.edited("appearance", "high_contrast", value)
        }
        SettingsToggle {
            Layout.fillWidth: true
            controlName: "desktopAccessibilityBus"
            label: "Enable Qt accessibility for new apps"
            checked: root.desktop.accessibility_bus
            onToggled: value => root.edited("desktop", "accessibility_bus", value)
        }
        GlassButton {
            objectName: "desktopOrca"
            text: "Start Orca screen reader"
            enabled: !root.pending
            onClicked: root.send({action: "start_screen_reader"})
        }
    }

    ColumnLayout {
        Layout.fillWidth: true
        visible: root.section === 2
        spacing: 12
        SettingsField {
            Layout.fillWidth: true
            controlName: "desktopLocale"
            label: "Installed locale (e.g. en_US.UTF-8, blank inherits)"
            value: root.desktop.locale
            limit: 80
            onEdited: value => root.edited("desktop", "locale", value.trim())
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "desktopTimezone"
            label: "App time zone (e.g. Asia/Kolkata, blank inherits)"
            value: root.desktop.timezone
            limit: 100
            onEdited: value => root.edited("desktop", "timezone", value.trim())
        }
        SettingsToggle {
            Layout.fillWidth: true
            controlName: "desktop24Hour"
            label: "24-hour shell clock"
            checked: root.desktop.clock_24h
            onToggled: value => root.edited("desktop", "clock_24h", value)
        }
    }

    ColumnLayout {
        Layout.fillWidth: true
        visible: root.section !== 0
        spacing: 8
        GlassButton {
            objectName: "desktopWriteEnvironment"
            text: "Write app profile"
            primary: true
            enabled: root.saved && !root.pending
            onClicked: root.send({action: "write_environment", revision: root.revision})
        }
        Text {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            text: "Save changes first. GTK, Qt, locale and time zone overrides apply to user-service launches after sign-in."
            color: Theme.inkFaint
            font.pixelSize: 12
        }
        Text {
            Layout.fillWidth: true
            visible: !!root.profilePath
            text: root.profilePath
            textFormat: Text.PlainText
            wrapMode: Text.WrapAnywhere
            color: Theme.cyan
            font.pixelSize: 11
        }
    }

    Text {
        Layout.fillWidth: true
        visible: !!text
        text: root.error || (root.pending ? "Waiting for the desktop service…" : root.message)
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
        color: root.error ? Theme.amber : Theme.inkDim
        font.pixelSize: 12
    }

    FileDialog {
        id: picker
        title: "Choose desktop wallpaper"
        nameFilters: ["Wallpaper images (*.png *.jpg *.jpeg *.webp)"]
        onAccepted: {
            const url = selectedFile.toString();
            if (url.startsWith("file:///")) root.edited("desktop", "wallpaper", decodeURIComponent(url.slice(7)));
        }
    }
}
