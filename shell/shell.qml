//@ pragma Env QSG_RENDER_LOOP=threaded
//@ pragma DefaultEnv QS_DROP_EXPENSIVE_FONTS=1

import QtQuick
import Quickshell
import qs.services

// Entry point. Services live here; visuals are loaded from the desktop module
// so `--preview` can substitute an ordinary window.
ShellRoot {
    id: root

    readonly property bool preview: Quickshell.env("HYPRLUME_PREVIEW") === "1"

    Backend { id: backend; audio: audioService }
    NativeAudio { id: audioService; active: backend.online && backend.snapshot.desktop_panel === "audio" }
    Clock { id: clock }
    Metrics { id: metrics; active: root.preview || workspaces.desktopVisible }
    Workspaces { id: workspaces }
    ShellState { id: shellState }

    Loader {
        Component.onCompleted: {
            if (root.preview) {
                setSource("PreviewWindow.qml", { backend: backend, clock: clock, metrics: metrics, workspaces: workspaces, state: shellState });
            } else {
                setSource("DesktopShell.qml", {
                    backend: backend,
                    clock: clock,
                    metrics: metrics,
                    workspaces: workspaces,
                    state: shellState
                });
            }
        }
    }
}
