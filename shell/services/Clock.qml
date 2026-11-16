import QtQuick

// Minute-aligned wall clock. No second-by-second wakeups for minute text.
Item {
    id: root

    property date now: new Date()

    readonly property string time: Qt.formatDateTime(now, "HH:mm")
    readonly property string date: Qt.formatDateTime(now, "ddd, dd MMM")
    readonly property string iso: Qt.formatDateTime(now, "yyyy-MM-dd")

    Timer {
        interval: 60000 - root.now.getSeconds() * 1000 - root.now.getMilliseconds()
        repeat: true
        running: true
        onTriggered: root.now = new Date()
    }
}
