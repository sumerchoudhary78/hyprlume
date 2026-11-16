import QtQuick
import QtTest
import "../../shell"

TestCase {
    name: "GlassPanel"
    when: windowShown
    visible: true
    width: 760
    height: 850

    GlassPanel { id: panel; width: 620 }
    SignalSpy { id: request; target: panel; signalName: "request" }

    function snapshot(phase, confirmation) {
        return {phase: phase, message: "Ready", transcript: "", actions: confirmation === null ? [] : ["Switch to workspace 2"], receipts: [], confirmation: confirmation, panel_open: true, voice: "off", workspace: 1, active_app: ""}
    }
    function init() {
        panel.snapshot = snapshot("ready", null)
        findChild(panel, "commandInput").text = ""
        request.clear()
    }
    function test_keyboard_enter_previews() {
        const input = findChild(panel, "commandInput")
        input.text = "workspace 2"
        input.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(request.count, 1)
        compare(request.signalArguments[0][0].dry_run, true)
    }
    function test_run_is_explicit() {
        findChild(panel, "commandInput").text = "volume 30"
        wait(20)
        const button = findChild(panel, "runButton")
        mouseClick(button, button.width / 2, button.height / 2)
        compare(request.count, 1)
        compare(request.signalArguments[0][0].dry_run, false)
    }
    function test_review_disables_new_commands() {
        panel.snapshot = snapshot("review", 42)
        compare(findChild(panel, "commandInput").enabled, false)
        wait(20)
        const button = findChild(panel, "confirmButton")
        mouseClick(button, button.width / 2, button.height / 2)
        compare(request.count, 1)
        compare(request.signalArguments[0][0].token, 42)
    }
    function test_busy_disables_actions() {
        findChild(panel, "commandInput").text = "workspace 2"
        panel.snapshot = snapshot("planning", null)
        compare(findChild(panel, "previewButton").enabled, false)
        compare(findChild(panel, "runButton").enabled, false)
    }
}
