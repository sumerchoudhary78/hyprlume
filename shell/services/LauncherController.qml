import QtQuick

Item {
    id: root
    property var apps: []
    property var windows: []
    property string mode: "apps"
    property string query: ""
    property bool busy: false
    property int pending: 0
    property string error: ""
    signal request(var operation)
    signal activated()
    function label(value) { return String(value || "").replace(/[\u0000-\u001f\u007f]/g, " ").slice(0, 300); }
    readonly property var results: {
        const terms = query.slice(0,128).toLowerCase().trim().split(/\s+/).filter(Boolean);
        const pool = mode === "windows" ? windows : apps;
        return pool.map((entry, index) => {
            const title = String(entry.title).toLowerCase();
            const search = title + " " + String(entry.search || entry.subtitle || "").toLowerCase();
            const score = terms.every(term => search.includes(term)) ? terms.reduce((n,term) => n + (title.startsWith(term) ? 30 : title.includes(term) ? 10 : 1), 0) : -1;
            return {entry:entry,score:score,index:index};
        }).filter(row => row.score >= 0).sort((a,b) => b.score-a.score || a.index-b.index).slice(0,32).map(row => row.entry);
    }
    function activate(entry) {
        const pool = mode === "windows" ? windows : apps;
        if (busy || !entry || pool.indexOf(entry) < 0) return false;
        error = "";
        request(mode === "windows" ? {op:"focus_window",address:entry.id} : {op:"launch_desktop",desktop_id:entry.id});
        return true;
    }
    function finish(ok, message) {
        busy = false; pending = 0;
        if (ok) { query = ""; activated(); } else error = label(message || "The request failed");
    }
}
