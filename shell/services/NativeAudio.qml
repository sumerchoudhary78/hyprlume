import QtQuick
import Quickshell.Services.Pipewire
import Quickshell.Services.Mpris

AudioController {
    id: root
    available: Pipewire.ready
    // A bounded view protects layout/binding cost from huge processing graphs.
    nodes: Pipewire.nodes.values.filter(node => node.audio).slice(0, 64)
    players: Mpris.players.values.slice(0, 24)
    defaultSink: Pipewire.defaultAudioSink
    defaultSource: Pipewire.defaultAudioSource
    onDefaultRequested: node => {
        if (node.isSink) Pipewire.preferredDefaultAudioSink = node;
        else Pipewire.preferredDefaultAudioSource = node;
    }
    PwObjectTracker {
        // The closed panel only needs the current output/input for bar state.
        // No peak capture, shell commands, or periodic volume polling.
        objects: root.active ? root.nodes : [root.defaultSink, root.defaultSource]
    }
}
