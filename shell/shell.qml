//@ pragma Env QSG_RENDER_LOOP=threaded
//@ pragma DefaultEnv QS_DROP_EXPENSIVE_FONTS=1

import QtQuick
import Quickshell
import Quickshell.Io
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
    Metrics { id: metrics; active: !backendService.locking.active && backendService.settings.values.preferences.appearance.metrics && shellState.menu === "overview" }
    Workspaces {
        id: workspaces
        atlasVisible: !shellState.locked && !shellState.servicePanelOpen && (shellState.menu === "overview"
            || (!shellState.menu && (root.preview ? (!!focused && countFor(focused) === 0)
                : Quickshell.screens.some(screen => {
                    const workspace = activeOn(screen);
                    return !!workspace && countFor(workspace) === 0;
                }))))
    }
    ShellState { id: shellState; backend: backendService }
    IpcHandler {
        target: "desktop"
        function overview(): void { shellState.toggleMenu("overview", root.preview ? null : scene.item?.focusedScreen); }
        function quickSettings(): void { shellState.toggleMenu("controls", root.preview ? null : scene.item?.focusedScreen); }
    }
    Binding {target:Theme;property:"scale";value:backendService.settings.values.preferences.appearance.scale}
    Binding {target:Theme;property:"accentName";value:backendService.settings.values.preferences.appearance.accent}
    Binding { target: Theme; property: "highContrast"; value: backendService.settings.values.preferences.appearance.high_contrast }
    Binding { target: Theme; property: "wallpaperPath"; value: backendService.settings.values.preferences.desktop.wallpaper }
    Binding { target: Theme; property: "glassRefraction"; value: Theme.highContrast ? 0 : backendService.settings.values.preferences.appearance.refraction }

    Loader {
        id: scene
        Component.onCompleted: {
            if (root.preview) {
                setSource("PreviewWindow.qml", { backend: backendService, clock: clock, metrics: metrics, workspaces: workspaces, shellState: shellState });
            } else {
                setSource("DesktopShell.qml", {
                    backend: backendService,
                    clock: clock,
                    metrics: metrics,
                    workspaces: workspaces,
                    shellState: shellState
                });
            }
        }
    }
}
