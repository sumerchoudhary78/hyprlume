import QtQuick
import qs.theme

Item {
    id: root
    property string customPath: Theme.wallpaperPath
    property bool failed: false
    readonly property bool custom: customPath !== "" && !failed
    onCustomPathChanged: failed = false
    readonly property bool ready: image.status === Image.Ready
    Image {
        id: image
        anchors.fill: parent
        source: root.custom
            ? "file://" + root.customPath.split("/").map(part => encodeURIComponent(part)).join("/")
            : "../../assets/backgrounds/observatory.png"
        sourceSize.width: root.custom ? 2560 : -1
        onStatusChanged: if (status === Image.Error && root.custom) root.failed = true
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
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
