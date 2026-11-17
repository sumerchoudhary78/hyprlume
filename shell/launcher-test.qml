import QtQuick
import Quickshell
import qs.services

ShellRoot {
    NativeApplications { id: catalogue }
    LauncherController { id: controller; apps: catalogue.apps }
    Timer {
        interval: 200; repeat: true; running: true
        property int attempts: 0
        onTriggered: {
            attempts++;
            if (!catalogue.apps.length && attempts < 30) return;
            try {
                const found = catalogue.apps.find(app => app.id === "hyprlume-qa" || app.id === "hyprlume-qa.desktop");
                if (!found || catalogue.apps.some(app => app.title === "Hidden QA" || app.title === "NoDisplay QA")) throw Error("index or visibility incorrect: " + JSON.stringify(catalogue.apps));
                controller.query = "quasar";
                if (controller.results.length !== 1 || controller.results[0] !== found) throw Error("keyword search failed");
                controller.query = "no such app";
                if (controller.results.length !== 0) throw Error("empty search failed");
                if (controller.activate({id:found.id})) throw Error("stale entry accepted");
                console.log("HYPRLUME_LAUNCHER_NATIVE_PASS"); Qt.quit();
            } catch (error) { console.error("HYPRLUME_LAUNCHER_FAIL " + error); Qt.exit(1); }
        }
    }
}
