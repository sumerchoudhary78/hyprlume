import QtQuick

// Minute-aligned wall clock. No second-by-second wakeups for minute text.
Item {
    id: root

    property date now: new Date()
    property bool use24Hour: true
    property string localeName: ""
    readonly property var clockLocale: localeName ? Qt.locale(localeName) : Qt.locale()

    readonly property string time: now.toLocaleTimeString(clockLocale, use24Hour ? "HH:mm" : "h:mm AP")
    readonly property string date: now.toLocaleDateString(clockLocale, "ddd, dd MMM")
    readonly property string iso: Qt.formatDateTime(now, "yyyy-MM-dd")

    Timer {
        interval: 60000 - root.now.getSeconds() * 1000 - root.now.getMilliseconds()
        repeat: true
        running: true
        onTriggered: root.now = new Date()
    }
}
