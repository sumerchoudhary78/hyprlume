import QtQuick
import qs.theme

Item {
    id: root
    property string customPath: Theme.wallpaperPath
    property bool sampleOnly: false
    property bool failed: false
    readonly property bool custom: customPath !== "" && !failed
    readonly property real deviceScale: Window.window?.devicePixelRatio ?? Screen.devicePixelRatio
    // A monitor crossing the size threshold reloads once. A window resize
    // within the same bucket keeps the existing decoded texture.
    readonly property int decodeLimit: sampleOnly ? 1600
        : Math.max(width, height) * deviceScale > 2560 ? 4096 : 2560
    onCustomPathChanged: failed = false
    readonly property bool ready: image.status === Image.Ready
    clip: true
    Image {
        id: image
        // Aspect-crop the paint bounds while keeping the decoder's size a cap.
        readonly property real cover: Math.max(root.width / Math.max(1, implicitWidth),
            root.height / Math.max(1, implicitHeight))
        anchors.centerIn: parent
        width: implicitWidth * cover
        height: implicitHeight * cover
        source: root.custom
            ? "file://" + root.customPath.split("/").map(part => encodeURIComponent(part)).join("/")
            : "../../assets/backgrounds/tidal-eclipse.jpg"
        sourceSize: root.custom || root.sampleOnly ? Qt.size(root.decodeLimit, root.decodeLimit) : Qt.size(-1, -1)
        onStatusChanged: if (status === Image.Error && root.custom) root.failed = true
        asynchronous: true
        retainWhileLoading: true
        cache: true
        smooth: true
    }
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0; color: "#09040a16" }
            GradientStop { position: 0.5; color: "#12020a19" }
            GradientStop { position: 1; color: "#30020711" }
        }
    }
}
