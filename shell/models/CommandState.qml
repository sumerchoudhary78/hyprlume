import QtQuick

// Derived UI state only. Transport and actions belong to the backend service.
QtObject {
    required property var snapshot

    readonly property bool busy: ["planning", "executing", "speaking"].indexOf(snapshot.phase) >= 0
    readonly property bool review: snapshot.confirmation !== null && snapshot.confirmation !== undefined
    readonly property bool blocked: busy || review
    readonly property string voice: snapshot.voice ?? "off"
    readonly property string transcript: snapshot.transcript ?? ""
    readonly property string activeApp: snapshot.active_app ?? ""
    readonly property var actions: snapshot.actions ?? []
    readonly property var receipts: snapshot.receipts ?? []
}
