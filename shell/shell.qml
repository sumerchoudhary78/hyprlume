//@ pragma Env QSG_RENDER_LOOP=threaded
//@ pragma DefaultEnv QS_DROP_EXPENSIVE_FONTS=1

import QtQuick
import Quickshell
import qs.services
import qs.theme

// Entry point. Services live here; visuals are loaded from the desktop module
// so `--preview` can substitute an ordinary window.
ShellRoot {
    id: root

    readonly property bool preview: Quickshell.env("HYPRLUME_PREVIEW") === "1"

    Backend { id: backendService; audio: audioService; tray: trayService; launcher: launcherService }
    NativeLauncher { id: launcherService; backend: backendService; active: backendService.online && ["launcher","switcher"].includes(backendService.snapshot.desktop_panel) }
    NativeTray { id: trayService; active: backendService.online && backendService.snapshot.desktop_panel === "tray" }
    NativeAudio { id: audioService; active: backendService.online && backendService.snapshot.desktop_panel === "audio" }
    Clock {
        id: clock
        use24Hour: backendService.settings.values.preferences.desktop.clock_24h
        localeName: backendService.settings.values.preferences.desktop.locale.split(/[.@]/)[0]
    }
    Metrics { id: metrics; active: !backendService.locking.active && backendService.settings.values.preferences.appearance.metrics && (root.preview || workspaces.desktopVisible) }
    Workspaces { id: workspaces }
    ShellState { id: shellState; backend: backendService }
    Binding {target:Theme;property:"scale";value:backendService.settings.values.preferences.appearance.scale}
    Binding {target:Theme;property:"accentName";value:backendService.settings.values.preferences.appearance.accent}
    Binding { target: Theme; property: "highContrast"; value: backendService.settings.values.preferences.appearance.high_contrast }
    Binding { target: Theme; property: "wallpaperPath"; value: backendService.settings.values.preferences.desktop.wallpaper }
    Binding { target: Theme; property: "glassRefraction"; value: Theme.highContrast ? 0 : backendService.settings.values.preferences.appearance.refraction }

    Loader {
        Component.onCompleted: {
            if (root.preview) {
                setSource("PreviewWindow.qml", { backend: backendService, clock: clock, metrics: metrics, workspaces: workspaces, state: shellState });
            } else {
                setSource("DesktopShell.qml", {
                    backend: backendService,
                    clock: clock,
                    metrics: metrics,
                    workspaces: workspaces,
                    state: shellState
                });
            }
        }
    }
}
