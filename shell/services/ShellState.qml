import QtQuick
import Quickshell

// Shell-wide state shared by surfaces that live in different windows.
// Kept deliberately small: each property is something two modules must agree
// on, not a dumping ground for view state.
QtObject {
    id: root

    // Index into the bar's mode strip. 0 is the terminal / command stage.
    property var backend: null
    property int mode: 0

    // Reserved for the reduced-effects setting (no bloom, no reflow motion).
    property bool reducedEffects: Quickshell.env("HYPRLUME_REDUCED_EFFECTS") === "1" || (backend !== null && backend.settings.values.preferences.appearance.reduced_effects)
}
