import QtQuick
import Quickshell

QtObject {
    id: root
    required property var backend

    readonly property bool reducedEffects: Quickshell.env("HYPRLUME_REDUCED_EFFECTS") === "1"
        || backend.settings.values.preferences.appearance.reduced_effects
    property string menu: ""
    property var panelScreen: null
    readonly property bool servicePanelOpen: backend.online && (backend.snapshot.panel_open
        || !!backend.snapshot.desktop_panel || backend.connectivity.panel_open || backend.notifications.panel_open)
    readonly property bool locked: backend.locking.active
    onServicePanelOpenChanged: {
        if (servicePanelOpen)
            menu = "";
        else if (!menu)
            panelScreen = null;
    }
    onLockedChanged: if (locked) closeMenu()
    property int windowRequest: 0
    property string windowError: ""
    property Connections windowReplies: Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (!root.windowRequest || id !== root.windowRequest) return;
            root.windowRequest = 0;
            if (!ok) root.windowError = error || "This window could not be focused";
            else if (root.menu === "overview") root.closeMenu();
        }
        function onOnlineChanged() {
            if (!root.backend.online && root.windowRequest) {
                root.windowRequest = 0;
                root.windowError = "Desktop service offline";
            }
        }
    }

    function focusWindow(entry) {
        if (!backend.online || locked || windowRequest || !entry?.id) return;
        windowError = "";
        windowRequest = backend.send({op: "focus_window", address: entry.id}) || 0;
        if (!windowRequest) windowError = backend.error || "Desktop service offline";
    }

    function toggleMenu(name, screen = null) {
        if (locked || servicePanelOpen)
            return;
        if (menu === name && (!screen || panelScreen === screen)) {
            closeMenu();
            return;
        }
        panelScreen = screen;
        windowError = "";
        menu = name;
    }

    function openFromBar(screen, operation) {
        if (!backend.online || locked)
            return;
        panelScreen = screen;
        backend.send(operation);
    }

    function closeMenu() {
        menu = "";
        panelScreen = null;
        windowRequest = 0;
        windowError = "";
    }

    readonly property var wifi: backend.connectivity.wifi
    readonly property var connection: backend.online && wifi.enabled
        ? wifi.interfaces.find(device => device.connected) : null
    readonly property string wifiLabel: !backend.online || !wifi.available ? "Unavailable"
        : !wifi.enabled ? "Off" : connection ? (connection.ssid || "Connected") : "Disconnected"
    readonly property var battery: backend.power.battery
    readonly property bool hasBattery: backend.online && battery.available && battery.present
        && Number.isFinite(battery.percentage)
    readonly property var sink: backend.audio?.defaultSink ?? null
    readonly property bool hasSound: !!(backend.audio?.available && sink?.ready && sink.audio)
    readonly property bool muted: hasSound && sink.audio.muted
}
