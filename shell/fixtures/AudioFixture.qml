import QtQuick
import Quickshell
import qs.services

AudioController {
    id: root
    available: Quickshell.env("HYPRLUME_AUDIO_STATE") !== "unavailable"
    nodes: available ? [speakers, headphones, microphone, browser] : []
    players: available ? [music, video] : []
    defaultSink: available ? speakers : null
    defaultSource: available ? microphone : null
    AudioFixtureNode { id: speakers; name: "Observatory speakers" }
    AudioFixtureNode { id: headphones; name: "Nebula headphones" }
    AudioFixtureNode { id: microphone; name: "Studio microphone"; isSink: false }
    AudioFixtureNode { id: browser; name: "Browser · Ambient session"; isStream: true; isSink: false }
    property QtObject music: QtObject {
        property string identity: "Music"
        property string trackTitle: "Between the stars"
        property string trackArtist: "Orbital Ensemble"
        property string trackAlbum: "Nightshift"
        property bool isPlaying: true
        property bool canControl: true
        property bool canTogglePlaying: true
        property bool canGoNext: true
        property bool canGoPrevious: true
        function togglePlaying() { isPlaying = !isPlaying; root.report("toggle", isPlaying); }
        function next() { root.report("next", identity); }
        function previous() { root.report("previous", identity); }
    }
    property QtObject video: QtObject {
        property string identity: "Video"
        property string trackTitle: "A quiet orbit"
        property string trackArtist: "Observatory sessions"
        property string trackAlbum: ""
        property bool isPlaying: false
        property bool canControl: true
        property bool canTogglePlaying: true
        property bool canGoNext: false
        property bool canGoPrevious: false
        function togglePlaying() { isPlaying = !isPlaying; root.report("toggle", isPlaying); }
        function next() { root.report("unexpected-next", identity); }
        function previous() { root.report("unexpected-previous", identity); }
    }
    onDefaultRequested: node => {
        if (node.isSink) defaultSink = node;
        else defaultSource = node;
        report("default", node.name);
    }
    function report(action, value) {
        if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"audio",action:action,value:value}));
    }
}
