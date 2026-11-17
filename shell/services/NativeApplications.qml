import QtQuick
import Quickshell

QtObject {
    // DesktopEntries watches the XDG directories. Metadata is rebuilt only
    // when the native index changes, never by spawning a process per query.
    readonly property var apps: DesktopEntries.applications.values.map(entry => ({
        handle:entry, id:entry.id, title:entry.name, subtitle:entry.genericName || entry.comment || entry.id,
        icon:entry.icon, search:[entry.id,entry.genericName,entry.comment,(entry.keywords || []).join(" ")].join(" ")
    })).sort((a,b) => a.title.localeCompare(b.title))
}
