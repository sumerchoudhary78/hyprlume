import QtQuick
import Quickshell
import "models/AtlasGeometry.js" as Atlas

ShellRoot {
    function check() {
        const output = {x:-1200, y:100, width:2400, height:1600, scale:2, lastIpcObject:{transform:0}};
        const wide = Atlas.outputBounds(output, null);
        const left = Atlas.windowFrame(wide, [-1200,100], [600,800]);
        const clipped = Atlas.windowFrame(wide, [-1500,-100], [600,400]);
        output.lastIpcObject.transform = 1;
        const portrait = Atlas.outputBounds(output, null);
        const logical = Atlas.outputBounds(output, {width:800, height:1200});
        const fractional = Atlas.outputBounds({x:320, y:-600, width:1920, height:1080, scale:1.5}, null);
        const fractionalTile = Atlas.windowFrame(fractional, [960,-600], [640,720]);
        const unavailable = Atlas.windowFrame(null, [0,0], [300,400]);
        if (wide.w !== 1200 || wide.h !== 800 || left.x !== 0 || left.w !== 0.5 || left.h !== 1
                || clipped.x !== 0 || clipped.w !== 0.25 || clipped.y !== 0 || clipped.h !== 0.25
                || portrait.w !== 800 || portrait.h !== 1200 || logical.w !== portrait.w
                || fractional.w !== 1280 || fractional.h !== 720 || fractionalTile.x !== 0.5 || fractionalTile.w !== 0.5
                || unavailable || Atlas.windowFrame(wide, [9000,100], [200,200])
                || Atlas.windowFrame(wide, [0,0], [0,1]) || Atlas.windowFrame(wide, [], [])) {
            console.error("Wrong atlas projection", JSON.stringify([wide,left,clipped,portrait,logical]));
            Qt.exit(1);
            return;
        }
        console.log("HYPRLUME_ATLAS_GEOMETRY_PASS");
        Qt.quit();
    }
    Component.onCompleted: Qt.callLater(check)
}
