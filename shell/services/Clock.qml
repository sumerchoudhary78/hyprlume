import QtQuick

// Wall clock for the stage and the bar. One timer at 1 Hz; the rendered text
// only changes once a minute.
Item {
    id: root

    property date now: new Date()

    readonly property string time: Qt.formatDateTime(now, "HH:mm")
    readonly property string date: Qt.formatDateTime(now, "ddd, dd MMM")
    readonly property string iso: Qt.formatDateTime(now, "yyyy-MM-dd")

    Timer {
        interval: 1000
        repeat: true
        running: true
        onTriggered: root.now = new Date()
    }
}
