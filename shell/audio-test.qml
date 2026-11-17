import QtQuick
import Quickshell
import qs.services
import qs.fixtures

// Only used by scripts/qa/check-audio.py inside a private audio/display session.
ShellRoot {
    id: root
    property int stage: 0
    property int attempts: 0
    property var sink: null
    NativeAudio { id: native; active: true }
    AudioFixture { id: sample }
    AudioFixtureNode { id: stale; name: "Removed node" }
    FloatingWindow { visible: true; implicitWidth: 64; implicitHeight: 64 }
    function check(condition, message) {
        if (!condition) { console.error("HYPRLUME_AUDIO_FAIL " + message); Qt.exit(1); }
    }
    Component.onCompleted: {
        const node = sample.outputs[0], player = sample.player;
        const volume = node.audio.volume;
        for (const invalid of [NaN, Infinity, -0.1, 1.01]) sample.setVolume(node, invalid);
        check(node.audio.volume === volume, "invalid volumes changed a device");
        sample.setVolume(stale, 0.1);
        check(stale.audio.volume === 0.64 && sample.error.length > 0, "removed device accepted");
        sample.setDefault(sample.streams[0]);
        check(sample.defaultSink === node && sample.error.length > 0, "stream accepted as default");
        node.ready = false;
        sample.setVolume(node, 0.1);
        check(node.audio.volume === volume, "unbound node accepted");
        node.ready = true;
        sample.available = false;
        sample.toggleMute(node);
        check(!node.audio.muted, "unavailable device accepted");
        sample.available = true;
        sample.selectPlayer(sample.players[1]);
        sample.playback(sample.player, "next");
        check(sample.error.length > 0, "unsupported action accepted");
        sample.players = [player];
        check(sample.player === player, "removed player retained");
        sample.playback(sample.video, "toggle");
        check(!sample.video.isPlaying && sample.error.length > 0, "stale player accepted");
        console.log("HYPRLUME_AUDIO_POLICY_PASS invalid, unavailable, unbound and stale targets rejected");
    }
    Timer {
        interval: 100; repeat: true; running: true
        onTriggered: {
            if (++root.attempts > 120) { console.error("HYPRLUME_AUDIO_FAIL native service timeout at stage " + root.stage); Qt.exit(1); return; }
            if (root.stage === 0) {
                const node = native.outputs.find(candidate => candidate.name === "hyprlume-qa-output");
                if (!node || !node.ready || native.players.length !== 1) return;
                check(native.players[0].identity === "Hyprlume QA", "unexpected media player on private bus");
                root.sink = node;
                native.setVolume(node, 0.37);
                native.toggleMute(node);
                native.playback(native.player, "toggle");
                root.stage = 1;
            } else if (root.stage === 1) {
                if (Math.abs(root.sink.audio.volume - 0.37) > 0.005 || !root.sink.audio.muted || native.player.isPlaying) return;
                native.setDefault(root.sink);
                native.playback(native.player, "next");
                root.stage = 2;
            } else if (root.stage === 2) {
                if (native.player.trackTitle !== "Next fixture track") return;
                console.log("HYPRLUME_AUDIO_NATIVE_PASS PipeWire volume/mute and MPRIS playback round trips");
                Qt.quit();
            }
        }
    }
}
