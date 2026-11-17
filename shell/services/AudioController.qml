import QtQuick

// Shared policy for native objects and isolated fixtures. Actions keep the
// selected object, rather than resolving a reusable numeric PipeWire ID.
Item {
    id: root
    property bool active: false
    property bool available: false
    property var nodes: []
    property var players: []
    property var defaultSink: null
    property var defaultSource: null
    property var playerChoice: null
    property string error: ""
    readonly property var outputs: nodes.filter(node => node.audio && !node.isStream && node.isSink)
    readonly property var inputs: nodes.filter(node => node.audio && !node.isStream && !node.isSink)
    readonly property var streams: nodes.filter(node => node.audio && node.isStream)
    readonly property var player: players.indexOf(playerChoice) >= 0 ? playerChoice
        : (players.find(candidate => candidate.isPlaying) || players[0] || null)
    signal defaultRequested(var node)

    function displayText(value, fallback, limit) {
        return String(value || fallback).replace(/[\u0000-\u001f\u007f]/g, " ").slice(0, limit);
    }
    function label(node) {
        return displayText(node ? (node.nickname || node.description || node.name) : "", node ? "Audio device" : "No device", 120);
    }
    function usable(node) {
        if (!available || !node || nodes.indexOf(node) < 0 || !node.ready || !node.audio) {
            error = "This audio device is no longer available.";
            return false;
        }
        error = "";
        return true;
    }
    function setVolume(node, volume) {
        if (!Number.isFinite(volume) || volume < 0 || volume > 1) {
            error = "Volume must be between 0 and 100%.";
            return;
        }
        if (usable(node)) node.audio.volume = volume;
    }
    function toggleMute(node) {
        if (usable(node)) node.audio.muted = !node.audio.muted;
    }
    function setDefault(node) {
        if (!usable(node)) return;
        if (node.isStream) { error = "Choose an input or output device."; return; }
        defaultRequested(node);
    }
    function selectPlayer(candidate) {
        if (players.indexOf(candidate) >= 0) { playerChoice = candidate; error = ""; }
    }
    function playback(candidate, action) {
        if (!candidate || players.indexOf(candidate) < 0 || !candidate.canControl) {
            error = "This player is no longer available for control.";
            return;
        }
        error = "";
        if (action === "toggle" && candidate.canTogglePlaying) candidate.togglePlaying();
        else if (action === "next" && candidate.canGoNext) candidate.next();
        else if (action === "previous" && candidate.canGoPrevious) candidate.previous();
        else error = "This playback action is not supported by the player.";
    }
}
