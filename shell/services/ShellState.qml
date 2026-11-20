import QtQuick
import Quickshell

// Shell-wide state shared by surfaces that live in different windows.
// Kept deliberately small: each property is something two modules must agree
// on, not a dumping ground for view state.
QtObject {
    id: root

    property var backend: null

    // Reduced effects skips refraction and animated affordances.
    property bool reducedEffects: Quickshell.env("HYPRLUME_REDUCED_EFFECTS") === "1" || (backend !== null && backend.settings.values.preferences.appearance.reduced_effects)
}
